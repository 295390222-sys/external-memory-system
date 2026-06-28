use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn name(&self) -> &str;
    fn dimension(&self) -> usize;
    async fn embed(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>>;
    async fn embed_one(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let results = self.embed(&[text]).await?;
        results.into_iter().next().ok_or_else(|| anyhow::anyhow!("no embedding returned"))
    }
}

pub mod openai;
pub mod ollama;
pub mod bge;
pub mod siliconflow;
