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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestFilter;

    impl Filter for TestFilter {
        fn on_request(&self, ctx: &mut RequestContext) -> FilterResult {
            ctx.headers.insert("x-test".into(), "req".into());
            FilterResult::Continue
        }

        fn on_response(&self, ctx: &mut ResponseContext) -> FilterResult {
            ctx.headers.insert("x-test".into(), "resp".into());
            FilterResult::Continue
        }
    }

    struct TestRouter;

    impl Router for TestRouter {
        fn choose_upstream(&self, ctx: &RequestContext) -> Option<RouteDecision> {
            if ctx.path.starts_with("/svc") {
                Some(RouteDecision { upstream_id: "u1".into() })
            } else {
                None
            }
        }
    }

    struct AlwaysHealthy;

    impl HealthChecker for AlwaysHealthy {
        fn check(&self, _upstream: &Upstream) -> HealthStatus {
            HealthStatus::Healthy
        }
    }

    struct TestMetrics {
        counters: Arc<Mutex<Vec<(String, u64)>>>,
        histograms: Arc<Mutex<Vec<(String, f64)>>>,
    }

    impl TestMetrics {
        fn new() -> Self {
            Self { counters: Arc::new(Mutex::new(vec![])), histograms: Arc::new(Mutex::new(vec![])) }
        }
    }

    impl Metrics for TestMetrics {
        fn incr_counter(&self, name: &str, value: u64) {
            self.counters.lock().unwrap().push((name.to_string(), value));
        }
        fn observe_histogram(&self, name: &str, value: f64) {
            self.histograms.lock().unwrap().push((name.to_string(), value));
        }
    }

    #[test]
    fn filter_should_mutate_contexts() {
        let f = TestFilter;
        let mut req = RequestContext { path: "/".into(), headers: Default::default() };
        let mut resp = ResponseContext { status: 200, headers: Default::default() };
        assert_eq!(f.on_request(&mut req), FilterResult::Continue);
        assert_eq!(f.on_response(&mut resp), FilterResult::Continue);
        assert_eq!(req.headers.get("x-test").unwrap(), "req");
        assert_eq!(resp.headers.get("x-test").unwrap(), "resp");
    }

    #[test]
    fn router_should_route_when_path_matches() {
        let r = TestRouter;
        let req = RequestContext { path: "/svc/a".into(), headers: Default::default() };
        let miss = RequestContext { path: "/healthz".into(), headers: Default::default() };
        let d = r.choose_upstream(&req).unwrap();
        assert_eq!(d.upstream_id, "u1");
        assert!(r.choose_upstream(&miss).is_none());
    }

    #[test]
    fn health_checker_reports_healthy() {
        let hc = AlwaysHealthy;
        let upstream = Upstream { id: "u1".into(), address: "127.0.0.1:80".parse().unwrap(), weight: 1, healthy: true };
        assert_eq!(hc.check(&upstream), HealthStatus::Healthy);
    }

    #[test]
    fn metrics_collects_values() {
        let m = TestMetrics::new();
        m.incr_counter("requests", 1);
        m.observe_histogram("latency_ms", 12.3);
        assert_eq!(m.counters.lock().unwrap().len(), 1);
        assert_eq!(m.histograms.lock().unwrap().len(), 1);
    }
}

