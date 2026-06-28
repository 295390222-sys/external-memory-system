use crate::memory::MemoryRecord;

pub struct Summarizer;

impl Summarizer {
    pub fn new() -> Self {
        Self
    }

    pub fn summarize(&self, records: &[MemoryRecord]) -> String {
        if records.is_empty() {
            return "No memories to summarize.".to_string();
        }

        let total = records.len();
        let high_importance = records.iter().filter(|r| r.importance >= 8).count();
        let types: Vec<String> = records.iter()
            .map(|r| format!("{:?}", r.memory_type))
            .collect();

        format!(
            "Summary of {} memories ({} high importance). Types: {}.",
            total,
            high_importance,
            types.join(", ")
        )
    }

    pub fn extract_key_points(&self, records: &[MemoryRecord]) -> Vec<String> {
        let mut points: Vec<String> = records.iter()
            .filter(|r| r.importance >= 7)
            .map(|r| {
                let preview = if r.content.len() > 200 {
                    format!("{}...", &r.content[..200])
                } else {
                    r.content.clone()
                };
                format!("[{}] {}", r.importance, preview)
            })
            .collect();
        points.truncate(10);
        points
    }

    pub fn classify_memory_type(&self, records: &[MemoryRecord]) -> String {
        if records.is_empty() {
            return "unknown".to_string();
        }
        let working = records.iter().filter(|r| r.memory_type == 0).count();
        let episodic = records.iter().filter(|r| r.memory_type == 1).count();
        let semantic = records.iter().filter(|r| r.memory_type == 2).count();

        if semantic > working && semantic > episodic {
            "semantic".to_string()
        } else if episodic > working {
            "episodic".to_string()
        } else {
            "mixed".to_string()
        }
    }

    pub fn build_context(&self, records: &[MemoryRecord], project: &str) -> String {
        if records.is_empty() {
            return format!("No context available for project '{}'.", project);
        }

        let high_imp: Vec<&MemoryRecord> = records.iter()
            .filter(|r| r.importance >= 6)
            .take(20)
            .collect();

        let context: Vec<String> = high_imp.iter()
            .enumerate()
            .map(|(i, r)| format!("{}. {}", i + 1, &r.content[..r.content.len().min(300)]))
            .collect();

        format!("Project: {}\nKey Memories:\n{}", project, context.join("\n"))
    }
}

impl Default for Summarizer {
    fn default() -> Self {
        Self::new()
    }
}
