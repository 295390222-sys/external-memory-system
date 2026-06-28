use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::storage::Storage;
use crate::isolation_engine::IsolationEngine;

#[derive(Clone, Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub namespaces: Vec<String>,
    pub created_at: i64,
}

pub struct ProjectManager {
    storage: Arc<RwLock<Storage>>,
    isolation: Arc<RwLock<IsolationEngine>>,
    projects: HashMap<String, Project>,
}

impl ProjectManager {
    pub fn new(storage: Arc<RwLock<Storage>>, isolation: Arc<RwLock<IsolationEngine>>) -> Self {
        let mut pm = Self {
            storage,
            isolation,
            projects: HashMap::new(),
        };

        pm.register(Project {
            id: "ai-drama".to_string(),
            name: "AI短剧宇宙".to_string(),
            platform: "edgeone".to_string(),
            namespaces: vec!["project/ai-drama/*".to_string()],
            created_at: chrono::Utc::now().timestamp_millis(),
        });

        pm.register(Project {
            id: "qq-channel".to_string(),
            name: "QQ频道".to_string(),
            platform: "qq".to_string(),
            namespaces: vec!["project/qq-channel/*".to_string()],
            created_at: chrono::Utc::now().timestamp_millis(),
        });

        pm.register(Project {
            id: "digital-human".to_string(),
            name: "数字人".to_string(),
            platform: "edgeone".to_string(),
            namespaces: vec!["project/digital-human/*".to_string()],
            created_at: chrono::Utc::now().timestamp_millis(),
        });

        pm
    }

    pub fn register(&mut self, project: Project) {
        self.projects.insert(project.id.clone(), project);
    }

    pub fn get(&self, id: &str) -> Option<&Project> {
        self.projects.get(id)
    }

    pub fn list(&self) -> Vec<&Project> {
        self.projects.values().collect()
    }

    pub fn switch_project(&self, agent_id: &str, project_id: &str) -> anyhow::Result<()> {
        let project = self.projects.get(project_id)
            .ok_or_else(|| anyhow::anyhow!("project not found: {}", project_id))?;

        let iso = self.isolation.blocking_write();
        if !iso.can_write(agent_id, &project.namespaces[0], Some(project_id)) {
            anyhow::bail!("agent {} has no write access to project {}", agent_id, project_id);
        }

        log::info!("Agent {} switched to project {}", agent_id, project_id);
        Ok(())
    }

    pub fn leave_project(&self, agent_id: &str, project_id: &str) -> anyhow::Result<()> {
        log::info!("Agent {} left project {}", agent_id, project_id);
        Ok(())
    }

    pub async fn get_context(&self, project_id: &str, agent_id: &str) -> anyhow::Result<String> {
        let project = self.projects.get(project_id)
            .ok_or_else(|| anyhow::anyhow!("project not found: {}", project_id))?;

        let ns_pattern = &project.namespaces[0];
        let storage = self.storage.read().await;
        let records = storage.query_by_namespace(ns_pattern, 30)?;

        if records.is_empty() {
            return Ok(format!("No memories for project '{}'.", project.name));
        }

        let mut context = format!("=== Project: {} ({}) ===\n", project.name, project.platform);
        for r in &records {
            let preview = if r.content.len() > 200 {
                format!("{}...", &r.content[..200])
            } else {
                r.content.clone()
            };
            context.push_str(&format!("[{}] {}\n", r.importance, preview));
        }

        Ok(context)
    }
}
