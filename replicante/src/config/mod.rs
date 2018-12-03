use std::fs::File;
use std::io::Read;
use std::path::Path;

use failure::ResultExt;
use failure::err_msg;
use serde_yaml;

use replicante_coordinator::Config as CoordinatorConfig;
use replicante_data_store::Config as StorageConfig;
use replicante_logging::Config as LoggingConfig;
use replicante_logging::LoggingLevel;
use replicante_tasks::Config as TasksConfig;
use replicante_util_tracing::Config as TracingConfig;

use super::ErrorKind;
use super::Result;

use super::components::DiscoveryConfig;
use super::interfaces::api::Config as APIConfig;


mod components;
mod events;
mod task_workers;
mod timeouts;

pub use self::components::ComponentsConfig;
pub use self::events::EventsConfig;
pub use self::events::SnapshotsConfig as EventsSnapshotsConfig;
pub use self::task_workers::TaskWorkers;
pub use self::timeouts::TimeoutsConfig;


/// Replicante configuration options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// API server configuration.
    #[serde(default)]
    pub api: APIConfig,

    /// Components enabling configuration.
    #[serde(default)]
    pub components: ComponentsConfig,

    /// Distributed coordinator configuration options.
    #[serde(default)]
    pub coordinator: CoordinatorConfig,

    /// Agent discovery configuration.
    #[serde(default)]
    pub discovery: DiscoveryConfig,

    /// Events configuration.
    #[serde(default)]
    pub events: EventsConfig,

    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Storage layer configuration.
    #[serde(default)]
    pub storage: StorageConfig,

    /// Task workers enabling configuration.
    #[serde(default)]
    pub task_workers: TaskWorkers,

    /// Tasks system configuration.
    #[serde(default)]
    pub tasks: TasksConfig,

    /// Timeouts configured here are used throughout the system for various reasons.
    #[serde(default)]
    pub timeouts: TimeoutsConfig,

    /// Distributed tracing configuration.
    #[serde(default)]
    pub tracing: TracingConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            api: APIConfig::default(),
            components: ComponentsConfig::default(),
            coordinator: CoordinatorConfig::default(),
            discovery: DiscoveryConfig::default(),
            events: EventsConfig::default(),
            logging: LoggingConfig::default(),
            storage: StorageConfig::default(),
            task_workers: TaskWorkers::default(),
            tasks: TasksConfig::default(),
            timeouts: TimeoutsConfig::default(),
            tracing: TracingConfig::default(),
        }
    }
}

impl Config {
    /// Loads the configuration from the given [`std::fs::File`].
    ///
    /// [`std::fs::File`]: https://doc.rust-lang.org/std/fs/struct.File.html
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let config = File::open(path)
            .context(ErrorKind::Legacy(err_msg("failed to open config file")))?;
        Config::from_reader(config)
    }

    /// Loads the configuration from the given [`std::io::Read`].
    ///
    /// [`std::io::Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
    pub fn from_reader<R: Read>(reader: R) -> Result<Config> {
        let conf = serde_yaml::from_reader(reader)
            .context(ErrorKind::Legacy(err_msg("failed to decode config file")))?;
        Ok(conf)
    }

    /// Apply transformations to the configuration to derive some parameters.
    ///
    /// Transvormation:
    ///
    ///   * Apply verbose debug level logic.
    pub fn transform(mut self) -> Self {
        if self.logging.level == LoggingLevel::Debug && !self.logging.verbose {
            self.logging.level = LoggingLevel::Info;
            self.logging.modules.entry("replicante".into()).or_insert(LoggingLevel::Debug);
        }
        self
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::Config;

    #[test]
    #[should_panic(expected = "invalid type: string")]
    fn from_reader_error() {
        let cursor = Cursor::new("some other text");
        Config::from_reader(cursor).unwrap();
    }

    #[test]
    fn from_reader_ok() {
        let cursor = Cursor::new("{}");
        Config::from_reader(cursor).unwrap();
    }
}
