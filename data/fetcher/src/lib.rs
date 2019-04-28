extern crate failure;
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
extern crate prometheus;
#[macro_use]
extern crate slog;

extern crate replicante_agent_client;
extern crate replicante_coordinator;
extern crate replicante_data_models;
extern crate replicante_data_store;
extern crate replicante_streams_events;
extern crate replicante_util_failure;

use std::time::Duration;

use failure::ResultExt;
use slog::Logger;

use replicante_agent_client::HttpClient;
use replicante_coordinator::NonBlockingLockWatcher;
use replicante_data_models::Agent;
use replicante_data_models::AgentStatus;
use replicante_data_models::ClusterDiscovery;
use replicante_data_store::store::Store;
use replicante_streams_events::EventsStream;
use replicante_util_failure::failure_info;
use replicante_util_failure::format_fail;

mod agent;
mod error;
mod metrics;
mod node;
mod shard;
mod snapshotter;

use self::agent::AgentFetcher;
use self::metrics::FETCHER_DURATION;
use self::metrics::FETCHER_ERRORS_COUNT;
use self::node::NodeFetcher;
use self::shard::ShardFetcher;

pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;
pub use self::metrics::register_metrics;
pub use self::snapshotter::Snapshotter;

struct ClusterIdentityChecker {
    display_name: Option<String>,
    id: String,
}

impl ClusterIdentityChecker {
    fn check_or_set_display_name(&mut self, display_name: &str, node_id: &str) -> Result<()> {
        if self.display_name.is_none() {
            self.display_name = Some(display_name.to_string());
            return Ok(());
        }
        let current = self.display_name.as_ref().unwrap();
        if current == display_name {
            return Ok(());
        }
        Err(ErrorKind::ClusterDisplayNameDoesNotMatch(
            current.to_string(),
            display_name.to_string(),
            node_id.to_string(),
        )
        .into())
    }

    fn check_id(&mut self, id: &str, node_id: &str) -> Result<()> {
        if self.id == id {
            return Ok(());
        }
        Err(
            ErrorKind::ClusterIdDoesNotMatch(self.id.clone(), id.to_string(), node_id.to_string())
                .into(),
        )
    }

    fn new(id: String, display_name: Option<String>) -> ClusterIdentityChecker {
        ClusterIdentityChecker { display_name, id }
    }
}

/// Node (agent and datastore) status fetching and processing logic.
///
/// The Fetcher is responsible for:
///
///   1. For each node:
///     1. Attempt to fetch agent info.
///     2. Persist `AgentInfo` record (if fetch succeeded).
///     3. Attempt to fetch node info (if agent is up).
///     4. Persist `Node` record (if fetch succeeded).
///     5. Attempt to fetch shards status (only if agent and datastore are up).
///     6. Persist each `Shard` record (if fetch succeeded).
///     7. Persist the `Agent` record.
pub struct Fetcher {
    agent: AgentFetcher,
    logger: Logger,
    node: NodeFetcher,
    shard: ShardFetcher,
    store: Store,
    timeout: Duration,
}

impl Fetcher {
    pub fn new(logger: Logger, events: EventsStream, store: Store, timeout: Duration) -> Fetcher {
        let agent = AgentFetcher::new(events.clone(), store.clone());
        let node = NodeFetcher::new(events.clone(), store.clone());
        let shard = ShardFetcher::new(events, store.clone());
        Fetcher {
            agent,
            logger,
            node,
            shard,
            store,
            timeout,
        }
    }

    /// Fetch an optimistic view of the cluster state.
    ///
    /// # Errors
    /// The frech process can encounter two kinds of errors:
    ///
    ///   * Core errors: store, coordinator, internal logic, ...
    ///   * Remote errors: agent is down, network issue, invalid data returned, ...
    ///
    /// Core errors are returned and interupt the fetching process early (if the primary store is
    /// failing to respond it is likely to fail again in a short time).
    ///
    /// Remote errors are logged and accounted for as part of the refresh process (a remote agent
    /// crashing should not prevent the full cluster from being checked).
    /// Refresh the optimistic view on a cluster state.
    ///
    /// If the refrsh process fails due to core-related issues (store errors,
    /// internal logic, ...) the process is aborted and the error propagated.
    ///
    /// If the refresh process encounters an agent error (invalid response or state,
    /// network issue, ...) the error is NOT propagated and is instead accounted for
    /// as part of the state refersh operation.
    // TODO: separatelly handle agnet/remote errors.
    pub fn fetch(&self, cluster: ClusterDiscovery, lock: NonBlockingLockWatcher) -> Result<()> {
        let _timer = FETCHER_DURATION.start_timer();
        let result = self.fetch_checked(cluster, lock).map_err(|error| {
            FETCHER_ERRORS_COUNT.inc();
            error
        });
        // TODO: propagate core errors.
        if let Err(error) = result {
            error!(self.logger, "Failed to process cluster refresh"; failure_info(&error));
        }
        Ok(())
    }

    /// Wrapped version of `fetch` so stats can be accounted for once.
    fn fetch_checked(&self, cluster: ClusterDiscovery, lock: NonBlockingLockWatcher) -> Result<()> {
        let cluster_id = cluster.cluster_id;
        debug!(self.logger, "Refreshing cluster state"; "cluster_id" => &cluster_id);
        let mut id_checker = ClusterIdentityChecker::new(cluster_id.clone(), cluster.display_name);
        self.store
            .cluster(cluster_id.clone())
            .mark_stale()
            .with_context(|_| ErrorKind::StoreWrite("cluster staleness"))?;

        for node in cluster.nodes {
            // Exit early if lock was lost.
            if !lock.inspect() {
                warn!(
                    self.logger, "Cluster fetcher lock lost, skipping futher nodes";
                    "cluster_id" => &cluster_id
                );
                return Ok(());
            }
            self.process_target(&cluster_id, &node, &mut id_checker)?;
        }
        Ok(())
    }

    fn process_target(
        &self,
        cluster: &str,
        node: &str,
        id_checker: &mut ClusterIdentityChecker,
    ) -> Result<()> {
        let client = HttpClient::make(node.to_string(), self.timeout)
            .with_context(|_| ErrorKind::AgentConnect(node.to_string()))?;
        let mut agent = Agent::new(cluster.to_string(), node.to_string(), AgentStatus::Up);
        let result = self
            .agent
            .process_agent_info(&client, cluster.to_string(), node.to_string());
        if let Err(error) = result {
            let message = format_fail(&error);
            agent.status = AgentStatus::AgentDown(message);
            self.agent.process_agent(agent)?;
            // TODO: figure out which errors to propagate.
            return Ok(());
        };

        let result = self.node.process_node(&client, id_checker);
        if let Err(error) = result {
            let message = format_fail(&error);
            agent.status = AgentStatus::NodeDown(message);
            self.agent.process_agent(agent)?;
            // TODO: figure out which errors to propagate.
            return Ok(());
        };

        let result = self.shard.process_shards(&client, cluster, node);
        if let Err(error) = result {
            let message = format_fail(&error);
            agent.status = AgentStatus::NodeDown(message);
            self.agent.process_agent(agent)?;
            // TODO: figure out which errors to propagate.
            return Ok(());
        };

        self.agent.process_agent(agent)
    }
}
