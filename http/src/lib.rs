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

