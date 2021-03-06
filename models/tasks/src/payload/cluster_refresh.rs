use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_models_core::cluster::discovery::ClusterDiscovery;

/// Cluster refresh task parameters.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterRefreshPayload {
    pub cluster: ClusterDiscovery,
    pub snapshot: bool,
}

impl ClusterRefreshPayload {
    pub fn new(cluster: ClusterDiscovery, snapshot: bool) -> ClusterRefreshPayload {
        ClusterRefreshPayload { cluster, snapshot }
    }
}
