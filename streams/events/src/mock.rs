use std::sync::Arc;

use replicante_data_models::Event;

use super::ErrorKind;
use super::EventsStream;
use super::Iter;
use super::Result;
use super::ScanFilters;
use super::ScanOptions;
use super::interface::StreamInterface;


/// Mock implementation of the events streaming interface.
#[derive(Default)]
pub struct MockEvents {}

impl MockEvents {
    pub fn new() -> MockEvents {
        MockEvents {}
    }

    pub fn mock(mock: Arc<MockEvents>) -> EventsStream {
        EventsStream(mock)
    }
}

impl StreamInterface for MockEvents {
    fn emit(&self, _event: Event) -> Result<()> {
        Err(ErrorKind::MockNotYetImplemented("emit").into())
    }

    fn scan(&self, _filters: ScanFilters, _options: ScanOptions) -> Result<Iter> {
        Err(ErrorKind::MockNotYetImplemented("scan").into())
    }
}
