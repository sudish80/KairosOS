# CLI entry point for the Knowledge Graph service

import argparse
import json
import sys
from .graph import KnowledgeGraph


def main():
    parser = argparse.ArgumentParser(description="KairosOS Personal Knowledge Graph")
    parser.add_argument("--db", default="~/.hermes/knowledge.db")

    sub = parser.add_subparsers(dest="command")

    sub.add_parser("stats", help="Show graph statistics")

    insert = sub.add_parser("insert", help="Insert an entity")
    insert.add_argument("--type", required=True)
    insert.add_argument("--name", required=True)
    insert.add_argument("--description", default="")
    insert.add_argument("--confidence", type=float, default=1.0)

    search = sub.add_parser("search", help="Search entities")
    search.add_argument("query")

    query_p = sub.add_parser("query", help="GraphRAG query")
    query_p.add_argument("query")
    query_p.add_argument("--strategy", default="auto")

    delete = sub.add_parser("delete", help="Delete an entity")
    delete.add_argument("entity_id")

    consolidate = sub.add_parser("consolidate", help="Run nightly consolidation")

    args = parser.parse_args()

    graph = KnowledgeGraph(args.db)
    graph.connect()

    if args.command == "stats":
        print(json.dumps(graph.get_stats(), indent=2))

    elif args.command == "insert":
        eid = graph.add_entity("", args.type, args.name, args.description, confidence=args.confidence)
        print(json.dumps({"id": eid}))

    elif args.command == "search":
        results = graph.search(args.query)
        print(json.dumps([dict(r) for r in results], indent=2))

    elif args.command == "query":
        from .retrieval import GraphRAG
        rag = GraphRAG(graph)
        result = rag.query(args.query, strategy=args.strategy)
        print(json.dumps(result, indent=2))

    elif args.command == "delete":
        success = graph.delete_entity(args.entity_id)
        print(json.dumps({"deleted": success}))

    elif args.command == "consolidate":
        from .retrieval import GraphRAG
        rag = GraphRAG(graph)
        result = rag.consolidate()
        print(json.dumps(result, indent=2))

    else:
        parser.print_help()

    graph.close()


if __name__ == "__main__":
    main()
