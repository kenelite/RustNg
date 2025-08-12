use rustng_core::Upstream;

#[derive(Debug, Clone)]
pub enum ConfigEvent {
    RouteAdded { route: String },
    RouteRemoved { route: String },
    UpstreamUpdated { upstream: Upstream },
    PluginReload,
}

pub trait ConfigSubscriber: Send + Sync {
    fn on_event(&self, event: &ConfigEvent);
}

pub struct ConfigManager {
    subscribers: Vec<Box<dyn ConfigSubscriber>>,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self { subscribers: Vec::new() }
    }
}

impl ConfigManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&mut self, sub: Box<dyn ConfigSubscriber>) {
        self.subscribers.push(sub);
    }

    pub fn emit(&self, event: ConfigEvent) {
        for s in &self.subscribers {
            s.on_event(&event);
        }
    }
}

