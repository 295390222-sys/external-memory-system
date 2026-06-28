use async_trait::async_trait;
use super::EmbeddingProvider;

pub struct OpenAIEmbedding {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAIEmbedding {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "text-embedding-3-small".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbedding {
    fn name(&self) -> &str {
        "openai"
    }

    fn dimension(&self) -> usize {
        1536
    }

    async fn embed(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>> {
        let body = serde_json::json!({
            "model": self.model,
            "input": texts,
        });

        let resp = self.client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;
        let mut results = Vec::new();

        if let Some(arr) = data["data"].as_array() {
            for entry in arr {
                if let Some(embedding) = entry["embedding"].as_array() {
                    let vec: Vec<f32> = embedding.iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect();
                    results.push(vec);
                }
            }
        }

        Ok(results)
    }
}
