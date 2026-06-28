use crate::memory::MemoryRecord;
use std::collections::HashMap;

pub struct KeywordEngine {
    index: HashMap<String, Vec<String>>,
}

impl KeywordEngine {
    pub fn new() -> Self {
        Self { index: HashMap::new() }
    }

    pub fn search(&self, query: &str, namespace: &str, limit: usize) -> anyhow::Result<Vec<MemoryRecord>> {
        let keywords: Vec<&str> = query
            .split_whitespace()
            .flat_map(|w| w.split(|c: char| !c.is_alphanumeric()))
            .filter(|w| w.len() > 1)
            .collect();

        if keywords.is_empty() {
            return Ok(vec![]);
        }

        Ok(vec![])
    }

    pub fn extract_keywords(&self, content: &str) -> Vec<String> {
        content
            .split_whitespace()
            .flat_map(|w| w.split(|c: char| !c.is_alphanumeric()))
            .filter(|w| w.len() > 1 && w.chars().all(|c| c.is_alphanumeric()))
            .map(|w| w.to_lowercase())
            .collect()
    }

    pub fn relevance_score(&self, content: &str, query: &str) -> f64 {
        let content_lower = content.to_lowercase();
        let query_keywords: Vec<&str> = query
            .split_whitespace()
            .flat_map(|w| w.split(|c: char| !c.is_alphanumeric()))
            .filter(|w| w.len() > 1)
            .collect();

        if query_keywords.is_empty() {
            return 0.0;
        }

        let matches = query_keywords.iter()
            .filter(|kw| content_lower.contains(&kw.to_lowercase()))
            .count();

        matches as f64 / query_keywords.len() as f64
    }
}

impl Default for KeywordEngine {
    fn default() -> Self {
        Self::new()
    }
}
