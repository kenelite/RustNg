use rustng_core::Upstream;

#[derive(Debug, Clone)]
pub enum SdEvent {
    UpstreamAdded(Upstream),
    UpstreamRemoved(String),
}

pub trait ServiceDiscovery: Send + Sync {
    fn start(&self);
}

pub struct NoopSd;

impl ServiceDiscovery for NoopSd {
    fn start(&self) {}
}

