use std::time::Duration;

use failure::ResultExt;
use prometheus::Registry;
use slog::Logger;

use replicante_service_tasks::TaskHandler;
use replicante_service_tasks::TaskQueue;
use replicante_service_tasks::WorkerSet;
use replicante_service_tasks::WorkerSetPool;
use replicante_util_upkeep::Upkeep;

use super::super::config::Config;
use super::super::interfaces::Interfaces;
use super::super::metrics::WORKERS_ENABLED;

use super::super::tasks::cluster_refresh;
use super::super::tasks::ReplicanteQueues;

use super::super::Error;
use super::super::ErrorKind;
use super::super::Result;

/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    self::cluster_refresh::register_metrics(logger, registry);
}

/// Store the state of the WorkerSet.
///
/// Initially the pool is configured but not active.
/// When the component is run, the pool configuration is taken and a handle to
/// the workers pool is stored in its place.
enum State {
    Configured(WorkerSet<ReplicanteQueues>),
    Started(WorkerSetPool),
}

/// Helper function to keep `Workers::new` simpler in the presence of conditional queues.
fn configure_worker<F, H>(
    workers: WorkerSet<ReplicanteQueues>,
    queue: ReplicanteQueues,
    enabled: bool,
    factory: F,
) -> Result<WorkerSet<ReplicanteQueues>>
where
    F: Fn() -> H,
    H: TaskHandler<ReplicanteQueues>,
{
    let name = queue.name();
    if enabled {
        WORKERS_ENABLED.with_label_values(&[&name]).set(1.0);
        let handler = factory();
        let workers = match workers.worker(queue, handler) {
            Ok(workers) => workers,
            Err(error) => {
                return Err(error)
                    .with_context(|_| ErrorKind::TaskWorkerRegistration(name))
                    .map_err(Error::from)
            }
        };
        return Ok(workers);
    }
    WORKERS_ENABLED.with_label_values(&[&name]).set(0.0);
    Ok(workers)
}

/// Wrapper object around `replicante_tasks::WorkerSet` objects.
pub struct Workers {
    state: Option<State>,
}

impl Workers {
    /// Configure the task workers to be run later.
    ///
    /// The tasks that are processed by this node are defined in the configuration file.
    pub fn new(interfaces: &mut Interfaces, logger: Logger, config: Config) -> Result<Workers> {
        let agents_timeout = Duration::from_secs(config.timeouts.agents_api);
        let worker_set = WorkerSet::new(
            logger.clone(),
            config.tasks,
            interfaces.healthchecks.register(),
        )
        .with_context(|_| ErrorKind::ClientInit("tasks workers"))?;
        let worker_set = configure_worker(
            worker_set,
            ReplicanteQueues::ClusterRefresh,
            config.task_workers.cluster_refresh(),
            || self::cluster_refresh::Handler::new(interfaces, logger.clone(), agents_timeout),
        )?;
        Ok(Workers {
            state: Some(State::Configured(worker_set)),
        })
    }

    /// Convert the WorkerSet configuration into a runnning WorkerSetPool.
    pub fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        if let Some(State::Configured(worker_set)) = self.state.take() {
            let workers = worker_set
                .run(upkeep)
                .with_context(|_| ErrorKind::ThreadSpawn("tasks workers"))?;
            self.state = Some(State::Started(workers));
            Ok(())
        } else {
            Err(ErrorKind::ComponentAlreadyRunning("workers").into())
        }
    }
}
