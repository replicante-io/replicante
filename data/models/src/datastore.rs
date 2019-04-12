use replicante_agent_models::DatastoreInfo as WireNode;
use replicante_agent_models::Shard as WireShard;

// Re-export some models for core to use.
// This opens up the option of replacing the implementation without changing dependants.
pub use replicante_agent_models::CommitOffset;
pub use replicante_agent_models::CommitUnit;
pub use replicante_agent_models::ShardRole;

/// Datastore version details.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Node {
    pub cluster_display_name: String,
    pub cluster_id: String,
    pub kind: String,
    pub node_id: String,
    pub version: String,
}

impl Node {
    pub fn new(node: WireNode) -> Node {
        let cluster_display_name = match node.cluster_display_name {
            Some(cluster_display_name) => cluster_display_name,
            None => node.cluster_id.clone(),
        };
        Node {
            cluster_display_name,
            cluster_id: node.cluster_id,
            kind: node.kind,
            node_id: node.id,
            version: node.version,
        }
    }
}

/// Information about a shard on a node.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Shard {
    pub cluster_id: String,
    pub commit_offset: Option<CommitOffset>,
    pub lag: Option<CommitOffset>,
    pub node_id: String,
    pub role: ShardRole,
    pub shard_id: String,
}

impl Shard {
    pub fn new<S1, S2>(cluster_id: S1, node_id: S2, shard: WireShard) -> Shard
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Shard {
            cluster_id: cluster_id.into(),
            commit_offset: shard.commit_offset,
            lag: shard.lag,
            node_id: node_id.into(),
            role: shard.role,
            shard_id: shard.id,
        }
    }
}

#[cfg(test)]
mod tests {
    mod node {
        use serde_json;
        use replicante_agent_models::DatastoreInfo as WireNode;
        use super::super::Node;

        #[test]
        fn from_json() {
            let payload = concat!(
                r#"{"cluster_display_name":"humans","cluster_id":"cluster","#,
                r#""kind":"DB","node_id":"Name","version":"1.2.3"}"#
            );
            let node: Node = serde_json::from_str(payload).unwrap();
            let wire = WireNode::new(Some("humans".into()), "cluster", "DB", "Name", "1.2.3");
            let expected = Node::new(wire);
            assert_eq!(node, expected);
        }

        #[test]
        fn to_json() {
            let wire = WireNode::new(None, "cluster", "DB", "Name", "1.2.3");
            let node = Node::new(wire);
            let payload = serde_json::to_string(&node).unwrap();
            let expected = concat!(
                r#"{"cluster_display_name":"cluster","cluster_id":"cluster","#,
                r#""kind":"DB","node_id":"Name","version":"1.2.3"}"#
            );
            assert_eq!(payload, expected);
        }
    }

    mod shard {
        use serde_json;
        use replicante_agent_models::CommitOffset;
        use replicante_agent_models::Shard as WireShard;
        use replicante_agent_models::ShardRole;
        use super::super::Shard;

        #[test]
        fn from_json() {
            let payload = concat!(
                r#"{"cluster_id":"cluster","commit_offset":{"unit":"seconds","value":54},"#,
                r#""shard_id":"shard","lag":null,"node_id":"node","role":"secondary"}"#
            );
            let shard: Shard = serde_json::from_str(payload).unwrap();
            let wire = WireShard::new(
                "shard", ShardRole::Secondary,
                Some(CommitOffset::seconds(54)), None
            );
            let expected = Shard::new("cluster", "node", wire);
            assert_eq!(shard, expected);
        }

        #[test]
        fn to_json() {
            let wire = WireShard::new(
                "shard", ShardRole::Secondary,
                Some(CommitOffset::seconds(54)), None
            );
            let shard = Shard::new("cluster", "node", wire);
            let payload = serde_json::to_string(&shard).unwrap();
            let expected = concat!(
                r#"{"cluster_id":"cluster","commit_offset":{"unit":"seconds","value":54},"#,
                r#""lag":null,"node_id":"node","role":"secondary","shard_id":"shard"}"#
            );
            assert_eq!(payload, expected);
        }
    }
}
