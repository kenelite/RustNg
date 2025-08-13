pub struct AdminApi;

impl AdminApi {
    pub fn new() -> Self {
        Self
    }

    pub fn health(&self) -> &'static str {
        "ok"
    }
}

