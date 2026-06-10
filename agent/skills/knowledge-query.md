# Knowledge Query Skill

## Description
Query the KairosOS Personal Knowledge Graph (kairos-pkg) using hybrid GraphRAG retrieval — combines vector similarity, full-text search, and graph neighborhood expansion for personal semantic memory.

## Triggers
- User asks about past work, documents, or learned information
- Agent needs to recall context from previous sessions
- Scheduled knowledge consolidation

## Actions

### query(q, top_k=10)
Search the knowledge graph:
- `q`: natural language query
- `top_k`: maximum results
- Returns ranked results with entity context and source provenance

### query_entity(entity_type=None, name=None)
Look up a specific entity in the knowledge graph:
- Entity types: url, file_path, package, ip_address, email, custom
- Returns all relationships and metadata for the matched entity

### expand(entity_id, hops=1)
Neighborhood expansion from an entity:
- `hops`: depth of relationship traversal (1-3)
- Returns subgraph centered on the entity

### store_note(content, tags=None, source=None)
Store a new note/document in the knowledge graph:
- Content is automatically analyzed for entity extraction
- Tags and source for provenance tracking

### nightly_consolidate()
Trigger knowledge consolidation:
- Deduplicate entities
- Merge weak relationships
- Rebuild FTS index

### stats()
Return knowledge graph statistics:
- Total entities, relationships, documents
- FTS index size, vector dimension count
- Last consolidation timestamp

## Example
User: "What was that URL I found about Rust eBPF last week?"
Agent: *calls query_entity with type=url, name containing "rust" and "ebpf"*

## Dependencies
- kairos-pkg service (MCP endpoint)
- SQLite with sqlite-vec extension
