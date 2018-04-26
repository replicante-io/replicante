/// Cluster description returned by the descovery system.
///
///
/// # Cluster membership
///
/// This model descibes the expected cluster members fully.
/// The list of nodes is used to determine if nodes are down and
/// when they are added and removed from the cluster.
///
///
/// # Cluster configuration (future plan)
/// 
/// Any configuration option that replicante should apply to the cluster is defined in this model.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterDiscovery {
    pub name: String,
    pub nodes: Vec<String>,
}

impl ClusterDiscovery {
    pub fn new<S>(name: S, nodes: Vec<String>) -> ClusterDiscovery
        where S: Into<String>,
    {
        ClusterDiscovery {
            name: name.into(),
            nodes,
        }
    }
}


/// Cluster metadata generated while fetching cluster state.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterMeta {
    pub name: String,
    pub kinds: Vec<String>,

    // BSON does not support unsigned integers so this must be signed.
    pub nodes: i32,
}

impl ClusterMeta {
    /// Create a new "top clusters" item.
    pub fn new<S1, S2>(name: S1, kind: S2, nodes: i32) -> ClusterMeta
        where S1: Into<String>,
              S2: Into<String>,
    {
        ClusterMeta {
            name: name.into(),
            kinds: vec![kind.into()],
            nodes,
        }
    }
}


#[cfg(test)]
mod tests {
    mod cluster_discovery {
        use serde_json;
        use super::super::ClusterDiscovery;

        #[test]
        fn from_json() {
            let payload = r#"{"name":"test","nodes":["a","b"]}"#;
            let cluster: ClusterDiscovery = serde_json::from_str(&payload).unwrap();
            let expected = ClusterDiscovery::new("test", vec!["a".into(), "b".into()]);
            assert_eq!(cluster, expected);
        }

        #[test]
        fn to_json() {
            let cluster = ClusterDiscovery::new("test", vec!["a".into(), "b".into()]);
            let payload = serde_json::to_string(&cluster).unwrap();
            let expected = r#"{"name":"test","nodes":["a","b"]}"#;
            assert_eq!(payload, expected);
        }
    }

    mod cluster_meta {
        use serde_json;
        use super::super::ClusterMeta;

        #[test]
        fn from_json() {
            let payload = r#"[{"name":"c1","kinds":["mongo"],"nodes":4},{"name":"c2","kinds":["redis"],"nodes":2}]"#;
            let clusters: Vec<ClusterMeta> = serde_json::from_str(payload).unwrap();
            let c1 = ClusterMeta::new("c1", "mongo", 4);
            let c2 = ClusterMeta::new("c2", "redis", 2);
            let expected = vec![c1, c2];
            assert_eq!(clusters, expected);
        }

        #[test]
        fn to_json() {
            let c1 = ClusterMeta::new("c1", "mongo", 4);
            let c2 = ClusterMeta::new("c2", "redis", 2);
            let clusters = vec![c1, c2];
            let payload = serde_json::to_string(&clusters).unwrap();
            let expected = r#"[{"name":"c1","kinds":["mongo"],"nodes":4},{"name":"c2","kinds":["redis"],"nodes":2}]"#;
            assert_eq!(payload, expected);
        }
    }
}