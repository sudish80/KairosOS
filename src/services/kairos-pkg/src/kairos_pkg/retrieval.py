# GraphRAG retrieval: anchor search + topological expansion
# Multi-strategy retrieval supporting different query types

from typing import Optional
from .graph import KnowledgeGraph


class GraphRAG:
    def __init__(self, graph: KnowledgeGraph):
        self.graph = graph

    def query(self, query: str, strategy: str = "auto", limit: int = 10) -> dict:
        strategies = {
            "auto": self._auto_strategy,
            "anchor": self._anchor_expand,
            "fts": self._fts_search,
        }

        resolver = strategies.get(strategy, self._auto_strategy)
        return resolver(query, limit)

    def _auto_strategy(self, query: str, limit: int) -> dict:
        # Auto-detect best strategy based on query structure
        if query.startswith("what") or query.startswith("who"):
            return self._anchor_expand(query, limit)
        elif query.startswith("find") or query.startswith("search"):
            return self._fts_search(query, limit)
        else:
            return self._anchor_expand(query, limit)

    def _anchor_expand(self, query: str, limit: int) -> dict:
        # Step 1: Find anchor entities via FTS
        anchors = self.graph.search(query, limit=5)

        if not anchors:
            return {"nodes": [], "edges": [], "query": query}

        # Step 2: Topological expansion from each anchor
        all_nodes = {}
        all_edges = []

        for anchor in anchors:
            hood = self.graph.get_neighborhood(anchor["id"], hops=2)
            for node in hood["nodes"]:
                if node["id"] not in all_nodes:
                    all_nodes[node["id"]] = node
            all_edges.extend(hood["edges"])

        return {
            "nodes": list(all_nodes.values())[:limit],
            "edges": all_edges,
            "query": query,
            "anchors": [a["name"] for a in anchors],
        }

    def _fts_search(self, query: str, limit: int) -> dict:
        results = self.graph.search(query, limit=limit)
        return {
            "nodes": results,
            "edges": [],
            "query": query,
        }

    def consolidate(self):
        """Nightly consolidation: deduplicate entities, merge weak edges"""
        stats = self.graph.get_stats()

        # Find duplicate entities by name
        conn = self.graph.conn
        duplicates = conn.execute(
            """SELECT name, COUNT(*) as cnt, GROUP_CONCAT(id) as ids
               FROM entities WHERE deleted_at IS NULL
               GROUP BY name HAVING cnt > 1"""
        ).fetchall()

        merged_count = 0
        for dup in duplicates:
            ids = dup["ids"].split(",")
            primary = ids[0]
            for duplicate_id in ids[1:]:
                # Move all relationships to primary
                conn.execute(
                    "UPDATE relationships SET source_id = ? WHERE source_id = ?",
                    (primary, duplicate_id)
                )
                conn.execute(
                    "UPDATE relationships SET target_id = ? WHERE target_id = ?",
                    (primary, duplicate_id)
                )
                # Delete duplicate
                conn.execute(
                    "UPDATE entities SET deleted_at = datetime('now') WHERE id = ?",
                    (duplicate_id,)
                )
                merged_count += 1

        conn.commit()
        return {
            "before": stats,
            "after": self.graph.get_stats(),
            "duplicates_merged": merged_count,
        }
