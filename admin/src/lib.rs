pub struct AdminApi;

impl AdminApi {
    pub fn new() -> Self {
        Self
    }

    pub fn health(&self) -> &'static str {
        "ok"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_is_ok() {
        let api = AdminApi::new();
        assert_eq!(api.health(), "ok");
    }
}

