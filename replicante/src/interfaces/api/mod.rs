//! HTTP API interface to interact with replicante.
//!
//! This interface is a wrapper around the [`iron`] framework.
//! This module does not implement all of the APIs but rather provides
//! tools for other interfaces and components to add their own endpoints.
use std::collections::HashMap;
use std::time::Duration;

use failure::ResultExt;
use humthreads::Builder as ThreadBuilder;
use iron::Iron;
use iron_json_response::JsonResponseMiddleware;
use slog::Logger;

#[cfg(test)]
use replicante_coordinator::mock::MockCoordinator;
use replicante_coordinator::Coordinator;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_iron::MetricsMiddleware;
use replicante_util_iron::RequestLogger;
use replicante_util_iron::RootDescriptor;
use replicante_util_iron::RootedRouter;
use replicante_util_iron::Router;
use replicante_util_upkeep::Upkeep;

use super::super::ErrorKind;
use super::super::Result;
use super::metrics::Metrics;

mod config;
mod metrics;
mod routes;

pub use self::config::Config;
pub use self::metrics::register_metrics;

use self::metrics::MIDDLEWARE;

/// The replicante HTTP API interface.
pub struct API {
    config: Config,
    logger: Logger,

    // Below attributes are consumed or generated by `API::run`.
    metrics_middleware: Option<MetricsMiddleware>,
    router: Option<Router>,
}

impl API {
    /// Creates a new API interface.
    pub fn new(config: Config, coordinator: Coordinator, logger: Logger, metrics: &Metrics) -> API {
        let registry = metrics.registry().clone();
        let mut router = Router::new(config.trees.clone().into());
        routes::mount(&mut router, coordinator, registry);

        let middleware = MetricsMiddleware::new(
            MIDDLEWARE.0.clone(),
            MIDDLEWARE.1.clone(),
            MIDDLEWARE.2.clone(),
            logger.clone(),
        );

        API {
            config,
            logger,
            metrics_middleware: Some(middleware),
            router: Some(router),
        }
    }

    /// Register routes for a specific API version.
    pub fn router_for(&mut self, root: &APIRoot) -> RootedRouter {
        self.router
            .as_mut()
            .expect("Unable to access router. Was API::run called already?")
            .for_root(root)
    }

    /// Creates an Iron server and spawns a thread to serve it.
    pub fn run(&mut self, upkeep: &mut Upkeep) -> Result<()> {
        let config = self.config.clone();
        let logger = self.logger.clone();

        let mut chain = self.router.take().unwrap().build();
        chain.link_after(JsonResponseMiddleware::new());
        chain.link_after(RequestLogger::new(self.logger.clone()));
        chain.link(self.metrics_middleware.take().unwrap().into_middleware());

        let handle = ThreadBuilder::new("r:i:api")
            .full_name("replicore:interface:api")
            .spawn(move |scope| {
                let mut server = Iron::new(chain);
                server.timeouts.keep_alive = config.timeouts.keep_alive.map(Duration::from_secs);
                server.timeouts.read = config.timeouts.read.map(Duration::from_secs);
                server.timeouts.write = config.timeouts.write.map(Duration::from_secs);
                if let Some(threads_count) = config.threads_count {
                    server.threads = threads_count;
                }

                info!(logger, "Starting API server"; "bind" => &config.bind);
                scope.activity("running https://github.com/iron/iron HTTP server");
                let mut bind = server
                    .http(config.bind)
                    .expect("Unable to start API server");
                // Once started, the server will run in the background.
                // When the guard returned by Iron::http is dropped it tries to join the server.
                // To support shutting down wait for the signal here, then close the server.
                // NOTE: closing the server does not really work, just prevent the need to join :-(
                //   See https://github.com/hyperium/hyper/issues/338
                while !scope.should_shutdown() {
                    ::std::thread::sleep(Duration::from_secs(1));
                }
                if let Err(error) = bind.close() {
                    capture_fail!(
                        &error,
                        logger,
                        "Failed to shutdown API server";
                        failure_info(&error),
                    );
                }
            })
            .with_context(|_| ErrorKind::ThreadSpawn("http server"))?;
        upkeep.register_thread(handle);
        Ok(())
    }

    /// Returns an `API` instance usable as a mock.
    #[cfg(test)]
    pub fn mock(logger: Logger, metrics: &Metrics) -> (API, MockCoordinator) {
        let config = Config::default();
        let coordinator = MockCoordinator::new(logger.clone());
        let api = API::new(config, coordinator.mock(), logger, metrics);
        (api, coordinator)
    }
}

/// Enumerates all possible API roots.
///
/// All endpoints must fall under one of these roots and are subject to all restrictions
/// of that specific root.
/// The main restriction is that versioned APIs are subject to semver guarantees.
pub enum APIRoot {
    /// API root for all endpoints that are not yet stable.
    ///
    /// Endpoints in this root are NOT subject to ANY compatibility guarantees!
    UnstableAPI,

    /// Instrospection APIs not yet stable.
    UnstableIntrospect,

    /// Specialised endpoints for the WebUI project.
    UnstableWebUI,
}

impl RootDescriptor for APIRoot {
    fn enabled(&self, flags: &HashMap<&'static str, bool>) -> bool {
        match self {
            APIRoot::UnstableAPI | APIRoot::UnstableWebUI => match flags.get("unstable") {
                Some(flag) => *flag,
                None => true,
            },
            APIRoot::UnstableIntrospect => match flags.get("unstable") {
                Some(flag) if !flag => *flag,
                _ => match flags.get("introspect") {
                    Some(flag) => *flag,
                    None => true,
                },
            },
        }
    }

    fn prefix(&self) -> &'static str {
        match self {
            APIRoot::UnstableAPI => "/api/unstable",
            APIRoot::UnstableIntrospect => "/api/unstable/introspect",
            APIRoot::UnstableWebUI => "/api/unstable/webui",
        }
    }
}
