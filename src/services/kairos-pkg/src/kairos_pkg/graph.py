# Personal Knowledge Graph core
# Stores entities as nodes with typed relationships and vector embeddings

import json
import sqlite3
import hashlib
from datetime import datetime, timezone
from typing import Optional

try:
    import sqlite_vec
except ImportError:
    sqlite_vec = None


class KnowledgeGraph:
    def __init__(self, db_path: str = "~/.hermes/knowledge.db"):
        self.db_path = db_path
        self.conn: Optional[sqlite3.Connection] = None

    def connect(self):
        self.conn = sqlite3.connect(self.db_path)
        self.conn.row_factory = sqlite3.Row
        self.conn.execute("PRAGMA journal_mode=WAL")
        self.conn.execute("PRAGMA foreign_keys=ON")

        if sqlite_vec:
            self.conn.enable_load_extension(True)
            sqlite_vec.load(self.conn)
            self.conn.enable_load_extension(False)

        self._init_schema()

    def _init_schema(self):
        self.conn.executescript("""
            CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                metadata TEXT,
                confidence REAL DEFAULT 1.0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                deleted_at TEXT
            );

            CREATE TABLE IF NOT EXISTS relationships (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
                target_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
                type TEXT NOT NULL,
                weight REAL DEFAULT 1.0,
                metadata TEXT,
                confidence REAL DEFAULT 1.0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                deleted_at TEXT,
                UNIQUE(source_id, target_id, type)
            );

            CREATE INDEX IF NOT EXISTS idx_ent_type ON entities(type);
            CREATE INDEX IF NOT EXISTS idx_ent_name ON entities(name);
            CREATE INDEX IF NOT EXISTS idx_rel_source ON relationships(source_id);
            CREATE INDEX IF NOT EXISTS idx_rel_target ON relationships(target_id);
            CREATE INDEX IF NOT EXISTS idx_rel_type ON relationships(type);

            CREATE VIRTUAL TABLE IF NOT EXISTS entity_fts USING fts5(
                name, description, metadata,
                content='entities',
                content_rowid='rowid'
            );

            CREATE TRIGGER IF NOT EXISTS ent_ai AFTER INSERT ON entities
            BEGIN
                INSERT INTO entity_fts(rowid, name, description, metadata)
                VALUES (new.rowid, new.name, new.description, new.metadata);
            END;
        """)

        if sqlite_vec:
            try:
                self.conn.execute("""
                    CREATE VIRTUAL TABLE IF NOT EXISTS entity_vectors
                    USING vec0(
                        id TEXT PRIMARY KEY,
                        embedding float[768]
                    )
                """)
            except sqlite3.OperationalError:
                pass

        self.conn.commit()

    def add_entity(self, entity_id: str, type_: str, name: str,
                   description: str = "", metadata: dict = None,
                   confidence: float = 1.0) -> str:
        eid = entity_id or hashlib.sha256(
            f"{type_}:{name}:{datetime.now().isoformat()}".encode()
        ).hexdigest()[:16]

        self.conn.execute(
            """INSERT OR REPLACE INTO entities
               (id, type, name, description, metadata, confidence)
               VALUES (?, ?, ?, ?, ?, ?)""",
            (eid, type_, name, description,
             json.dumps(metadata or {}), confidence)
        )
        self.conn.commit()
        return eid

    def add_relationship(self, source_id: str, target_id: str,
                         type_: str, weight: float = 1.0,
                         confidence: float = 1.0,
                         metadata: dict = None) -> str:
        rid = hashlib.sha256(
            f"{source_id}:{target_id}:{type_}".encode()
        ).hexdigest()[:16]

        self.conn.execute(
            """INSERT OR REPLACE INTO relationships
               (id, source_id, target_id, type, weight, confidence, metadata)
               VALUES (?, ?, ?, ?, ?, ?, ?)""",
            (rid, source_id, target_id, type_, weight, confidence,
             json.dumps(metadata or {}))
        )
        self.conn.commit()
        return rid

    def query_entities(self, type_: str = None, name: str = None,
                       limit: int = 50) -> list:
        sql = "SELECT * FROM entities WHERE deleted_at IS NULL"
        params = []

        if type_:
            sql += " AND type = ?"
            params.append(type_)
        if name:
            sql += " AND name LIKE ?"
            params.append(f"%{name}%")

        sql += f" ORDER BY confidence DESC LIMIT {limit}"
        return self.conn.execute(sql, params).fetchall()

    def get_neighborhood(self, entity_id: str, hops: int = 2) -> dict:
        """GraphRAG: anchor + topological expansion"""
        nodes = {}
        edges = []

        def expand(current_id: str, depth: int):
            if depth > hops or current_id in nodes:
                return

            entity = self.conn.execute(
                "SELECT * FROM entities WHERE id = ? AND deleted_at IS NULL",
                (current_id,)
            ).fetchone()

            if entity:
                nodes[current_id] = dict(entity)

            rels = self.conn.execute(
                """SELECT * FROM relationships
                   WHERE (source_id = ? OR target_id = ?)
                   AND deleted_at IS NULL""",
                (current_id, current_id)
            ).fetchall()

            for rel in rels:
                edges.append(dict(rel))
                next_id = rel["target_id"] if rel["source_id"] == current_id else rel["source_id"]
                if next_id not in nodes:
                    expand(next_id, depth + 1)

        expand(entity_id, 0)
        return {"nodes": list(nodes.values()), "edges": edges}

    def delete_entity(self, entity_id: str) -> bool:
        self.conn.execute(
            "UPDATE entities SET deleted_at = datetime('now') WHERE id = ?",
            (entity_id,)
        )
        self.conn.execute(
            "UPDATE relationships SET deleted_at = datetime('now') "
            "WHERE source_id = ? OR target_id = ?",
            (entity_id, entity_id)
        )
        self.conn.execute(
            "DELETE FROM entity_fts WHERE rowid IN "
            "(SELECT rowid FROM entities WHERE id = ?)",
            (entity_id,)
        )
        self.conn.commit()
        return True

    def search(self, query: str, limit: int = 20) -> list:
        """FTS5 search across entity names and descriptions"""
        rows = self.conn.execute(
            """SELECT e.*, rank
               FROM entity_fts f
               JOIN entities e ON e.rowid = f.rowid
               WHERE entity_fts MATCH ?
               AND e.deleted_at IS NULL
               ORDER BY rank
               LIMIT ?""",
            (query, limit)
        ).fetchall()
        return [dict(r) for r in rows]

    def get_stats(self) -> dict:
        return {
            "entities": self.conn.execute(
                "SELECT COUNT(*) FROM entities WHERE deleted_at IS NULL"
            ).fetchone()[0],
            "relationships": self.conn.execute(
                "SELECT COUNT(*) FROM relationships WHERE deleted_at IS NULL"
            ).fetchone()[0],
            "types": [
                dict(r) for r in self.conn.execute(
                    "SELECT type, COUNT(*) as count FROM entities "
                    "WHERE deleted_at IS NULL GROUP BY type ORDER BY count DESC"
                ).fetchall()
            ],
        }

    def close(self):
        if self.conn:
            self.conn.close()
