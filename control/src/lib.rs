pub trait ControlPlane {
    fn apply_config(&self, payload: &[u8]) -> Result<(), String>;
}

pub struct NoopControlPlane;

impl ControlPlane for NoopControlPlane {
    fn apply_config(&self, _payload: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_apply_config_ok() {
        let cp = NoopControlPlane;
        assert!(cp.apply_config(b"{}").is_ok());
    }
}

