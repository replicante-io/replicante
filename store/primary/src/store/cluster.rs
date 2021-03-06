use opentracingrust::SpanContext;

use replicante_models_core::cluster::discovery::ClusterDiscovery;
use replicante_models_core::cluster::ClusterSettings;

use crate::backend::ClusterImpl;
use crate::Result;

/// Operate on cluster-level models.
pub struct Cluster {
    cluster: ClusterImpl,
    attrs: ClusterAttribures,
}

impl Cluster {
    pub(crate) fn new(cluster: ClusterImpl, attrs: ClusterAttribures) -> Cluster {
        Cluster { cluster, attrs }
    }

    /// Query a `ClusterDiscovery` record, if any is stored.
    pub fn discovery<S>(&self, span: S) -> Result<Option<ClusterDiscovery>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.cluster.discovery(&self.attrs, span.into())
    }

    /// Mark the cluster state as stale until the data is updated.
    ///
    /// Stale data simply means that some cluster models listed below are marked as stale
    /// and can't be trusted to reflect the state of the cluster anymore.
    ///
    /// The staleness mark is automatically removed once records
    /// are updated by a cluster refresh operation.
    ///
    /// List of models that "go stale":
    ///
    ///   * AgentInfo
    ///   * Node
    ///   * Shard
    ///
    /// # Example
    /// If an agent goes down the current state of the node can't be determined
    /// but we still have the state the node was in before the agent failed.
    /// Instead of deliting this state, which would otherwise make the node
    /// appear new when it comes back online, we mark it as stale.
    pub fn mark_stale<S>(&self, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.cluster.mark_stale(&self.attrs, span.into())
    }

    /// Query a `ClusterSettings` record, if any is stored.
    pub fn settings<S>(&self, span: S) -> Result<Option<ClusterSettings>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.cluster.settings(&self.attrs, span.into())
    }
}

/// Attributes attached to all cluster-level operations.
pub struct ClusterAttribures {
    pub cluster_id: String,
    pub namespace: String,
}
