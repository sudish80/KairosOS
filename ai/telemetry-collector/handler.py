"""Handler for telemetry-collector."""
import sys
import os
import time
import logging
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from .config import Config

logger = logging.getLogger(__name__)

class Handler:
    def __init__(self, config: Config):
        self.config = config
        self.metrics_store = {}

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
        if method == "record":  # params: source,metrics
            return (lambda s=params.get('source','unknown'), m=params.get('metrics',{}): (self.metrics_store.setdefault(s, []).append({'ts': time.time(), 'metrics': m}), self.metrics_store.__setitem__(s, self.metrics_store[s][-1000:]), {'ok': True})[2])()
        if method == "query":  # params: source,limit
            return (lambda s=params.get('source'), limit=params.get('limit',100): {s: self.metrics_store.get(s, [])[-limit:]} if s else {k: v[-limit:] for k,v in self.metrics_store.items()})()
        if method == "stats":
            return {s: len(v) for s,v in self.metrics_store.items()}
        raise NotImplementedError(method)
