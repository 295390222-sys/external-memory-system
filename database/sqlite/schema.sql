CREATE TABLE IF NOT EXISTS memory_records (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    namespace TEXT NOT NULL DEFAULT 'shared',
    memory_type TEXT NOT NULL CHECK(memory_type IN ('working','episodic','semantic','procedural','dream')),
    importance INTEGER NOT NULL DEFAULT 0,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    access_count INTEGER NOT NULL DEFAULT 0,
    last_access INTEGER NOT NULL,
    expire_at INTEGER
);

CREATE INDEX idx_memory_agent ON memory_records(agent_id);
CREATE INDEX idx_memory_namespace ON memory_records(namespace);
CREATE INDEX idx_memory_type ON memory_records(memory_type);
CREATE INDEX idx_memory_importance ON memory_records(importance DESC);
CREATE INDEX idx_memory_created ON memory_records(created_at DESC);
CREATE INDEX idx_memory_expire ON memory_records(expire_at) WHERE expire_at IS NOT NULL;

CREATE TABLE IF NOT EXISTS entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    memory_id TEXT NOT NULL REFERENCES memory_records(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    relation TEXT DEFAULT '',
    attributes TEXT DEFAULT '{}'
);

CREATE INDEX idx_entity_name ON entities(name);
CREATE INDEX idx_entity_memory ON entities(memory_id);

CREATE TABLE IF NOT EXISTS entity_graph (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_entity TEXT NOT NULL,
    target_entity TEXT NOT NULL,
    relation TEXT NOT NULL,
    weight REAL DEFAULT 1.0,
    created_at INTEGER NOT NULL,
    UNIQUE(source_entity, target_entity, relation)
);

CREATE INDEX idx_graph_source ON entity_graph(source_entity);
CREATE INDEX idx_graph_target ON entity_graph(target_entity);

CREATE TABLE IF NOT EXISTS dream_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    namespace TEXT NOT NULL,
    pattern_summary TEXT,
    inferences TEXT DEFAULT '[]',
    hypotheses TEXT DEFAULT '[]',
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_dream_agent ON dream_log(agent_id);
