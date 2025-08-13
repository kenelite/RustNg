pub mod proxy {
    use rustng_core::{Filter, RequestContext, ResponseContext};

    #[derive(Default)]
    pub struct HttpProxy;

    impl HttpProxy {
        pub fn new() -> Self {
            Self
        }

        pub fn apply_filter<F: Filter>(&self, filter: &F, req: &mut RequestContext, resp: &mut ResponseContext) {
            let _ = filter.on_request(req);
            let _ = filter.on_response(resp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::proxy::HttpProxy;
    use rustng_core::{Filter, FilterResult, RequestContext, ResponseContext};

    struct F;
    impl Filter for F {
        fn on_request(&self, ctx: &mut RequestContext) -> FilterResult {
            ctx.headers.insert("x".into(), "1".into());
            FilterResult::Continue
        }
        fn on_response(&self, ctx: &mut ResponseContext) -> FilterResult {
            ctx.headers.insert("y".into(), "2".into());
            FilterResult::Continue
        }
    }

    #[test]
    fn apply_filter_mutates() {
        let p = HttpProxy::new();
        let f = F;
        let mut req = RequestContext::default();
        let mut resp = ResponseContext::default();
        p.apply_filter(&f, &mut req, &mut resp);
        assert_eq!(req.headers.get("x").unwrap(), "1");
        assert_eq!(resp.headers.get("y").unwrap(), "2");
    }
}

