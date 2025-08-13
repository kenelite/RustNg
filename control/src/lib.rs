pub trait ControlPlane {
    fn apply_config(&self, payload: &[u8]) -> Result<(), String>;
}

pub struct NoopControlPlane;

impl ControlPlane for NoopControlPlane {
    fn apply_config(&self, _payload: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

