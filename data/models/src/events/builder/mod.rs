use chrono::DateTime;
use chrono::Utc;

use super::Event;
use super::EventPayload;

mod agent;
mod cluster;
mod node;
mod shard;
mod snapshot;

use self::agent::AgentBuilder;
use self::cluster::ClusterBuilder;
use self::node::NodeBuilder;
use self::shard::ShardBuilder;
use self::snapshot::SnapshotBuilder;

/// Build `Event`s, validating inputs.
#[derive(Default)]
pub struct EventBuilder {
    timestamp: Option<DateTime<Utc>>,
}

impl EventBuilder {
    pub fn new() -> EventBuilder {
        EventBuilder::default()
    }

    /// Specialise the builder into an agent event builder.
    pub fn agent(self) -> AgentBuilder {
        AgentBuilder::builder(self)
    }

    /// Specialise the builder into a cluster event builder.
    pub fn cluster(self) -> ClusterBuilder {
        ClusterBuilder::builder(self)
    }

    /// Specialise the builder into a node event builder.
    pub fn node(self) -> NodeBuilder {
        NodeBuilder::builder(self)
    }

    /// Specialise the builder into a shard event builder.
    pub fn shard(self) -> ShardBuilder {
        ShardBuilder::builder(self)
    }

    /// Specialise the builder into a snapshot event builder.
    pub fn snapshot(self) -> SnapshotBuilder {
        SnapshotBuilder::builder(self)
    }

    /// Set the event occurrence timestamp.
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

impl EventBuilder {
    /// Helper method to finish building an event.
    ///
    /// The specialised buidlers create the `EventPayload` argument.
    /// Add `Event` metadata is then added and/or generated by this method.
    fn build(self, data: EventPayload) -> Event {
        Event {
            payload: data,
            timestamp: self.timestamp.unwrap_or_else(Utc::now),
        }
    }
}
