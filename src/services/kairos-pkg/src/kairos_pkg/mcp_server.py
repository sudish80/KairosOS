# MCP protocol server for the Knowledge Graph
# Exposes kg:// resources and tools to any MCP-compatible agent

import json
import asyncio
from typing import Optional
from .graph import KnowledgeGraph
from .retrieval import GraphRAG


class KGMcpServer:
    def __init__(self, graph: KnowledgeGraph, socket_path: str = "/run/kairos/pkg-mcp.sock"):
        self.graph = graph
        self.rag = GraphRAG(graph)
        self.socket_path = socket_path

    async def start(self):
        print(f"PKG MCP server starting on {self.socket_path}")
        # Resources:
        #   kg://entity/<id>
        #   kg://search?q=<query>
        #   kg://stats
        #   kg://neighborhood/<id>
        # Tools:
        #   kg-query(query, strategy)
        #   kg-insert(type, name, description)
        #   kg-delete(entity_id)
        #   kg-consolidate

        while True:
            await asyncio.sleep(3600)

    async def handle_resource(self, uri: str) -> str:
        parts = uri.replace("kg://", "").split("/")

        if parts[0] == "stats":
            return json.dumps(self.graph.get_stats(), indent=2)

        elif parts[0] == "entity" and len(parts) > 1:
            entity = self.graph.conn.execute(
                "SELECT * FROM entities WHERE id = ? AND deleted_at IS NULL",
                (parts[1],)
            ).fetchone()
            if entity:
                return json.dumps(dict(entity), indent=2)
            return json.dumps({"error": "Entity not found"})

        elif parts[0] == "neighborhood" and len(parts) > 1:
            hood = self.graph.get_neighborhood(parts[1])
            return json.dumps(hood, indent=2)

        elif parts[0] == "search":
            query = parts[1] if len(parts) > 1 else ""
            results = self.graph.search(query)
            return json.dumps(results, indent=2)

        return json.dumps({"error": f"Unknown resource: {uri}"})

    async def handle_tool(self, name: str, args: dict) -> dict:
        if name == "kg-query":
            query = args.get("query", "")
            strategy = args.get("strategy", "auto")
            result = self.rag.query(query, strategy=strategy)
            return {"content": [{"type": "text", "text": json.dumps(result, indent=2)}]}

        elif name == "kg-insert":
            entity_id = self.graph.add_entity(
                entity_id=args.get("id", ""),
                type_=args.get("type", "note"),
                name=args.get("name", ""),
                description=args.get("description", ""),
                metadata=args.get("metadata"),
                confidence=args.get("confidence", 1.0),
            )
            return {"content": [{"type": "text", "text": json.dumps({"id": entity_id})}]}

        elif name == "kg-delete":
            success = self.graph.delete_entity(args.get("entity_id", ""))
            return {"content": [{"type": "text", "text": json.dumps({"deleted": success})}]}

        elif name == "kg-consolidate":
            result = self.rag.consolidate()
            return {"content": [{"type": "text", "text": json.dumps(result, indent=2)}]}

        elif name == "kg-stats":
            stats = self.graph.get_stats()
            return {"content": [{"type": "text", "text": json.dumps(stats, indent=2)}]}

        return {"content": [{"type": "text", "text": json.dumps({"error": f"Unknown tool: {name}"})}]}
