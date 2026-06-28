use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub memory_limit: i32,
    pub capabilities: Vec<String>,
    pub read_globs: Vec<String>,
    pub write_globs: Vec<String>,
}

pub struct AgentRegistry {
    agents: HashMap<String, Agent>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        let mut reg = Self { agents: HashMap::new() };

        reg.register(Agent {
            id: "main".to_string(),
            name: "煤球".to_string(),
            namespace: "main".to_string(),
            memory_limit: 10000,
            capabilities: vec!["read".to_string(), "write".to_string(), "dream".to_string()],
            read_globs: vec!["shared/*".to_string(), "main/*".to_string()],
            write_globs: vec!["main/*".to_string()],
        });

        reg.register(Agent {
            id: "claw".to_string(),
            name: "claw酱".to_string(),
            namespace: "claw".to_string(),
            memory_limit: 5000,
            capabilities: vec!["read".to_string(), "write".to_string(), "code".to_string()],
            read_globs: vec!["shared/*".to_string(), "claw/*".to_string()],
            write_globs: vec!["claw/*".to_string()],
        });

        reg.register(Agent {
            id: "content".to_string(),
            name: "content_farmer".to_string(),
            namespace: "content".to_string(),
            memory_limit: 5000,
            capabilities: vec!["read".to_string(), "write".to_string()],
            read_globs: vec!["shared/*".to_string(), "content/*".to_string()],
            write_globs: vec!["content/*".to_string()],
        });

        reg
    }

    pub fn register(&mut self, agent: Agent) {
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn get(&self, id: &str) -> Option<&Agent> {
        self.agents.get(id)
    }

    pub fn list(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    pub fn exists(&self, id: &str) -> bool {
        self.agents.contains_key(id)
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
