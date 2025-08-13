use std::collections::HashMap;
use std::net::SocketAddr;

pub mod prelude {
    pub use crate::{Filter, FilterResult, RequestContext, ResponseContext, Router};
}

#[derive(Debug, Default, Clone)]
pub struct RequestContext {
    pub path: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Default, Clone)]
pub struct ResponseContext {
    pub status: u16,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterResult {
    Continue,
    Stop,
}

impl Default for FilterResult {
    fn default() -> Self {
        FilterResult::Continue
    }
}

pub trait Filter: Send + Sync {
    fn on_request(&self, _ctx: &mut RequestContext) -> FilterResult {
        FilterResult::Continue
    }
    fn on_response(&self, _ctx: &mut ResponseContext) -> FilterResult {
        FilterResult::Continue
    }
}

#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub upstream_id: String,
}

pub trait Router: Send + Sync {
    fn choose_upstream(&self, ctx: &RequestContext) -> Option<RouteDecision>;
}

#[derive(Debug, Clone)]
pub struct Upstream {
    pub id: String,
    pub address: SocketAddr,
    pub weight: u32,
    pub healthy: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

pub trait HealthChecker: Send + Sync {
    fn check(&self, upstream: &Upstream) -> HealthStatus;
}

pub trait Metrics: Send + Sync {
    fn incr_counter(&self, name: &str, value: u64);
    fn observe_histogram(&self, name: &str, value: f64);
}

