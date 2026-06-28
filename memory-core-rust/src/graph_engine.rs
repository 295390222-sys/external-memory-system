use crate::memory::{MemoryRecord, Entity};
use std::collections::HashMap;

pub struct GraphEngine {
    edges: HashMap<String, Vec<(String, String, f64)>>,
}

impl GraphEngine {
    pub fn new() -> Self {
        Self { edges: HashMap::new() }
    }

    pub fn store_entities(&mut self, record: &MemoryRecord) -> anyhow::Result<()> {
        for entity in &record.entities {
            self.edges
                .entry(entity.name.clone())
                .or_default()
                .push((record.id.clone(), entity.relation.clone(), 1.0));
        }
        Ok(())
    }

    pub fn search(&self, _query: &str, _namespace: &str, _limit: usize) -> anyhow::Result<Vec<MemoryRecord>> {
        Ok(vec![])
    }

    pub fn traverse(&self, entity_name: &str, depth: usize) -> Vec<&str> {
        let mut visited = Vec::new();
        let mut stack = vec![entity_name];

        for _ in 0..depth {
            if let Some(current) = stack.pop() {
                if let Some(neighbors) = self.edges.get(current) {
                    for (neighbor, _, _) in neighbors {
                        if !visited.contains(&neighbor.as_str()) {
                            visited.push(neighbor.as_str());
                            stack.push(neighbor);
                        }
                    }
                }
            }
        }

        visited
    }

    pub fn delete(&mut self, id: &str) -> anyhow::Result<()> {
        self.edges.retain(|_, v| {
            v.retain(|(mem_id, _, _)| mem_id != id);
            !v.is_empty()
        });
        Ok(())
    }
}

impl Default for GraphEngine {
    fn default() -> Self {
        Self::new()
    }
}
