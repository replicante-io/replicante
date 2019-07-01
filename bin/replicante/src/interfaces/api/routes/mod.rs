//! Module that defines a set of core routes for the API interface.
use prometheus::Registry;

use replicante_service_coordinator::Coordinator;
use replicante_util_iron::Router;

use super::super::healthchecks::HealthResultsCache;
use super::APIRoot;

mod index;
mod introspect;

/// Mount core API route handlers.
pub fn mount(
    router: &mut Router,
    coordinator: Coordinator,
    registry: Registry,
    healthchecks: HealthResultsCache,
) {
    // Create the index root for each API root.
    let roots = [APIRoot::UnstableAPI];
    for root in roots.iter() {
        let mut root = router.for_root(root);
        root.get("/", index::handler, "index");
    }
    self::introspect::mount(router, coordinator, registry, healthchecks);
}