use rustng_core::{Filter, FilterResult, RequestContext, ResponseContext};

pub trait PluginFactory: Send + Sync {
    fn create(&self) -> Box<dyn Filter>;
}

pub struct NoopFilter;

impl Filter for NoopFilter {
    fn on_request(&self, _ctx: &mut RequestContext) -> FilterResult {
        FilterResult::Continue
    }

    fn on_response(&self, _ctx: &mut ResponseContext) -> FilterResult {
        FilterResult::Continue
    }
}

