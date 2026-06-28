use std::collections::HashSet;
use crate::memory::MemoryRecord;

pub struct ContextBuilder {
    max_tokens: usize,
    min_importance: i32,
    recency_weight: f64,
    importance_weight: f64,
}

#[derive(Debug)]
pub struct Context {
    pub user_query: String,
    pub project: String,
    pub recents: Vec<String>,
    pub high_importance: Vec<String>,
    pub namespace_groups: Vec<(String, Vec<String>)>,
    pub used_count: usize,
    pub total_count: usize,
    pub token_usage: usize,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            max_tokens: 4096,
            min_importance: 3,
            recency_weight: 0.3,
            importance_weight: 0.7,
        }
    }

    pub fn with_max_tokens(mut self, n: usize) -> Self {
        self.max_tokens = n;
        self
    }

    pub fn build_context(&self, query: &str, mut memories: Vec<MemoryRecord>) -> Context {
        let total = memories.len();

        memories = self.dedup(memories);
        memories = self.score_and_rank(query, memories);
        memories = self.topk_within_budget(memories);

        let used = memories.len();
        let mut token_usage = 0usize;

        let mut high_importance = Vec::new();
        let mut recents = Vec::new();
        let mut namespace_groups: Vec<(String, Vec<String>)> = Vec::new();

        for mem in &memories {
            let content = self.compress(&mem.content);
            let estimated = content.len() / 2;
            if token_usage + estimated > self.max_tokens {
                break;
            }
            token_usage += estimated;

            if mem.importance >= 8 {
                high_importance.push(format!("[{}] {}", mem.importance, content));
            }

            if mem.created_at > chrono::Utc::now().timestamp_millis() - 86400000 {
                recents.push(content.clone());
            }

            let ns = &mem.namespace;
            let group_key = if ns.starts_with("project/") {
                ns.to_string()
            } else if ns == "shared" {
                "Shared".to_string()
            } else {
                ns.to_string()
            };

            if let Some((_, entries)) = namespace_groups.iter_mut().find(|(k, _)| k == &group_key) {
                entries.push(content);
            } else {
                namespace_groups.push((group_key, vec![content]));
            }
        }

        let project = namespace_groups.iter()
            .find(|(k, _)| k.starts_with("project/"))
            .map(|(k, _)| k.trim_start_matches("project/").trim_end_matches("/*").to_string())
            .unwrap_or_default();

        Context {
            user_query: query.to_string(),
            project,
            recents,
            high_importance,
            namespace_groups,
            used_count: used,
            total_count: total,
            token_usage,
        }
    }

    fn dedup(&self, memories: Vec<MemoryRecord>) -> Vec<MemoryRecord> {
        let mut seen = HashSet::new();
        memories.into_iter()
            .filter(|m| {
                let key = self.content_hash(&m.content);
                seen.insert(key)
            })
            .collect()
    }

    fn score_and_rank(&self, query: &str, mut memories: Vec<MemoryRecord>) -> Vec<MemoryRecord> {
        let now = chrono::Utc::now().timestamp_millis();
        let query_lower = query.to_lowercase();

        for mem in &mut memories {
            let relevance = if query_lower.is_empty() {
                0.5
            } else {
                let content_lower = mem.content.to_lowercase();
                let query_words: Vec<&str> = query_lower.split_whitespace().collect();
                let matches = query_words.iter()
                    .filter(|w| content_lower.contains(*w))
                    .count();
                if query_words.is_empty() {
                    0.0
                } else {
                    matches as f64 / query_words.len() as f64
                }
            };

            let age_hours = (now - mem.created_at) as f64 / 3600000.0;
            let recency = (-age_hours / 72.0).exp();

            let imp_factor = mem.importance as f64 / 10.0;

            let score = relevance * 0.4 + recency * self.recency_weight + imp_factor * self.importance_weight;
            mem.importance = (score * 10.0) as i32;
        }

        memories.sort_by(|a, b| b.importance.cmp(&a.importance));
        memories
    }

    fn topk_within_budget(&self, memories: Vec<MemoryRecord>) -> Vec<MemoryRecord> {
        let mut result = Vec::new();
        let mut token_usage = 0usize;

        for mem in memories {
            let estimated = mem.content.len() / 2;
            if token_usage + estimated > self.max_tokens {
                break;
            }
            token_usage += estimated;
            result.push(mem);
        }
        result
    }

    fn content_hash(&self, content: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn compress(&self, content: &str) -> String {
        if content.len() <= 300 {
            return content.to_string();
        }
        let sentences: Vec<&str> = content
            .split(|c| c == '.' || c == '!' || c == '?' || c == '\n')
            .collect();

        if sentences.len() <= 3 {
            return format!("{}...", &content[..300]);
        }

        let compressed: Vec<&str> = sentences.iter()
            .take(5)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        compressed.join(". ") + "."
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
