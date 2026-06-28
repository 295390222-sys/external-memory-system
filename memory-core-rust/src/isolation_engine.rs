use std::collections::HashMap;

struct AccessRule {
    read_globs: Vec<String>,
    write_globs: Vec<String>,
}

pub struct IsolationEngine {
    project_rules: HashMap<String, AccessRule>,
    agent_rules: HashMap<String, AccessRule>,
}

impl IsolationEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            project_rules: HashMap::new(),
            agent_rules: HashMap::new(),
        };

        engine.project_rules.insert("ai-drama".to_string(), AccessRule {
            read_globs: vec!["shared/*".to_string(), "project/ai-drama/*".to_string()],
            write_globs: vec!["project/ai-drama/*".to_string()],
        });

        engine.agent_rules.insert("content".to_string(), AccessRule {
            read_globs: vec!["shared/*".to_string(), "content/*".to_string()],
            write_globs: vec!["content/*".to_string()],
        });

        engine
    }

    pub fn can_read(&self, agent_id: &str, namespace: &str) -> bool {
        if let Some(rule) = self.agent_rules.get(agent_id) {
            rule.read_globs.iter().any(|g| glob_match(g, namespace))
        } else {
            true
        }
    }

    pub fn can_write(&self, agent_id: &str, namespace: &str, project: Option<&str>) -> bool {
        if let Some(proj) = project {
            if let Some(rule) = self.project_rules.get(proj) {
                return rule.write_globs.iter().any(|g| glob_match(g, namespace));
            }
        }
        if let Some(rule) = self.agent_rules.get(agent_id) {
            return rule.write_globs.iter().any(|g| glob_match(g, namespace));
        }
        true
    }
}

fn glob_match(pattern: &str, namespace: &str) -> bool {
    if pattern == "*" || pattern == "*/*" {
        return true;
    }
    let regex_pattern = format!("^{}$", regex::escape(pattern).replace("\\*", ".*"));
    regex::Regex::new(&regex_pattern)
        .map(|re| re.is_match(namespace))
        .unwrap_or(false)
}

impl Default for IsolationEngine {
    fn default() -> Self {
        Self::new()
    }
}
