use std::thread::Builder as ThreadBuilder;
use std::thread::JoinHandle;
use std::thread::sleep;
use std::time::Duration;

use error_chain::ChainedError;
use slog::Logger;

use replicante_agent_client::Client;
use replicante_agent_client::HttpClient;
use replicante_agent_discovery::Config as BackendsConfig;
use replicante_agent_discovery::Discovery;
use replicante_agent_discovery::discover;
use replicante_data_models::Node;

use super::Result;


/// Agent discovery configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Discovery backends configuration.
    #[serde(default)]
    pub backends: BackendsConfig,

    /// Seconds to wait between discovery runs.
    #[serde(default = "Config::default_interval")]
    pub interval: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            backends: BackendsConfig::default(),
            interval: Config::default_interval(),
        }
    }
}

impl Config {
    /// Default value for `interval` used by serde.
    fn default_interval() -> u64 { 60 }
}


/// Components to periodically perform service discovery.
pub struct DiscoveryComponent {
    config: BackendsConfig,
    interval: Duration,
    logger: Logger,

    worker: Option<JoinHandle<()>>,
}

impl DiscoveryComponent {
    /// Creates a new agent discovery component.
    pub fn new(config: Config, logger: Logger) -> DiscoveryComponent {
        let interval = Duration::from_secs(config.interval);
        DiscoveryComponent {
            config: config.backends,
            interval,
            logger,
            worker: None,
        }
    }

    /// Starts the agent discovery process in a background thread.
    pub fn run(&mut self) -> Result<()> {
        let interval = self.interval.clone();
        let worker = DiscoveryWorker::new(
            self.config.clone(),
            self.logger.clone()
        );

        info!(self.logger, "Starting Agent Discovery thread");
        let logger = self.logger.clone();
        let thread = ThreadBuilder::new().name(String::from("Agent Discovery")).spawn(move || {
            loop {
                if let Err(err) = worker.run() {
                    let error = err.display_chain().to_string();
                    error!(logger, "Agent discovery iteration failed"; "error" => error);
                }
                sleep(interval.clone());
            }
        })?;
        self.worker = Some(thread);
        Ok(())
    }

    /// Wait for the worker thread to stop.
    pub fn wait(&mut self) -> Result<()> {
        info!(self.logger, "Waiting for Agent Discovery to stop");
        self.worker.take().map(|handle| handle.join());
        Ok(())
    }
}


/// Implements the discovery logic of a signle discovery loop.
struct DiscoveryWorker {
    config: BackendsConfig,
    logger: Logger,
}

impl DiscoveryWorker {
    /// Creates a discover worker.
    pub fn new(config: BackendsConfig, logger: Logger) -> DiscoveryWorker {
        DiscoveryWorker {
            config,
            logger,
        }
    }

    /// Runs a signle discovery loop.
    pub fn run(&self) -> Result<()> {
        debug!(self.logger, "Discovering agents ...");
        for agent in discover(self.config.clone())? {
            let agent = match agent {
                Ok(agent) => agent,
                Err(err) => {
                    let error = err.display_chain().to_string();
                    error!(self.logger, "Failed to fetch agent"; "error" => error);
                    continue;
                }
            };
            if let Err(err) = self.process(agent) {
                let error = err.display_chain().to_string();
                error!(self.logger, "Failed to process agent"; "error" => error);
            }
        }
        debug!(self.logger, "Agents discovery complete");
        Ok(())
    }

    /// Process a discovery result to fetch the node state.
    fn process(&self, discovery: Discovery) -> Result<()> {
        // TODO: replace with useful logic.
        let node = fetch_state(discovery)?;
        debug!(self.logger, "Discovered agent state: {:?}", node);
        Ok(())
    }
}


/*** NOTE: the code below will likely be moved when tasks are introduced  ***/
/// Converts an agent discovery result into a Node's status.
fn fetch_state(discovery: Discovery) -> Result<Node> {
    let client = HttpClient::new(discovery.target().clone())?;
    let info = client.info()?;
    let status = client.status()?;
    Ok(Node::new(info, status))
}
