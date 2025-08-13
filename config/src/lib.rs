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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct CapturingSub(Arc<Mutex<Vec<String>>>);
    impl ConfigSubscriber for CapturingSub {
        fn on_event(&self, event: &ConfigEvent) {
            let s = match event {
                ConfigEvent::RouteAdded { route } => format!("add:{route}"),
                ConfigEvent::RouteRemoved { route } => format!("del:{route}"),
                ConfigEvent::UpstreamUpdated { upstream } => format!("up:{id}", id = upstream.id),
                ConfigEvent::PluginReload => "plugin".into(),
            };
            self.0.lock().unwrap().push(s);
        }
    }

    #[test]
    fn events_are_delivered_to_subscribers() {
        let sink = Arc::new(Mutex::new(vec![]));
        let mut mgr = ConfigManager::new();
        mgr.subscribe(Box::new(CapturingSub(sink.clone())));

        mgr.emit(ConfigEvent::RouteAdded { route: "/a".into() });
        mgr.emit(ConfigEvent::PluginReload);

        let got = sink.lock().unwrap().clone();
        assert_eq!(got, vec!["add:/a".to_string(), "plugin".to_string()]);
    }
}

