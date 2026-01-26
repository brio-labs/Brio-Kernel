use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct EventBus {
    // topic -> set of subscriber plugin_ids
    subscriptions: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self, topic: String, plugin_id: String) {
        let mut subs = self.subscriptions.write().expect("RwLock poisoned");
        subs.entry(topic).or_default().insert(plugin_id);
    }

    pub fn subscribers(&self, topic: &str) -> Vec<String> {
        let subs = self.subscriptions.read().expect("RwLock poisoned");
        subs.get(topic)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }
}
