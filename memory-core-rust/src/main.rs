mod storage;
mod context_engine;
mod scheduler;
mod event_bus;
mod agent_registry;
mod project_manager;
mod embedding;
mod keyword_engine;
mod vector_engine;
mod graph_engine;
mod dreaming_engine;
mod scoring_engine;
mod isolation_engine;
mod summarizer;

pub mod memory {
    tonic::include_proto!("memory");
}

use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

use memory::{
    memory_service_server::{MemoryService, MemoryServiceServer},
    StoreRequest, StoreResponse, SearchRequest, SearchResponse,
    DeleteRequest, DeleteResponse, SummaryRequest, SummaryResponse,
    DreamRequest, DreamResponse, ProjectRequest, ProjectResponse,
    MemoryRecord, MemoryType,
};

use storage::Storage;
use context_engine::ContextBuilder;
use scheduler::MemoryScheduler;
use event_bus::{EventBus, Event};
use agent_registry::AgentRegistry;
use project_manager::ProjectManager;
use keyword_engine::KeywordEngine;
use vector_engine::VectorEngine;
use graph_engine::GraphEngine;
use dreaming_engine::DreamingEngine;
use scoring_engine::ScoringEngine;
use isolation_engine::IsolationEngine;
use summarizer::Summarizer;

pub struct MemoryServiceImpl {
    storage: Arc<RwLock<Storage>>,
    context: ContextBuilder,
    keyword: KeywordEngine,
    vector: VectorEngine,
    graph: GraphEngine,
    dreaming: Arc<RwLock<DreamingEngine>>,
    scoring: ScoringEngine,
    isolation: Arc<RwLock<IsolationEngine>>,
    summarizer: Summarizer,
    agents: AgentRegistry,
    projects: Arc<RwLock<ProjectManager>>,
    event_bus: Arc<RwLock<EventBus>>,
}

#[tonic::async_trait]
impl MemoryService for MemoryServiceImpl {
    async fn store_memory(&self, req: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        let store_req = req.into_inner();
        let mut record = store_req.record.unwrap();

        record.importance = self.scoring.score(&record.content);

        if !self.isolation.read().await.can_write(&record.agent_id, &record.namespace, None) {
            return Err(Status::permission_denied("no write access"));
        }

        let mut storage = self.storage.write().await;
        let id = storage.insert(&record).map_err(|e| Status::internal(e.to_string()))?;
        drop(storage);

        self.vector.store(&record).map_err(|e| Status::internal(e.to_string()))?;
        self.graph.store_entities(&record).map_err(|e| Status::internal(e.to_string()))?;

        self.event_bus.read().await.emit(Event::MemorySaved {
            agent_id: record.agent_id.clone(),
            memory_id: id.clone(),
            namespace: record.namespace.clone(),
            importance: record.importance,
        });

        Ok(Response::new(StoreResponse { id, success: true }))
    }

    async fn search_memory(&self, req: Request<SearchRequest>) -> Result<Response<SearchResponse>, Status> {
        let search_req = req.into_inner();

        if !self.isolation.read().await.can_read(&search_req.agent_id, &search_req.namespace) {
            return Err(Status::permission_denied("no read access"));
        }

        let storage = self.storage.read().await;
        let mut results = Vec::new();

        if search_req.use_keyword {
            if let Ok(mut kw) = self.keyword.search(&search_req.query, &search_req.namespace, 50) {
                results.append(&mut kw);
            }
        }

        if search_req.use_vector {
            if let Ok(mut vec) = self.vector.search(&search_req.query, &search_req.namespace, 50) {
                results.append(&mut vec);
            }
        }

        if search_req.use_graph {
            if let Ok(mut gr) = self.graph.search(&search_req.query, &search_req.namespace, 50) {
                results.append(&mut gr);
            }
        }

        results.sort_by(|a, b| b.importance.cmp(&a.importance));
        results.truncate(search_req.limit as usize);

        for r in &results {
            storage.increment_access(&r.id).ok();
        }

        Ok(Response::new(SearchResponse { results }))
    }

    async fn delete_memory(&self, req: Request<DeleteRequest>) -> Result<Response<DeleteResponse>, Status> {
        let del_req = req.into_inner();
        let mut storage = self.storage.write().await;
        storage.delete(&del_req.id).map_err(|e| Status::internal(e.to_string()))?;
        self.vector.delete(&del_req.id).ok();
        self.graph.delete(&del_req.id).ok();
        Ok(Response::new(DeleteResponse { success: true }))
    }

