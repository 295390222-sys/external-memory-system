#!/Users/wangjuncong/memory-system/venv/bin/python3
"""Memory gRPC server — pure Python, no Rust/Docker needed."""

import os
import sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))) + '/memory-plugin')
import time
import json
import sqlite3
import uuid
import hashlib
import logging
import threading
from concurrent import futures
from dataclasses import dataclass, field

import grpc
from memory_pb2 import (
    StoreRequest, StoreResponse,
    SearchRequest, SearchResponse,
    DeleteRequest, DeleteResponse,
    SummaryRequest, SummaryResponse,
    DreamRequest, DreamResponse,
    ProjectRequest, ProjectResponse,
    MemoryRecord as ProtoRecord,
)
from memory_pb2_grpc import (
    MemoryServiceServicer,
    add_MemoryServiceServicer_to_server,
)

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("memory-server")

SCHEMA_SQL = """
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
CREATE INDEX IF NOT EXISTS idx_memory_agent ON memory_records(agent_id);
CREATE INDEX IF NOT EXISTS idx_memory_namespace ON memory_records(namespace);
CREATE INDEX IF NOT EXISTS idx_memory_importance ON memory_records(importance DESC);
CREATE INDEX IF NOT EXISTS idx_memory_expire ON memory_records(expire_at) WHERE expire_at IS NOT NULL;

CREATE TABLE IF NOT EXISTS entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    memory_id TEXT NOT NULL REFERENCES memory_records(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    relation TEXT DEFAULT '',
    attributes TEXT DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_entity_name ON entities(name);

CREATE TABLE IF NOT EXISTS entity_graph (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_entity TEXT NOT NULL,
    target_entity TEXT NOT NULL,
    relation TEXT NOT NULL,
    weight REAL DEFAULT 1.0,
    created_at INTEGER NOT NULL,
    UNIQUE(source_entity, target_entity, relation)
);

CREATE TABLE IF NOT EXISTS dream_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    namespace TEXT NOT NULL,
    fact_summary TEXT,
    inferences TEXT DEFAULT '[]',
    hypotheses TEXT DEFAULT '[]',
    created_at INTEGER NOT NULL
);
"""


