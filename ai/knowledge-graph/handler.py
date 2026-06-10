"""Handler for knowledge-graph."""
import sys, os, json, time, logging
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from .config import Config

logger = logging.getLogger(__name__)

class Handler:
    def __init__(self, config: Config):
        self.config = config
        self.entities = {}; self.relations = []

    async def handle(self, req: dict) -> dict:
        method = req.get("method", "")
        rid = req.get("id")
        try:
            result = self._dispatch(method, req.get("params", {}))
            return {"jsonrpc": "2.0", "result": result, "id": rid}
        except NotImplementedError:
            return {"jsonrpc": "2.0", "error": {"code": -32601, "message": f"method not found: {method}"}, "id": rid}
        except Exception as e:
            logger.error("method %s: %s", method, e)
            return {"jsonrpc": "2.0", "error": {"code": -32603, "message": str(e)}, "id": rid}

    def _dispatch(self, method: str, params: dict) -> dict:
        if method == "add_entity":  # params: id,type,properties
            return (lambda eid=params.get('id', str(time.time())): (self.entities.__setitem__(eid, {'id': eid, 'type': params.get('type', 'generic'), 'properties': params.get('properties', {}), 'ts': time.time()}), {'entity_id': eid})[1])()
        if method == "add_relation":  # params: source,target,type
            return (lambda: (self.relations.append({'source': params['source'], 'target': params['target'], 'type': params.get('type', 'related'), 'ts': time.time()}), {'ok': True})[1])()
        if method == "search":  # params: query
            return (lambda q=params.get('query', '').lower(): {'results': [e for eid,e in self.entities.items() if q in eid.lower() or q in str(e.get('properties',{})).lower()][:50]})()
        if method == "stats":
            return {'entities': len(self.entities), 'relations': len(self.relations)}
        raise NotImplementedError(method)
