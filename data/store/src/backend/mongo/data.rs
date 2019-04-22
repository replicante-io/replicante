use mongodb::coll::options::FindOptions;
use mongodb::coll::Collection;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use serde::Deserialize;

use replicante_data_models::Agent;
use replicante_data_models::AgentInfo;
use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;
use replicante_data_models::Event;
use replicante_data_models::Node;
use replicante_data_models::Shard;

use super::super::super::Cursor;
use super::super::super::Result;
use super::super::DataInterface;
use super::common::find_with_options;
use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;
use super::constants::COLLECTION_CLUSTER_META;
use super::constants::COLLECTION_DISCOVERIES;
use super::constants::COLLECTION_EVENTS;
use super::constants::COLLECTION_NODES;
use super::constants::COLLECTION_SHARDS;
use super::legacy::EventWrapper;

/// Scan all documents in a collection.
///
/// Intended for data validation purposes.
pub fn scan_collection<'de, T>(collection: Collection) -> Result<Cursor<T>>
where
    T: Deserialize<'de>,
{
    let filter = doc! {};
    let sort = doc! {"_id" => 1};
    let mut options = FindOptions::new();
    options.sort = Some(sort);
    find_with_options(collection, filter, options)
}

/// Data admin operations implementation using MongoDB.
pub struct Data {
    client: Client,
    db: String,
}

impl Data {
    pub fn new(client: Client, db: String) -> Data {
        Data { client, db }
    }
}

impl DataInterface for Data {
    fn agents(&self) -> Result<Cursor<Agent>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS);
        scan_collection(collection)
    }

    fn agents_info(&self) -> Result<Cursor<AgentInfo>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS_INFO);
        scan_collection(collection)
    }

    fn cluster_discoveries(&self) -> Result<Cursor<ClusterDiscovery>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_DISCOVERIES);
        scan_collection(collection)
    }

    fn clusters_meta(&self) -> Result<Cursor<ClusterMeta>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_CLUSTER_META);
        scan_collection(collection)
    }

    fn events(&self) -> Result<Cursor<Event>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_EVENTS);
        let cursor = scan_collection(collection)?;
        let cursor = cursor.map(|result: Result<EventWrapper>| result.map(Event::from));
        Ok(Cursor(Box::new(cursor)))
    }

    fn nodes(&self) -> Result<Cursor<Node>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_NODES);
        scan_collection(collection)
    }

    fn shards(&self) -> Result<Cursor<Shard>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_SHARDS);
        scan_collection(collection)
    }
}