    async fn summarize_memory(&self, req: Request<SummaryRequest>) -> Result<Response<SummaryResponse>, Status> {
        let sum_req = req.into_inner();
        let storage = self.storage.read().await;
        let records = storage.query_by_time(&sum_req.agent_id, &sum_req.namespace, sum_req.since, sum_req.until)
            .map_err(|e| Status::internal(e.to_string()))?;

        let summary = self.summarizer.summarize(&records);
        let key_points = self.summarizer.extract_key_points(&records);
        let memory_type = self.summarizer.classify_memory_type(&records);

        Ok(Response::new(SummaryResponse { summary, key_points, memory_type }))
    }

    async fn dreaming(&self, req: Request<DreamRequest>) -> Result<Response<DreamResponse>, Status> {
        let dream_req = req.into_inner();
        let storage = self.storage.read().await;
        let daily = storage.query_recent(&dream_req.agent_id, &dream_req.namespace, 100)
            .map_err(|e| Status::internal(e.to_string()))?;

        let result = self.dreaming.write().await.process(daily);
        let mut new_memories = Vec::new();

        for dm in &result.facts {
            let record = MemoryRecord {
                id: uuid::Uuid::new_v4().to_string(),
                agent_id: dream_req.agent_id.clone(),
                namespace: dream_req.namespace.clone(),
                memory_type: MemoryType::Semantic as i32,
                importance: (dm.confidence * 10.0) as i32,
                content: dm.content.clone(),
                embedding: vec![],
                entities: vec![],
                created_at: chrono::Utc::now().timestamp_millis(),
                updated_at: chrono::Utc::now().timestamp_millis(),
                access_count: 0,
                last_access: chrono::Utc::now().timestamp_millis(),
                expire_at: None,
            };
            new_memories.push(record);
        }

        let mut storage = self.storage.write().await;
        for mem in &new_memories {
            storage.insert(mem).ok();
        }

        Ok(Response::new(DreamResponse {
            new_memories,
            patterns: result.patterns(),
            inferences: result.inferences(),
            hypotheses: result.hypotheses(),
        }))
    }

    async fn get_project_context(&self, req: Request<ProjectRequest>) -> Result<Response<ProjectResponse>, Status> {
        let proj_req = req.into_inner();
        let context = self.projects.read().await
            .get_context(&proj_req.project, &proj_req.agent_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let storage = self.storage.read().await;
        let ns = format!("project/{}/*", proj_req.project);
        let recent = storage.query_by_namespace(&ns, 30).unwrap_or_default();
        let related = storage.query_related_projects(&proj_req.project).unwrap_or_default();

        Ok(Response::new(ProjectResponse {
            context,
            recent_memories: recent,
            related_projects: related,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let sqlite_path = std::env::var("SQLITE_PATH").unwrap_or_else(|_| "memory.db".to_string());
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let grpc_addr = std::env::var("GRPC_ADDR").unwrap_or_else(|_| "0.0.0.0:50051".to_string());

    let storage = Arc::new(RwLock::new(Storage::new(&sqlite_path)?));
    let event_bus = Arc::new(RwLock::new(EventBus::new()));
    let isolation = Arc::new(RwLock::new(IsolationEngine::new()));

    // Register event listeners
    {
        let storage = storage.clone();
        event_bus.write().await.on("memory_saved", move |event: Event| {
            if let Event::MemorySaved { agent_id, importance, .. } = event {
                log::debug!("Memory saved for {} (importance={})", agent_id, importance);
            }
        });
    }

    let scheduler = Arc::new(MemoryScheduler::new(storage.clone(), event_bus.clone()));
    scheduler.start();

    let service = MemoryServiceImpl {
        context: ContextBuilder::new(),
        storage,
        keyword: KeywordEngine::new(),
        vector: VectorEngine::new(&qdrant_url)?,
        graph: GraphEngine::new(),
        dreaming: Arc::new(RwLock::new(DreamingEngine::new())),
        scoring: ScoringEngine::new(),
        isolation,
        summarizer: Summarizer::new(),
        agents: AgentRegistry::new(),
        projects: Arc::new(RwLock::new(ProjectManager::new(
            storage.clone(),
            Arc::new(RwLock::new(IsolationEngine::new())),
        ))),
        event_bus,
    };

    let addr: std::net::SocketAddr = grpc_addr.parse()?;
    log::info!("Memory server v0.2.0 listening on {}", addr);

    Server::builder()
        .add_service(MemoryServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
