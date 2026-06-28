use async_trait::async_trait;
use super::EmbeddingProvider;

pub struct OllamaEmbedding {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaEmbedding {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: "nomic-embed-text".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.base_url = url.trim_end_matches('/').to_string();
        self
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbedding {
    fn name(&self) -> &str {
        "ollama"
    }

    fn dimension(&self) -> usize {
        768
    }

    async fn embed(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();

        for text in texts {
            let body = serde_json::json!({
                "model": self.model,
                "prompt": text,
            });

            let resp = self.client
                .post(format!("{}/api/embeddings", self.base_url))
                .json(&body)
                .send()
                .await?;

            let data: serde_json::Value = resp.json().await?;
            if let Some(embedding) = data["embedding"].as_array() {
                let vec: Vec<f32> = embedding.iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();
                results.push(vec);
            }
        }

        Ok(results)
    }
}

impl Default for OllamaEmbedding {
    fn default() -> Self {
        Self::new()
    }
}
