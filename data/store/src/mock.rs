use std::collections::HashMap;
use std::sync::Mutex;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;
use replicante_data_models::Event;
use replicante_data_models::Node;
use replicante_data_models::Shard;

use super::InnerStore;
use super::Result;


/// A mock implementation of the storage layer for tests.
pub struct MockStore {
    pub agents: Mutex<HashMap<(String, String), Agent>>,
    pub agents_info: Mutex<HashMap<(String, String), AgentInfo>>,
    pub clusters_meta: Mutex<HashMap<String, ClusterMeta>>,
    pub discoveries: Mutex<HashMap<String, ClusterDiscovery>>,
    pub nodes: Mutex<HashMap<(String, String), Node>>,
    pub shards: Mutex<HashMap<(String, String, String), Shard>>,
    pub events: Mutex<Vec<Event>>,
}

impl InnerStore for MockStore {
    fn agent(&self, cluster: String, host: String) -> Result<Option<Agent>> {
        let agents = self.agents.lock().unwrap();
        let agent = agents.get(&(cluster, host)).map(|a| a.clone());
        Ok(agent)
    }

    fn agent_info(&self, cluster: String, host: String) -> Result<Option<AgentInfo>> {
        let agents_info = self.agents_info.lock().unwrap();
        let agent_info = agents_info.get(&(cluster, host)).map(|a| a.clone());
        Ok(agent_info)
    }

    fn cluster_discovery(&self, cluster: String) -> Result<Option<ClusterDiscovery>> {
        let discoveries = self.discoveries.lock().unwrap();
        let discovery = discoveries.get(&cluster).map(|c| c.clone());
        Ok(discovery)
    }

    fn cluster_meta(&self, cluster: String) -> Result<Option<ClusterMeta>> {
        let clusters = self.clusters_meta.lock().unwrap();
        let meta = clusters.get(&cluster).map(|c| c.clone());
        Ok(meta)
    }

    fn find_clusters(&self, search: String, _: u8) -> Result<Vec<ClusterMeta>> {
        let clusters = self.clusters_meta.lock().unwrap();
        let results = clusters.iter()
            .filter(|&(name, _)| name.contains(&search))
            .map(|(_, meta)| meta.clone())
            .collect();
        Ok(results)
    }
    
    fn node(&self, cluster: String, name: String) -> Result<Option<Node>> {
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&(cluster, name)).map(|n| n.clone());
        Ok(node)
    }

    fn persist_agent(&self, agent: Agent) -> Result<()> {
        let cluster = agent.cluster.clone();
        let host = agent.host.clone();
        let mut agents = self.agents.lock().unwrap();
        agents.insert((cluster, host), agent);
        Ok(())
    }

    fn persist_agent_info(&self, agent: AgentInfo) -> Result<()> {
        let cluster = agent.cluster.clone();
        let host = agent.host.clone();
        let mut agents_info = self.agents_info.lock().unwrap();
        agents_info.insert((cluster, host), agent);
        Ok(())
    }

    fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<()> {
        let name = cluster.name.clone();
        let mut discoveries = self.discoveries.lock().unwrap();
        discoveries.insert(name, cluster);
        Ok(())
    }

    fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<()> {
        let name = meta.name.clone();
        let mut clusters = self.clusters_meta.lock().unwrap();
        clusters.insert(name, meta);
        Ok(())
    }

    fn persist_event(&self, event: Event) -> Result<()> {
        let mut events = self.events.lock().unwrap();
        events.push(event);
        Ok(())
    }

    fn persist_node(&self, node: Node) -> Result<()> {
        let cluster = node.cluster.clone();
        let name = node.name.clone();
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert((cluster, name), node);
        Ok(())
    }

    fn persist_shard(&self, shard: Shard) -> Result<()> {
        let cluster = shard.cluster.clone();
        let node = shard.node.clone();
        let id = shard.id.clone();
        let mut shards = self.shards.lock().unwrap();
        shards.insert((cluster, node, id), shard);
        Ok(())
    }

    fn shard(&self, cluster: String, node: String, id: String) -> Result<Option<Shard>> {
        let shards = self.shards.lock().unwrap();
        let shard = shards.get(&(cluster, node, id)).map(|n| n.clone());
        Ok(shard)
    }

    fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        let clusters = self.clusters_meta.lock().unwrap();
        let mut results: Vec<ClusterMeta> = clusters.iter()
            .map(|(_, meta)| meta.clone())
            .collect();
        results.sort_by_key(|meta| meta.nodes);
        Ok(results)
    }
}

impl MockStore {
    /// Creates a new, empty, mock store.
    pub fn new() -> MockStore {
        MockStore {
            agents: Mutex::new(HashMap::new()),
            agents_info: Mutex::new(HashMap::new()),
            clusters_meta: Mutex::new(HashMap::new()),
            discoveries: Mutex::new(HashMap::new()),
            nodes: Mutex::new(HashMap::new()),
            shards: Mutex::new(HashMap::new()),
            events: Mutex::new(Vec::new()),
        }
    }
}


