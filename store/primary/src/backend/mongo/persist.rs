use std::ops::Deref;
use std::sync::Arc;

use bson::bson;
use bson::doc;
use bson::Bson;
use failure::ResultExt;
use mongodb::db::ThreadedDatabase;
use mongodb::Client;
use mongodb::ThreadedClient;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_models_core::Agent as AgentModel;
use replicante_models_core::AgentInfo as AgentInfoModel;
use replicante_models_core::ClusterDiscovery as ClusterDiscoveryModel;
use replicante_models_core::Node as NodeModel;
use replicante_models_core::Shard as ShardModel;

use super::super::super::ErrorKind;
use super::super::super::Result;
use super::super::PersistInterface;
use super::common::replace_one;
use super::constants::COLLECTION_AGENTS;
use super::constants::COLLECTION_AGENTS_INFO;
use super::constants::COLLECTION_DISCOVERIES;
use super::constants::COLLECTION_NODES;
use super::constants::COLLECTION_SHARDS;
use super::document::AgentInfoDocument;
use super::document::NodeDocument;
use super::document::ShardDocument;

/// Persistence operations implementation using MongoDB.
pub struct Persist {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Persist {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Persist
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Persist { client, db, tracer }
    }
}

impl PersistInterface for Persist {
    fn agent(&self, agent: AgentModel, span: Option<SpanContext>) -> Result<()> {
        let filter = doc! {
            "cluster_id" => &agent.cluster_id,
            "host" => &agent.host,
        };
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS);
        let document = bson::to_bson(&agent).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Agent failed to encode as BSON document"),
        };
        replace_one(
            collection,
            filter,
            document,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
    }

    fn agent_info(&self, agent: AgentInfoModel, span: Option<SpanContext>) -> Result<()> {
        let filter = doc! {
            "cluster_id" => &agent.cluster_id,
            "host" => &agent.host,
        };
        let agent = AgentInfoDocument::from(agent);
        let collection = self.client.db(&self.db).collection(COLLECTION_AGENTS_INFO);
        let document = bson::to_bson(&agent).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("AgentInfo failed to encode as BSON document"),
        };
        replace_one(
            collection,
            filter,
            document,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
    }

    fn cluster_discovery(
        &self,
        discovery: ClusterDiscoveryModel,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let filter = doc! {"cluster_id" => &discovery.cluster_id};
        let collection = self.client.db(&self.db).collection(COLLECTION_DISCOVERIES);
        let document = bson::to_bson(&discovery).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("ClusterDiscovery failed to encode as BSON document"),
        };
        replace_one(
            collection,
            filter,
            document,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
    }

    fn node(&self, node: NodeModel, span: Option<SpanContext>) -> Result<()> {
        let filter = doc! {
            "cluster_id" => &node.cluster_id,
            "node_id" => &node.node_id,
        };
        let node = NodeDocument::from(node);
        let collection = self.client.db(&self.db).collection(COLLECTION_NODES);
        let document = bson::to_bson(&node).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Node failed to encode as BSON document"),
        };
        replace_one(
            collection,
            filter,
            document,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
    }

    fn shard(&self, shard: ShardModel, span: Option<SpanContext>) -> Result<()> {
        let filter = doc! {
            "cluster_id" => &shard.cluster_id,
            "node_id" => &shard.node_id,
            "shard_id" => &shard.shard_id,
        };
        let shard = ShardDocument::from(shard);
        let collection = self.client.db(&self.db).collection(COLLECTION_SHARDS);
        let document = bson::to_bson(&shard).with_context(|_| ErrorKind::MongoDBBsonEncode)?;
        let document = match document {
            Bson::Document(document) => document,
            _ => panic!("Shard failed to encode as BSON document"),
        };
        replace_one(
            collection,
            filter,
            document,
            span,
            self.tracer.as_ref().map(|tracer| tracer.deref()),
        )
    }
}