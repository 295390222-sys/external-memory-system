use async_trait::async_trait;
use super::EmbeddingProvider;

pub struct SiliconFlowEmbedding {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl SiliconFlowEmbedding {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "BAAI/bge-large-zh-v1.5".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }
}

#[async_trait]
impl EmbeddingProvider for SiliconFlowEmbedding {
    fn name(&self) -> &str {
        "siliconflow"
    }

    fn dimension(&self) -> usize {
        1024
    }

    async fn embed(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>> {
        let body = serde_json::json!({
            "model": self.model,
            "input": texts,
        });

        let resp = self.client
            .post("https://api.siliconflow.cn/v1/embeddings")
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
