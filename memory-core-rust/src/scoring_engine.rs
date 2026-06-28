pub struct ScoringEngine;

impl ScoringEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn score(&self, content: &str) -> i32 {
        let mut score = 5;

        if content.len() > 500 {
            score += 1;
        }
        if content.len() > 2000 {
            score += 1;
        }

        let important_keywords = [
            "project", "deploy", "release", "production", "critical",
            "bug", "fix", "migration", "api", "database",
            "architecture", "decision", "goal", "milestone",
            "完成", "修复", "部署", "上线", "架构",
        ];
        let lower = content.to_lowercase();
        for kw in &important_keywords {
            if lower.contains(kw) {
                score += 1;
                break;
            }
        }

        let code_markers = ['{', '}', ';', 'fn', 'pub', 'struct', "impl", "async"];
        if code_markers.iter().any(|m| content.contains(*m)) {
            score += 1;
        }

        score.clamp(0, 10)
    }

    pub fn decay(importance: i32, access_count: i32, last_access: i64) -> f64 {
        let now = chrono::Utc::now().timestamp_millis();
        let hours_since_access = (now - last_access) as f64 / 3600000.0;
        let time_decay = (-hours_since_access / 720.0).exp();
        let access_factor = (access_count as f64 + 1.0).ln();
        importance as f64 * access_factor * time_decay
    }

    pub fn should_forget(weight: f64) -> bool {
        weight < 1.0
    }
}

impl Default for ScoringEngine {
    fn default() -> Self {
        Self::new()
    }
}
