use failure::Fail;
use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;

use replicante_externals_mongodb::operations::scan_collection;
use replicante_models_core::Agent;
use replicante_models_core::AgentInfo;
use replicante_models_core::ClusterDiscovery;
use replicante_models_core::ClusterMeta;
use replicante_models_core::Node;
use replicante_models_core::Shard;

use super::super::DataInterface;
use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;
use super::constants::COLLECTION_CLUSTER_META;
use super::constants::COLLECTION_DISCOVERIES;
use super::constants::COLLECTION_NODES;
use super::constants::COLLECTION_SHARDS;
use super::document::AgentInfoDocument;
use super::document::NodeDocument;
use super::document::ShardDocument;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

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
        let cursor = scan_collection(collection)
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }

    fn agents_info(&self) -> Result<Cursor<AgentInfo>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS_INFO);
        let cursor = scan_collection(collection)
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<AgentInfoDocument>| result.map(AgentInfo::from));
        Ok(Cursor::new(cursor))
    }

    fn cluster_discoveries(&self) -> Result<Cursor<ClusterDiscovery>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_DISCOVERIES);
        let cursor = scan_collection(collection)
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }

    fn clusters_meta(&self) -> Result<Cursor<ClusterMeta>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_CLUSTER_META);
        let cursor = scan_collection(collection)
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()));
        Ok(Cursor::new(cursor))
    }

    fn nodes(&self) -> Result<Cursor<Node>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_NODES);
        let cursor = scan_collection(collection)
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<NodeDocument>| result.map(Node::from));
        Ok(Cursor::new(cursor))
    }

    fn shards(&self) -> Result<Cursor<Shard>> {
        let collection = self.client.db(&self.db).collection(COLLECTION_SHARDS);
        let cursor = scan_collection(collection)
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<ShardDocument>| result.map(Shard::from));
        Ok(Cursor::new(cursor))
    }
}
