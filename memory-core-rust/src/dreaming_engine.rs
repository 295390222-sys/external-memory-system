use crate::memory::MemoryRecord;

#[derive(Debug, Clone, PartialEq)]
pub enum DreamCategory {
    Fact,
    Inference,
    Hypothesis,
}

#[derive(Debug, Clone)]
pub struct DreamMemory {
    pub content: String,
    pub category: DreamCategory,
    pub source_ids: Vec<String>,
    pub confidence: f64,
}

pub struct DreamResult {
    pub facts: Vec<DreamMemory>,
    pub inferences: Vec<DreamMemory>,
    pub hypotheses: Vec<DreamMemory>,
}

impl DreamResult {
    pub fn new_semantic(&self) -> Vec<String> {
        self.facts.iter().map(|f| f.content.clone()).collect()
    }

    pub fn patterns(&self) -> Vec<String> {
        self.facts.iter()
            .filter(|f| f.confidence > 0.8)
            .map(|f| format!("Pattern: {}", f.content))
            .collect()
    }

    pub fn inferences(&self) -> Vec<String> {
        self.inferences.iter().map(|i| format!("Inference: {}", i.content)).collect()
    }

    pub fn hypotheses(&self) -> Vec<String> {
        self.hypotheses.iter().map(|h| format!("Hypothesis: {}", h.content)).collect()
    }
}

pub struct DreamingEngine;

impl DreamingEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn process(&self, memories: Vec<MemoryRecord>) -> DreamResult {
        let source_ids: Vec<String> = memories.iter().map(|m| m.id.clone()).collect();

        let clusters = self.cluster(&memories);

        let facts: Vec<DreamMemory> = self.extract_facts(&clusters, &source_ids);
        let inferences: Vec<DreamMemory> = self.make_inferences(&facts);
        let hypotheses: Vec<DreamMemory> = self.form_hypotheses(&facts, &inferences);

        DreamResult { facts, inferences, hypotheses }
    }

    fn cluster(&self, memories: &[MemoryRecord]) -> Vec<Vec<&MemoryRecord>> {
        let mut clusters: Vec<Vec<&MemoryRecord>> = Vec::new();
        for mem in memories {
            let mut added = false;
            for cluster in &mut clusters {
                if let Some(first) = cluster.first() {
                    if self.similarity(&first.content, &mem.content) > 0.6 {
                        cluster.push(mem);
                        added = true;
                        break;
                    }
                }
            }
            if !added {
                clusters.push(vec![mem]);
            }
        }
        clusters
    }

    fn extract_facts(&self, clusters: &[Vec<&MemoryRecord>], source_ids: &[String]) -> Vec<DreamMemory> {
        clusters.iter().filter_map(|cluster| {
            if cluster.len() < 2 {
                return None;
            }

            let contents: Vec<&str> = cluster.iter().map(|m| m.content.as_str()).collect();
            let summary = format!("[Fact] {}", contents.join("; "));

            let avg_importance: i32 = cluster.iter().map(|m| m.importance).sum::<i32>() / cluster.len() as i32;
            let confidence = avg_importance as f64 / 10.0;

            Some(DreamMemory {
                content: summary,
                category: DreamCategory::Fact,
                source_ids: source_ids.to_vec(),
                confidence,
            })
        }).collect()
    }

    fn make_inferences(&self, facts: &[DreamMemory]) -> Vec<DreamMemory> {
        facts.iter()
            .filter(|f| f.confidence > 0.7)
            .take(3)
            .map(|f| DreamMemory {
                content: format!("[Inference] Based on correlated patterns: {}", f.content),
                category: DreamCategory::Inference,
                source_ids: f.source_ids.clone(),
                confidence: f.confidence * 0.7,
            })
            .collect()
    }

    fn form_hypotheses(&self, facts: &[DreamMemory], inferences: &[DreamMemory]) -> Vec<DreamMemory> {
        let mut hypotheses = Vec::new();

        for fact in facts.iter().take(2) {
            for inf in inferences.iter().take(1) {
                hypotheses.push(DreamMemory {
                    content: format!(
                        "[Hypothesis] If fact patterns ({:.50}...) hold, then {}",
                        fact.content, inf.content
                    ),
                    category: DreamCategory::Hypothesis,
                    source_ids: {
                        let mut ids = fact.source_ids.clone();
                        ids.extend(inf.source_ids.clone());
                        ids
                    },
                    confidence: fact.confidence * inf.confidence * 0.5,
                });
            }
        }

        hypotheses
    }

    fn similarity(&self, a: &str, b: &str) -> f64 {
        let words_a: Vec<&str> = a.split_whitespace().collect();
        let words_b: Vec<&str> = b.split_whitespace().collect();
        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }
        let overlap = words_a.iter().filter(|w| words_b.contains(w)).count();
        overlap as f64 / words_a.len().max(words_b.len()) as f64
    }
}

impl Default for DreamingEngine {
    fn default() -> Self {
        Self::new()
    }
}
