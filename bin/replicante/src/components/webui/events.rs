//! Module to define events related WebUI endpoints.
use failure::ResultExt;

use iron::status;
use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;

use replicante_store_primary::store::legacy::EventsFilters;
use replicante_store_primary::store::legacy::EventsOptions;
use replicante_store_primary::store::Store;

use super::super::super::interfaces::api::APIRoot;
use super::super::super::interfaces::Interfaces;
use super::super::super::Error;
use super::super::super::ErrorKind;

const RECENT_EVENTS_LIMIT: i64 = 100;

/// Cluster discovery (`/webui/events`) handler.
pub struct Events {
    store: Store,
}

impl Handler for Events {
    fn handle(&self, _req: &mut Request) -> IronResult<Response> {
        let mut options = EventsOptions::default();
        options.limit = Some(RECENT_EVENTS_LIMIT);
        options.reverse = true;
        let iter = self
            .store
            .legacy()
            .events(EventsFilters::all(), options, None)
            .with_context(|_| ErrorKind::PrimaryStoreQuery("events"))
            .map_err(Error::from)?;
        let mut events = Vec::new();
        for event in iter {
            let event = event
                .with_context(|_| ErrorKind::Deserialize("event record", "Event"))
                .map_err(Error::from)?;
            events.push(event);
        }

        let mut resp = Response::new();
        resp.set_mut(JsonResponse::json(events)).set_mut(status::Ok);
        Ok(resp)
    }
}

impl Events {
    pub fn attach(interfaces: &mut Interfaces) {
        let mut router = interfaces.api.router_for(&APIRoot::UnstableWebUI);
        let handler = Events {
            store: interfaces.store.clone(),
        };
        router.get("/events", handler, "/events");
    }
}