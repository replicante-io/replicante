use std::sync::Arc;

use bson::doc;
use bson::Document;
use failure::Fail;
use failure::ResultExt;
use mongodb::sync::Client;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

use replicante_externals_mongodb::operations::aggregate;
use replicante_externals_mongodb::operations::find;
use replicante_models_core::agent::Shard;

use super::super::ShardsInterface;
use super::constants::COLLECTION_SHARDS;
use super::document::ShardDocument;
use crate::store::shards::ShardsAttribures;
use crate::store::shards::ShardsCounts;
use crate::Cursor;
use crate::ErrorKind;
use crate::Result;

/// Return a document to count shards in given state as part of the $group stage.
fn aggregate_count_role(role: &'static str) -> Document {
    doc! {"$sum": {
        "$cond": {
            "if": {"$eq": ["$role", role]},
            "then": 1,
            "else": 0,
        }
    }}
}

/// Shards operations implementation using MongoDB.
pub struct Shards {
    client: Client,
    db: String,
    tracer: Option<Arc<Tracer>>,
}

impl Shards {
    pub fn new<T>(client: Client, db: String, tracer: T) -> Shards
    where
        T: Into<Option<Arc<Tracer>>>,
    {
        let tracer = tracer.into();
        Shards { client, db, tracer }
    }
}

impl ShardsInterface for Shards {
    fn counts(&self, attrs: &ShardsAttribures, span: Option<SpanContext>) -> Result<ShardsCounts> {
        // Let mongo figure out the counts with an aggregation.
        // Remember to count each shard only once across all nodes (and NOT once per node).
        let filter = doc! {"$match": {
            "cluster_id": &attrs.cluster_id,
            "stale": false,
        }};
        let count_nodes = doc! {"$sum": 1};
        let count_primaries = aggregate_count_role("primary");
        // First aggregate counts for each shard.
        let group_map = doc! {"$group": {
            "_id": {
                "cluster_id": "$cluster_id",
                "shard_id": "$shard_id",
            },
            "nodes": count_nodes,
            "primaries": count_primaries,
        }};
        // Then aggregate all shards into one document.
        let group_reduce = doc! {"$group": {
            "_id": "$cluster_id",
            "shards": {"$sum": 1},
            "primaries": {"$sum": "$primaries"},
        }};
        let pipeline = vec![filter, group_map, group_reduce];

        // Run aggrgation and grab the one and only (expected) result.
        let collection = self.client.database(&self.db).collection(COLLECTION_SHARDS);
        let mut cursor = aggregate(collection, pipeline, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?;
        let counts: ShardsCounts = match cursor.next() {
            Some(counts) => counts.with_context(|_| ErrorKind::MongoDBCursor)?,
            None => {
                return Ok(ShardsCounts {
                    shards: 0,
                    primaries: 0,
                })
            }
        };
        if cursor.next().is_some() {
            return Err(
                ErrorKind::DuplicateRecord("ShardsCounts", attrs.cluster_id.clone()).into(),
            );
        }
        Ok(counts)
    }

    fn iter(&self, attrs: &ShardsAttribures, span: Option<SpanContext>) -> Result<Cursor<Shard>> {
        let filter = doc! {"cluster_id": &attrs.cluster_id};
        let collection = self.client.database(&self.db).collection(COLLECTION_SHARDS);
        let cursor = find(collection, filter, span, self.tracer.as_deref())
            .with_context(|_| ErrorKind::MongoDBOperation)?
            .map(|item| item.map_err(|error| error.context(ErrorKind::MongoDBCursor).into()))
            .map(|result: Result<ShardDocument>| result.map(Shard::from));
        Ok(Cursor::new(cursor))
    }
}
