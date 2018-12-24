use std::sync::Arc;

use slog::Logger;

use super::super::super::NodeId;
use super::super::super::Result;
use super::super::super::config::ZookeeperConfig;
use super::super::super::coordinator::NonBlockingLock;
use super::super::Backend;
use super::client::Client;

mod cleaner;
mod lock;

use self::cleaner::Cleaner;


/// Zookeeper-backed distributed coordination.
pub struct Zookeeper {
    // Background thread to clean unused nodes.
    _cleaner: Cleaner,
    client: Arc<Client>,
    logger: Logger,
    node_id: NodeId,
}

impl Zookeeper {
    pub fn new(node_id: NodeId, config: ZookeeperConfig, logger: Logger) -> Result<Zookeeper> {
        let client = Arc::new(Client::new(config.clone(), Some(&node_id), logger.clone())?);
        let cleaner = Cleaner::new(Arc::clone(&client), config, logger.clone())?;
        Ok(Zookeeper {
            _cleaner: cleaner,
            client,
            logger,
            node_id,
        })
    }
}

impl Backend for Zookeeper {
    fn non_blocking_lock(&self, lock: String) -> NonBlockingLock {
        NonBlockingLock::new(Box::new(self::lock::ZookeeperNBLock::new(
            Arc::clone(&self.client), lock, self.node_id.clone(), self.logger.clone()
        )))
    }

    fn node_id(&self) -> &NodeId {
        &self.node_id
    }
}