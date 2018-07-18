use bson;
use bson::Bson;

use mongodb::Client;
use mongodb::ThreadedClient;
use mongodb::coll::Collection;
use mongodb::coll::options::FindOptions;
use mongodb::coll::options::UpdateOptions;
use mongodb::db::ThreadedDatabase;

use regex;
use slog::Logger;

use replicante_data_models::ClusterDiscovery;
use replicante_data_models::ClusterMeta;

use super::super::super::Result;
use super::super::super::ResultExt;

use super::constants::COLLECTION_CLUSTER_META;
use super::constants::COLLECTION_DISCOVERIES;

use super::constants::FAIL_FIND_CLUSTERS;
use super::constants::FAIL_FIND_CLUSTER_DISCOVERY;
use super::constants::FAIL_FIND_CLUSTER_META;

use super::constants::FAIL_PERSIST_CLUSTER_DISCOVERY;
use super::constants::FAIL_PERSIST_CLUSTER_META;

use super::constants::FAIL_TOP_CLUSTERS;
use super::constants::TOP_CLUSTERS_LIMIT;

use super::metrics::MONGODB_OPS_COUNT;
use super::metrics::MONGODB_OPS_DURATION;
use super::metrics::MONGODB_OP_ERRORS_COUNT;


/// Subset of the `Store` trait that deals with clusters.
pub struct ClusterStore {
    client: Client,
    db: String,
    logger: Logger,
}

impl ClusterStore {
    pub fn new(client: Client, db: String, logger: Logger) -> ClusterStore {
        ClusterStore { client, db, logger }
    }

    pub fn cluster_discovery(&self, cluster: String) -> Result<Option<ClusterDiscovery>> {
        let filter = doc!{"cluster" => cluster};
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_discoveries();
        let discovery = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_CLUSTER_DISCOVERY)?;
        timer.observe_duration();
        if discovery.is_none() {
            return Ok(None);
        }
        let discovery = discovery.unwrap();
        let discovery = bson::from_bson::<ClusterDiscovery>(bson::Bson::Document(discovery))
            .chain_err(|| FAIL_FIND_CLUSTER_DISCOVERY)?;
        Ok(Some(discovery))
    }

    pub fn cluster_meta(&self, cluster: String) -> Result<Option<ClusterMeta>> {
        let filter = doc!{"name" => cluster};
        MONGODB_OPS_COUNT.with_label_values(&["findOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["findOne"]).start_timer();
        let collection = self.collection_cluster_meta();
        let meta = collection.find_one(Some(filter), None)
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["findOne"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_CLUSTER_META)?;
        timer.observe_duration();
        if meta.is_none() {
            return Ok(None);
        }
        let meta = meta.unwrap();
        let meta = bson::from_bson::<ClusterMeta>(bson::Bson::Document(meta))
            .chain_err(|| FAIL_FIND_CLUSTER_META)?;
        Ok(Some(meta))
    }

    pub fn find_clusters(&self, search: &str, limit: u8) -> Result<Vec<ClusterMeta>> {
        let search = regex::escape(&search);
        let filter = doc!{"name" => {"$regex" => search, "$options" => "i"}};
        let mut options = FindOptions::new();
        options.limit = Some(i64::from(limit));

        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let collection = self.collection_cluster_meta();
        let cursor = collection.find(Some(filter), Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .chain_err(|| FAIL_FIND_CLUSTERS)?;

        let mut clusters = Vec::new();
        for doc in cursor {
            let doc = doc.chain_err(|| FAIL_FIND_CLUSTERS)?;
            let cluster = bson::from_bson::<ClusterMeta>(bson::Bson::Document(doc))
                .chain_err(|| FAIL_FIND_CLUSTERS)?;
            clusters.push(cluster);
        }
        Ok(clusters)
    }

    pub fn top_clusters(&self) -> Result<Vec<ClusterMeta>> {
        let sort = doc!{
            "nodes" => -1,
            "name" => 1,
        };
        let mut options = FindOptions::new();
        options.limit = Some(i64::from(TOP_CLUSTERS_LIMIT));
        options.sort = Some(sort);
        let collection = self.collection_cluster_meta();
        MONGODB_OPS_COUNT.with_label_values(&["find"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["find"]).start_timer();
        let cursor = collection.find(None, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["find"]).inc();
                error
            })
            .chain_err(|| FAIL_TOP_CLUSTERS)?;

        let mut clusters = Vec::new();
        for doc in cursor {
            let doc = doc.chain_err(|| FAIL_TOP_CLUSTERS)?;
            let cluster = bson::from_bson::<ClusterMeta>(bson::Bson::Document(doc))
                .chain_err(|| FAIL_TOP_CLUSTERS)?;
            clusters.push(cluster);
        }
        Ok(clusters)
    }

    pub fn persist_cluster_meta(&self, meta: ClusterMeta) -> Result<()> {
        let replacement = bson::to_bson(&meta).chain_err(|| FAIL_PERSIST_CLUSTER_META)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("ClusterMeta failed to encode as BSON document")
        };
        let filter = doc!{"name" => meta.name};
        let collection = self.collection_cluster_meta();
        let mut options = UpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["replaceOne"]).inc();
        let _timer = MONGODB_OPS_DURATION.with_label_values(&["replaceOne"]).start_timer();
        collection.replace_one(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["replaceOne"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_CLUSTER_META)?;
        Ok(())
    }

    pub fn persist_discovery(&self, cluster: ClusterDiscovery) -> Result<()> {
        let replacement = bson::to_bson(&cluster).chain_err(|| FAIL_PERSIST_CLUSTER_DISCOVERY)?;
        let replacement = match replacement {
            Bson::Document(replacement) => replacement,
            _ => panic!("ClusterDiscovery failed to encode as BSON document")
        };
        let filter = doc!{"cluster" => cluster.cluster.clone()};
        let collection = self.collection_discoveries();
        let mut options = UpdateOptions::new();
        options.upsert = Some(true);
        MONGODB_OPS_COUNT.with_label_values(&["replaceOne"]).inc();
        let timer = MONGODB_OPS_DURATION.with_label_values(&["replaceOne"]).start_timer();
        let result = collection.replace_one(filter, replacement, Some(options))
            .map_err(|error| {
                MONGODB_OP_ERRORS_COUNT.with_label_values(&["replaceOne"]).inc();
                error
            })
            .chain_err(|| FAIL_PERSIST_CLUSTER_DISCOVERY)?;
        timer.observe_duration();
        debug!(
            self.logger, "Persisted cluster discovery";
            "cluster" => cluster.cluster,
            "matched_count" => result.matched_count,
            "modified_count" => result.modified_count,
            "upserted_id" => ?result.upserted_id,
        );
        Ok(())
    }

    /// Returns the `clusters_meta` collection.
    fn collection_cluster_meta(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_CLUSTER_META)
    }

    /// Returns the `discoveries` collection.
    fn collection_discoveries(&self) -> Collection {
        self.client.db(&self.db).collection(COLLECTION_DISCOVERIES)
    }
}