#[cfg(test)]
mod tests {
    mod agent {
        use std::sync::Arc;
        use replicante_data_models::Agent;
        use replicante_data_models::AgentStatus;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn get() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let agent = Agent::new("test", "node", AgentStatus::Up);
            let key = (String::from("test"), String::from("node"));
            mock.agents.lock().unwrap().insert(key.clone(), agent.clone());
            let stored = store.agent(key.0, key.1).unwrap().unwrap();
            assert_eq!(stored, agent);
        }

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let agent = Agent::new("test", "node", AgentStatus::Up);
            store.persist_agent(agent.clone()).unwrap();
            let stored = mock.agents.lock().expect("Faild to lock")
                .get(&("test".into(), "node".into()))
                .map(|n| n.clone()).expect("Agent not found");
            assert_eq!(agent, stored)
        }
    }

    mod agent_info {
        use std::sync::Arc;
        use replicante_data_models::AgentInfo;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let info = AgentInfo {
                cluster: "test".into(),
                host: "node".into(),
                version_checkout: "commit".into(),
                version_number: "1.2.3".into(),
                version_taint: "yep".into(),
            };
            store.persist_agent_info(info.clone()).unwrap();
            let stored = mock.agents_info.lock().expect("Faild to lock")
                .get(&("test".into(), "node".into()))
                .map(|n| n.clone()).expect("Agent not found");
            assert_eq!(info, stored);
        }
    }

    mod cluster_meta {
        use std::sync::Arc;
        use replicante_data_models::ClusterMeta;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn find_clusters() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let cluster1 = ClusterMeta::new("cluster1", "Redis", 44);
            let cluster2 = ClusterMeta::new("cluster2", "Redis", 44);
            mock.clusters_meta.lock().expect("Faild to lock")
                .insert("cluster1".into(), cluster1.clone());
            mock.clusters_meta.lock().expect("Faild to lock")
                .insert("cluster2".into(), cluster2.clone());
            let results = store.find_clusters("2", 33).unwrap();
            assert_eq!(results, vec![cluster2]);
        }

        #[test]
        fn found_meta() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let meta = ClusterMeta::new("test", "Redis", 44);
            mock.clusters_meta.lock().expect("Faild to lock").insert("test".into(), meta.clone());
            let found = store.cluster_meta("test").unwrap().unwrap();
            assert_eq!(found, meta);
        }

        #[test]
        fn missing_meta() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            assert!(store.cluster_meta("test").unwrap().is_none());
        }

        #[test]
        fn top_clusters() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let cluster1 = ClusterMeta::new("cluster1", "Redis", 44);
            let cluster2 = ClusterMeta::new("cluster2", "Redis", 4);
            mock.clusters_meta.lock().expect("Faild to lock")
                .insert("cluster1".into(), cluster1.clone());
            mock.clusters_meta.lock().expect("Faild to lock")
                .insert("cluster2".into(), cluster2.clone());
            let results = store.top_clusters().unwrap();
            assert_eq!(results, vec![cluster2, cluster1]);
        }

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let meta = ClusterMeta::new("test", "Redis", 44);
            store.persist_cluster_meta(meta.clone()).unwrap();
            let stored = mock.clusters_meta.lock().expect("Faild to lock")
                .get("test")
                .map(|n| n.clone()).expect("Cluster not found");
            assert_eq!(meta, stored);
        }
    }

    mod discovery {
        use std::sync::Arc;
        use replicante_data_models::ClusterDiscovery;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn found_discovery() {
            let cluster = ClusterDiscovery::new("test", vec!["test".into()]);
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            mock.discoveries.lock().expect("Faild to lock").insert("test".into(), cluster.clone());
            let found = store.cluster_discovery("test").unwrap().unwrap();
            assert_eq!(found, cluster);
        }

        #[test]
        fn missing_discovery() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            assert!(store.cluster_discovery("test").unwrap().is_none());
        }

        #[test]
        fn persist() {
            let cluster = ClusterDiscovery::new("test", vec!["test".into()]);
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            store.persist_discovery(cluster.clone()).unwrap();
            let stored = mock.discoveries.lock().expect("Faild to lock")
                .get("test")
                .map(|n| n.clone()).expect("Cluster not found");
            assert_eq!(cluster, stored);
        }
    }

    mod event {
        use std::sync::Arc;
        use replicante_data_models::ClusterDiscovery;
        use replicante_data_models::Event;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let cluster = ClusterDiscovery::new("test", vec!["test".into()]);
            let event = Event::builder().cluster().new(cluster);
            store.persist_event(event.clone()).unwrap();
            let stored = mock.events.lock().expect("Faild to lock").clone();
            assert_eq!(vec![event], stored);
        }
    }

    mod node {
        use std::sync::Arc;
        use replicante_agent_models::DatastoreInfo;
        use replicante_data_models::Node;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let node = Node::new(DatastoreInfo::new("cluster", "kind", "name", "version"));
            store.persist_node(node.clone()).unwrap();
            let key = (String::from("cluster"), String::from("name"));
            let stored = mock.nodes.lock().expect("Faild to lock")
                .get(&key)
                .map(|n| n.clone()).expect("Cluster not found");
            assert_eq!(node, stored);
        }
    }

    mod shards {
        use std::sync::Arc;
        use replicante_agent_models::Shard as WireShard;
        use replicante_data_models::Shard;
        use replicante_data_models::ShardRole;

        use super::super::super::Store;
        use super::super::MockStore;

        #[test]
        fn persist() {
            let mock = Arc::new(MockStore::new());
            let store = Store::mock(Arc::clone(&mock));
            let shard = Shard::new("cluster", "node", WireShard::new(
                "id", ShardRole::Primary, None, 1
            ));
            store.persist_shard(shard.clone()).unwrap();
            let key = (String::from("cluster"), String::from("node"), String::from("id"));
            let stored = mock.shards.lock().expect("Faild to lock")
                .get(&key)
                .map(|n| n.clone()).expect("Shard not found");
            assert_eq!(shard, stored)
        }
    }
}