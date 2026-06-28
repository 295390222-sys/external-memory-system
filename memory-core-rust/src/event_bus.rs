use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type Callback = Arc<dyn Fn(Event) + Send + Sync>;

#[derive(Clone, Debug)]
pub enum Event {
    MessageReceived {
        agent_id: String,
        user_message: String,
        namespace: String,
    },
    MemorySaved {
        agent_id: String,
        memory_id: String,
        namespace: String,
        importance: i32,
    },
    DreamFinished {
        agent_id: String,
        facts_count: usize,
        inferences_count: usize,
    },
    SummaryFinished {
        agent_id: String,
        summary: String,
        key_points: Vec<String>,
    },
    ForgettingFinished {
        expired_removed: usize,
        low_weight_removed: usize,
    },
}

pub struct EventBus {
    listeners: HashMap<String, Vec<Callback>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { listeners: HashMap::new() }
    }

    pub fn on<F>(&mut self, event_type: &str, callback: F)
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        self.listeners
            .entry(event_type.to_string())
            .or_default()
            .push(Arc::new(callback));
    }

    pub fn emit(&self, event: Event) {
        let type_key = match &event {
            Event::MessageReceived { .. } => "message_received",
            Event::MemorySaved { .. } => "memory_saved",
            Event::DreamFinished { .. } => "dream_finished",
            Event::SummaryFinished { .. } => "summary_finished",
            Event::ForgettingFinished { .. } => "forgetting_finished",
        };

        if let Some(callbacks) = self.listeners.get(type_key) {
            for cb in callbacks {
                cb(event.clone());
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
