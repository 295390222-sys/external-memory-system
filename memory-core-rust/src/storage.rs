use rusqlite::{Connection, params};
use serde_json;
use crate::memory::{MemoryRecord, MemoryType, Entity};

#[derive(Clone)]
pub struct Storage {
    pub conn: Connection,
}

impl Storage {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(include_str!("../../database/sqlite/schema.sql"))?;
        Ok(Self { conn })
    }

    pub fn insert(&mut self, record: &MemoryRecord) -> rusqlite::Result<String> {
        let id = if record.id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            record.id.clone()
        };

        let entities_json = serde_json::to_string(&record.entities).unwrap_or_default();

        self.conn.execute(
            "INSERT INTO memory_records (id, agent_id, namespace, memory_type, importance, content, created_at, updated_at, access_count, last_access, expire_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                content=excluded.content,
                importance=excluded.importance,
                updated_at=excluded.updated_at,
                access_count=excluded.access_count,
                last_access=excluded.last_access,
                expire_at=excluded.expire_at",
            params![
                id,
                record.agent_id,
                record.namespace,
                match MemoryType::try_from(record.memory_type).unwrap_or(MemoryType::Working) {
                    MemoryType::Working => "working",
                    MemoryType::Episodic => "episodic",
                    MemoryType::Semantic => "semantic",
                    MemoryType::Procedural => "procedural",
                    MemoryType::Dream => "dream",
                },
                record.importance,
                record.content,
                record.created_at,
                record.updated_at,
                record.access_count,
                record.last_access,
                record.expire_at,
            ],
        )?;

        for entity in &record.entities {
            let attrs = serde_json::to_string(&entity.attributes).unwrap_or_default();
            self.conn.execute(
                "INSERT INTO entities (memory_id, name, relation, attributes) VALUES (?1, ?2, ?3, ?4)",
                params![id, entity.name, entity.relation, attrs],
            )?;
        }

        Ok(id)
    }

    pub fn search_keyword(&self, query: &str, namespace: &str, limit: usize) -> rusqlite::Result<Vec<MemoryRecord>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_id, namespace, memory_type, importance, content, created_at, updated_at, access_count, last_access, expire_at
             FROM memory_records
             WHERE namespace LIKE ?1 AND content LIKE ?2
             ORDER BY importance DESC
             LIMIT ?3"
        )?;

        let rows = stmt.query_map(params![namespace, pattern, limit as i64], |row| {
            let mem_type_str: String = row.get(3)?;
            Ok(MemoryRecord {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                namespace: row.get(2)?,
                memory_type: mem_type_to_i32(&mem_type_str),
                importance: row.get(4)?,
                content: row.get(5)?,
                embedding: vec![],
                entities: vec![],
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                access_count: row.get(8)?,
                last_access: row.get(9)?,
                expire_at: row.get(10)?,
            })
        })?;

        rows.collect()
    }

    pub fn query_by_time(&self, agent_id: &str, namespace: &str, since: i64, until: i64) -> rusqlite::Result<Vec<MemoryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_id, namespace, memory_type, importance, content, created_at, updated_at, access_count, last_access, expire_at
             FROM memory_records
             WHERE agent_id = ?1 AND namespace LIKE ?2 AND created_at >= ?3 AND created_at <= ?4
             ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map(params![agent_id, namespace, since, until], |row| {
            let mem_type_str: String = row.get(3)?;
            Ok(MemoryRecord {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                namespace: row.get(2)?,
                memory_type: mem_type_to_i32(&mem_type_str),
                importance: row.get(4)?,
                content: row.get(5)?,
                embedding: vec![],
                entities: vec![],
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                access_count: row.get(8)?,
                last_access: row.get(9)?,
                expire_at: row.get(10)?,
            })
        })?;

        rows.collect()
    }

    pub fn query_recent(&self, agent_id: &str, namespace: &str, limit: usize) -> rusqlite::Result<Vec<MemoryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_id, namespace, memory_type, importance, content, created_at, updated_at, access_count, last_access, expire_at
             FROM memory_records
             WHERE agent_id = ?1 AND namespace LIKE ?2
             ORDER BY created_at DESC
             LIMIT ?3"
        )?;

        let rows = stmt.query_map(params![agent_id, namespace, limit as i64], |row| {
            let mem_type_str: String = row.get(3)?;
            Ok(MemoryRecord {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                namespace: row.get(2)?,
                memory_type: mem_type_to_i32(&mem_type_str),
                importance: row.get(4)?,
                content: row.get(5)?,
                embedding: vec![],
                entities: vec![],
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                access_count: row.get(8)?,
                last_access: row.get(9)?,
                expire_at: row.get(10)?,
            })
        })?;

        rows.collect()
    }

    pub fn query_by_namespace(&self, namespace_pattern: &str, limit: usize) -> rusqlite::Result<Vec<MemoryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_id, namespace, memory_type, importance, content, created_at, updated_at, access_count, last_access, expire_at
             FROM memory_records
             WHERE namespace LIKE ?1
             ORDER BY importance DESC, created_at DESC
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![namespace_pattern, limit as i64], |row| {
            let mem_type_str: String = row.get(3)?;
            Ok(MemoryRecord {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                namespace: row.get(2)?,
                memory_type: mem_type_to_i32(&mem_type_str),
                importance: row.get(4)?,
                content: row.get(5)?,
                embedding: vec![],
                entities: vec![],
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                access_count: row.get(8)?,
                last_access: row.get(9)?,
                expire_at: row.get(10)?,
            })
        })?;

        rows.collect()
    }

    pub fn query_related_projects(&self, project: &str) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT namespace FROM memory_records
             WHERE namespace LIKE 'project/%' AND namespace != ?1
             LIMIT 20"
        )?;

        let rows = stmt.query_map(params![format!("project/{}/*", project)], |row| {
            row.get::<_, String>(0)
        })?;

        rows.collect()
    }

    pub fn delete(&mut self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM entities WHERE memory_id = ?1", params![id])?;
        self.conn.execute("DELETE FROM memory_records WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn increment_access(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE memory_records SET access_count = access_count + 1, last_access = ?1 WHERE id = ?2",
            params![chrono::Utc::now().timestamp_millis(), id],
        )?;
        Ok(())
    }
}

fn mem_type_to_i32(s: &str) -> i32 {
    match s {
        "working" => 0,
        "episodic" => 1,
        "semantic" => 2,
        "procedural" => 3,
        "dream" => 4,
        _ => 0,
    }
}
