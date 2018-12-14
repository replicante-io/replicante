use std::sync::Arc;
use std::thread::Builder;
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::RecvTimeoutError;
use crossbeam_channel::Sender;
use crossbeam_channel::bounded;

use failure::ResultExt;
use rand::Rng;
use rand::thread_rng;
use slog::Logger;
use zookeeper::ZkError;

use replicante_util_failure::failure_info;

use super::super::super::super::ErrorKind;
use super::super::super::super::Result;
use super::super::super::super::config::ZookeeperConfig;
use super::super::metrics::ZOO_CLEANUP_COUNT;
use super::super::metrics::ZOO_OP_DURATION;
use super::super::metrics::ZOO_OP_ERRORS_COUNT;
use super::super::metrics::ZOO_TIMEOUTS_COUNT;
use super::Client;


/// Background thread to cleanup unused nodes.
///
/// Prevent the prefix nodes that do not contain anything from piling up without value.
/// Once the new container znode type is stable this code can be dropped in favour of that.
pub struct Cleaner {
    handle: Option<JoinHandle<()>>,
    logger: Logger,
    shutdown_signal: Option<Sender<()>>,
}

impl Cleaner {
    pub fn new(client: Arc<Client>, config: ZookeeperConfig, logger: Logger) -> Result<Cleaner> {
        let (sender, receiver) = bounded(0);
        let inner_logger = logger.clone();
        let handle = Builder::new().name("r:coordinator:zoo:cleaner".into()).spawn(move || {
            let logger = inner_logger;
            let cleaner = InnerCleaner {
                client,
                config,
                logger: logger.clone(),
            };
            loop {
                info!(logger, "Running zookeeper cleanup cycle");
                if let Err(error) = cleaner.cycle() {
                    error!(logger, "Zookeeper cleanup cycle failed"; failure_info(&error));
                }
                debug!(logger, "Zookeeper cleanup cycle ended");

                // Wait for the quiet period to be over or exit when signaled.
                let timeout = cleaner.interval();
                debug!(logger, "Zookeeper cleanup cycle sleeping"; "timeout" => ?timeout);
                match receiver.recv_timeout(timeout) {
                    Ok(()) => return,
                    Err(RecvTimeoutError::Disconnected) => return,
                    Err(RecvTimeoutError::Timeout) => (),
                };
            }
        }).context(ErrorKind::SpawnThread("zookeeper cleaner"))?;
        Ok(Cleaner {
            handle: Some(handle),
            logger,
            shutdown_signal: Some(sender),
        })
    }
}

impl Drop for Cleaner {
    fn drop(&mut self) {
        if let Some(shutdown_signal) = self.shutdown_signal.take() {
            drop(shutdown_signal);
        }
        if let Some(handle) = self.handle.take() {
            if let Err(error) = handle.join() {
                error!(self.logger, "Zookeeper cleaner thread paniced"; "error" => ?error);
            }
        }
    }
}


/// Helper class to collect worker thread context.
struct InnerCleaner {
    client: Arc<Client>,
    config: ZookeeperConfig,
    logger: Logger,
}

impl InnerCleaner {
    /// Clean children of the given path.
    fn clean(&self, path: &str, limit: usize) -> Result<usize> {
        let client = self.client.get()?;
        let mut limit = limit;

        let timer = ZOO_OP_DURATION.with_label_values(&["get_children"]).start_timer();
        let children = client.get_children(path, false)
            .map_err(|error| {
                ZOO_OP_ERRORS_COUNT.with_label_values(&["get_children"]).inc();
                if error == ZkError::OperationTimeout {
                    ZOO_TIMEOUTS_COUNT.inc();
                }
                error
            })
            .context(ErrorKind::Backend("children lookup"))?;
        timer.observe_duration();

        for child in children {
            let child = format!("{}/{}", path, child);
            let timer = ZOO_OP_DURATION.with_label_values(&["exists"]).start_timer();
            let stats = match client.exists(&child, false) {
                Err(ZkError::NoNode) | Ok(None) => {
                    timer.observe_duration();
                    continue;
                },
                Err(error) => {
                    timer.observe_duration();
                    ZOO_OP_ERRORS_COUNT.with_label_values(&["exists"]).inc();
                    if error == ZkError::OperationTimeout {
                        ZOO_TIMEOUTS_COUNT.inc();
                    }
                    return Err(error).context(ErrorKind::Backend("node lookup"))?;
                },
                Ok(Some(stats)) => stats,
            };
            timer.observe_duration();

            // Look only at empty nodes.
            if stats.num_children != 0 {
                continue;
            }

            // Delete and count.
            let timer = ZOO_OP_DURATION.with_label_values(&["delete"]).start_timer();
            match client.delete(&child, Some(stats.version)) {
                Err(ZkError::NoNode) |
                    Err(ZkError::NotEmpty) |
                    Ok(()) => (),
                Err(error) => {
                    timer.observe_duration();
                    ZOO_OP_ERRORS_COUNT.with_label_values(&["delete"]).inc();
                    if error == ZkError::OperationTimeout {
                        ZOO_TIMEOUTS_COUNT.inc();
                    }
                    return Err(error).context(ErrorKind::Backend("node delete"))?;
                },
            };
            timer.observe_duration();
            ZOO_CLEANUP_COUNT.inc();
            limit = limit - 1;
            if limit == 0 {
                return Ok(0)
            }
        }

        Ok(limit)
    }

    /// Perform a single zookeeper cleanup cycle.
    fn cycle(&self) -> Result<()> {
        let limit = self.config.cleanup.limit;
        let limit = self.clean("/nodes", limit)?;
        if limit == 0 {
            info!(self.logger, "Reached limit of nodes to clean for one cycle");
            return Ok(());
        }
        Ok(())
    }

    /// Determine how long to wait before a new cleaner cycle should run.
    fn interval(&self) -> Duration {
        let mut rng = thread_rng();
        let timeout: u64 = rng.gen_range(
            self.config.cleanup.interval_min, self.config.cleanup.interval_max
        );
        Duration::from_secs(timeout)
    }
}