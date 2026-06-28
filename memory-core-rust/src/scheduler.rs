use tokio::time::{interval, Duration};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::event_bus::{EventBus, Event};
use crate::storage::Storage;
use crate::dreaming_engine::DreamingEngine;
use crate::summarizer::Summarizer;
use crate::scoring_engine::ScoringEngine;

pub struct MemoryScheduler {
    dreaming_engine: Arc<RwLock<DreamingEngine>>,
    storage: Arc<RwLock<Storage>>,
    summarizer: Summarizer,
    event_bus: Arc<RwLock<EventBus>>,
}

impl MemoryScheduler {
    pub fn new(
        storage: Arc<RwLock<Storage>>,
        event_bus: Arc<RwLock<EventBus>>,
    ) -> Self {
        Self {
            dreaming_engine: Arc::new(RwLock::new(DreamingEngine::new())),
            storage,
            summarizer: Summarizer::new(),
            event_bus,
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            self.run_dreaming_loop().await;
        });
        tokio::spawn(async move {
            self.run_summary_loop().await;
        });
        tokio::spawn(async move {
            self.run_forgetting_loop().await;
        });
        log::info!("MemoryScheduler started: dreaming(daily), summary(hourly), forgetting(weekly)");
    }

    async fn run_dreaming_loop(&self) {
        let mut timer = interval(Duration::from_secs(86400));
        loop {
            timer.tick().await;

            let agents = vec!["main".to_string(), "claw".to_string(), "content".to_string()];
            for agent_id in &agents {
                let storage = self.storage.read().await;
                let recent = storage.query_recent(agent_id, "shared", 100).unwrap_or_default();
                drop(storage);

                if recent.is_empty() {
                    continue;
                }

                let mut engine = self.dreaming_engine.write().await;
                let result = engine.process(recent);

                let mut storage = self.storage.write().await;
                for dm in &result.facts {
                    let record = crate::memory::MemoryRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        agent_id: agent_id.clone(),
                        namespace: "shared".to_string(),
                        memory_type: 2,
                        importance: (dm.confidence * 10.0) as i32,
                        content: dm.content.clone(),
                        embedding: vec![],
                        entities: vec![],
                        created_at: chrono::Utc::now().timestamp_millis(),
                        updated_at: chrono::Utc::now().timestamp_millis(),
                        access_count: 0,
                        last_access: chrono::Utc::now().timestamp_millis(),
                        expire_at: None,
                    };
                    storage.insert(&record).ok();
                }

                log::info!("Dreaming completed for agent={}: {} facts, {} inferences, {} hypotheses",
                    agent_id, result.facts.len(), result.inferences.len(), result.hypotheses.len());

                let bus = self.event_bus.write().await;
                bus.emit(Event::DreamFinished {
                    agent_id: agent_id.clone(),
                    facts_count: result.facts.len(),
                    inferences_count: result.inferences.len(),
                });
            }
        }
    }

    async fn run_summary_loop(&self) {
        let mut timer = interval(Duration::from_secs(3600));
        loop {
            timer.tick().await;

            let agents = vec!["main".to_string(), "claw".to_string(), "content".to_string()];
            for agent_id in &agents {
                let storage = self.storage.read().await;
                let since = chrono::Utc::now().timestamp_millis() - 86400000;
                let until = chrono::Utc::now().timestamp_millis();
                let records = storage.query_by_time(agent_id, "shared", since, until).unwrap_or_default();

                if records.is_empty() {
                    continue;
                }

                let summary = self.summarizer.summarize(&records);
                let key_points = self.summarizer.extract_key_points(&records);

                let bus = self.event_bus.write().await;
                bus.emit(Event::SummaryFinished {
                    agent_id: agent_id.clone(),
                    summary,
                    key_points,
                });
            }
        }
    }

    async fn run_forgetting_loop(&self) {
        let mut timer = interval(Duration::from_secs(604800));
        loop {
            timer.tick().await;

            let storage = self.storage.read().await;
            let conn = &storage.conn;
            let now = chrono::Utc::now().timestamp_millis();

            let expired = conn.execute(
                "DELETE FROM memory_records WHERE expire_at IS NOT NULL AND expire_at < ?1",
                rusqlite::params![now],
            ).unwrap_or(0);

            let low_weight = conn.execute(
                "DELETE FROM memory_records WHERE id IN (
                    SELECT id FROM memory_records
                    WHERE importance * (CAST(access_count AS REAL) + 1.0) * exp(-(CAST(?1 - last_access AS REAL) / 3600000.0) / 720.0) < 1.0
                    AND memory_type != 'semantic'
                    LIMIT 200
                )",
                rusqlite::params![now],
            ).unwrap_or(0);

            log::info!("Forgetting: removed {} expired, {} low-weight memories", expired, low_weight);

            let bus = self.event_bus.write().await;
            bus.emit(Event::ForgettingFinished {
                expired_removed: expired as usize,
                low_weight_removed: low_weight as usize,
            });
        }
    }
}
