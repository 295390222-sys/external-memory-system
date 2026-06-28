use crate::memory::MemoryRecord;

pub struct VectorEngine {
    client: Option<qdrant_client::Qdrant>,
}

impl VectorEngine {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        let client = qdrant_client::Qdrant::from_url(url)
            .build()?;
        Ok(Self { client: Some(client) })
    }

    pub fn store(&self, record: &MemoryRecord) -> anyhow::Result<()> {
        if record.embedding.is_empty() {
            return Ok(());
        }
        Ok(())
    }

    pub fn search(&self, _query: &str, _namespace: &str, _limit: usize) -> anyhow::Result<Vec<MemoryRecord>> {
        Ok(vec![])
    }

    pub fn delete(&self, id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