class MemoryServer(MemoryServiceServicer):
    def __init__(self, db_path: str):
        self.conn = sqlite3.connect(db_path, check_same_thread=False)
        self.conn.executescript(SCHEMA_SQL)
        self.conn.commit()
        self.lock = threading.Lock()
        logger.info(f"SQLite ready: {db_path}")

    # ── Store ───────────────────────────────────────────────
    def StoreMemory(self, request, context):
        r = request.record
        mem_id = r.id or str(uuid.uuid4())
        mem_type = ["working", "episodic", "semantic", "procedural", "dream"][r.memory_type]
        importance = self._score(r.content)

        with self.lock:
            self.conn.execute(
                """INSERT OR REPLACE INTO memory_records
                   (id, agent_id, namespace, memory_type, importance, content,
                    created_at, updated_at, access_count, last_access, expire_at)
                   VALUES (?,?,?,?,?,?, ?,?,?,?,?)""",
                (mem_id, r.agent_id, r.namespace, mem_type, importance, r.content,
                 r.created_at or int(time.time() * 1000),
                 r.updated_at or int(time.time() * 1000),
                 r.access_count, r.last_access or int(time.time() * 1000),
                 r.expire_at if r.HasField("expire_at") else None),
            )
            # entities
            for e in r.entities:
                self.conn.execute(
                    "INSERT INTO entities (memory_id, name, relation, attributes) VALUES (?,?,?,?)",
                    (mem_id, e.name, e.relation, json.dumps(e.attributes)),
                )
            self.conn.commit()

        logger.info(f"Stored {mem_id} ({mem_type}, imp={importance})")
        return StoreResponse(id=mem_id, success=True)

    # ── Search ──────────────────────────────────────────────
    def SearchMemory(self, request, context):
        q = request.query
        ns = request.namespace
        limit = request.limit or 10
        pattern = f"%{q}%"

        with self.lock:
            rows = self.conn.execute(
                """SELECT id, agent_id, namespace, memory_type, importance, content,
                          created_at, updated_at, access_count, last_access, expire_at
                   FROM memory_records
                   WHERE namespace LIKE ? AND content LIKE ?
                   ORDER BY importance DESC
                   LIMIT ?""",
                (ns, pattern, limit),
            ).fetchall()

            results = []
            for row in rows:
                results.append(self._row_to_proto(row))
                # increment access
                self.conn.execute(
                    "UPDATE memory_records SET access_count = access_count + 1, last_access = ? WHERE id = ?",
                    (int(time.time() * 1000), row[0]),
                )
            self.conn.commit()

        return SearchResponse(results=results)

    # ── Delete ──────────────────────────────────────────────
    def DeleteMemory(self, request, context):
        with self.lock:
            self.conn.execute("DELETE FROM entities WHERE memory_id = ?", (request.id,))
            self.conn.execute("DELETE FROM memory_records WHERE id = ?", (request.id,))
            self.conn.commit()
        return DeleteResponse(success=True)

    # ── Summarize ───────────────────────────────────────────
    def SummarizeMemory(self, request, context):
        with self.lock:
            rows = self.conn.execute(
                """SELECT id, agent_id, namespace, memory_type, importance, content,
                          created_at, updated_at, access_count, last_access, expire_at
                   FROM memory_records
                   WHERE agent_id = ? AND namespace LIKE ? AND created_at >= ? AND created_at <= ?
                   ORDER BY created_at DESC""",
                (request.agent_id, request.namespace, request.since, request.until),
            ).fetchall()

        if not rows:
            return SummaryResponse(summary="No memories.", key_points=[], memory_type="unknown")

        high = [r for r in rows if r[4] >= 7]
        key_points = [f"[{r[4]}] {r[5][:200]}" for r in high[:10]]
        summary = f"Summary of {len(rows)} memories ({len(high)} high importance)."
        mtype = "semantic" if len(high) > len(rows) / 2 else "mixed"

        return SummaryResponse(summary=summary, key_points=key_points, memory_type=mtype)

    # ── Dream ───────────────────────────────────────────────
    def Dreaming(self, request, context):
        with self.lock:
            rows = self.conn.execute(
                """SELECT id, agent_id, namespace, memory_type, importance, content,
                          created_at, updated_at, access_count, last_access, expire_at
                   FROM memory_records
                   WHERE agent_id = ? AND namespace LIKE ?
                   ORDER BY importance DESC LIMIT 100""",
                (request.agent_id, request.namespace),
            ).fetchall()

        new_memories = []
        patterns = []
        inferences = []
        hypotheses = []

        if rows:
            high = [r for r in rows if r[4] >= 6]
            seen = set()
            facts = []
            for r in high:
                h = hashlib.md5(r[5][:100].encode()).hexdigest()
                if h not in seen:
                    seen.add(h)
                    facts.append(r[5])

            if facts:
                # cluster by rough similarity
                clusters = []
                for f in facts:
                    added = False
                    for c in clusters:
                        if self._similarity(f, c[0]) > 0.5:
                            c.append(f)
                            added = True
                            break
                    if not added:
                        clusters.append([f])

                for i, cluster in enumerate(clusters):
                    if len(cluster) < 2:
                        continue
                    summary = f"[Fact] {'; '.join(c[:100] for c in cluster)}"
                    mem = ProtoRecord(
                        id=str(uuid.uuid4()),
                        agent_id=request.agent_id,
                        namespace=request.namespace,
                        memory_type=2,  # semantic
                        importance=8,
                        content=summary,
                        created_at=int(time.time() * 1000),
                        updated_at=int(time.time() * 1000),
                        access_count=0,
                        last_access=int(time.time() * 1000),
                    )
                    new_memories.append(mem)

                    if i < 3:
                        inferences.append(f"[Inference] Pattern detected: {cluster[0][:80]}...")
                    if i < 2:
                        hypotheses.append(f"[Hypothesis] If {cluster[0][:60]}... holds, then new approach needed")

            # store dream log
            with self.lock:
                self.conn.execute(
                    "INSERT INTO dream_log (agent_id, namespace, fact_summary, inferences, hypotheses, created_at) VALUES (?,?,?,?,?,?)",
                    (request.agent_id, request.namespace,
                     json.dumps([m.content[:100] for m in new_memories]),
                     json.dumps(inferences), json.dumps(hypotheses),
                     int(time.time() * 1000)),
                )
                self.conn.commit()

                for mem in new_memories:
                    self.conn.execute(
                        """INSERT OR IGNORE INTO memory_records
                           (id, agent_id, namespace, memory_type, importance, content,
                            created_at, updated_at, access_count, last_access)
                           VALUES (?,?,?,?,?,?,?,?,?,?)""",
                        (mem.id, mem.agent_id, mem.namespace, "semantic", mem.importance,
                         mem.content, mem.created_at, mem.updated_at,
                         mem.access_count, mem.last_access),
                    )
                self.conn.commit()

        return DreamResponse(
            new_memories=new_memories,
            patterns=patterns,
            inferences=inferences,
            hypotheses=hypotheses,
        )

    # ── Project Context ─────────────────────────────────────
    def GetProjectContext(self, request, context):
        ns = f"project/{request.project}/*"
        with self.lock:
            rows = self.conn.execute(
                """SELECT id, agent_id, namespace, memory_type, importance, content,
                          created_at, updated_at, access_count, last_access, expire_at
                   FROM memory_records
                   WHERE namespace LIKE ?
                   ORDER BY importance DESC, created_at DESC LIMIT 30""",
                (ns,),
            ).fetchall()

            related = self.conn.execute(
                "SELECT DISTINCT namespace FROM memory_records WHERE namespace LIKE 'project/%' AND namespace != ? LIMIT 20",
                (ns,),
            ).fetchall()

        proto_records = [self._row_to_proto(r) for r in rows]
        context_str = f"Project: {request.project}\n"
        for r in rows:
            context_str += f"[{r[4]}] {r[5][:200]}\n"

        return ProjectResponse(
            context=context_str,
            recent_memories=proto_records,
            related_projects=[r[0] for r in related],
        )

    # ── Helpers ─────────────────────────────────────────────
    def _score(self, content: str) -> int:
        score = 5
        if len(content) > 500: score += 1
        if len(content) > 2000: score += 1
        important = ["project", "deploy", "release", "bug", "fix", "migration",
                      "architecture", "decision", "完成", "修复", "部署", "上线"]
        if any(kw in content.lower() for kw in important):
            score += 1
        code_markers = ["{", "}", ";", "fn ", "pub ", "struct", "impl", "async"]
        if any(m in content for m in code_markers):
            score += 1
        return max(0, min(10, score))

    def _similarity(self, a: str, b: str) -> float:
        wa = set(a.split()[:50])
        wb = set(b.split()[:50])
        if not wa or not wb:
            return 0.0
        return len(wa & wb) / max(len(wa), len(wb))

    def _row_to_proto(self, row):
        mem_type_map = {"working": 0, "episodic": 1, "semantic": 2, "procedural": 3, "dream": 4}
        return ProtoRecord(
            id=row[0], agent_id=row[1], namespace=row[2],
            memory_type=mem_type_map.get(row[3], 0),
            importance=row[4], content=row[5],
            created_at=row[6], updated_at=row[7],
            access_count=row[8], last_access=row[9],
            expire_at=row[10] if len(row) > 10 else None,
        )


def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", default=os.path.expanduser("~/.memory-system/memory.db"))
    parser.add_argument("--port", type=int, default=50051)
    args = parser.parse_args()

    os.makedirs(os.path.dirname(args.db), exist_ok=True)

    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    add_MemoryServiceServicer_to_server(MemoryServer(args.db), server)
    server.add_insecure_port(f"0.0.0.0:{args.port}")
    server.start()
    logger.info(f"Memory server ready on 0.0.0.0:{args.port}, db={args.db}")
    server.wait_for_termination()


if __name__ == "__main__":
    main()
