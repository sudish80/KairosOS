"""Handler for skill-evolver."""
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
        self.skills = {}

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
        if method == "register_skill":  # params: name,version
            return (lambda n=params['name']: (self.skills.__setitem__(n, {'name': n, 'version': params.get('version', '1.0.0'), 'calls': 0, 'errors': 0, 'latency_ms': [], 'ts': time.time()}), {'ok': True})[1])()
        if method == "record_execution":  # params: name,error,latency_ms
            return (lambda n=params['name']: (self.skills.setdefault(n, {'calls':0,'errors':0,'latency_ms':[]}), self.skills[n].__setitem__('calls', self.skills[n]['calls']+1), params.get('error') and self.skills[n].__setitem__('errors', self.skills[n]['errors']+1), params.get('latency_ms') and self.skills[n]['latency_ms'].append(params['latency_ms']), {'ok': True})[4])()
        if method == "stats":
            return {n: {k:v for k,v in s.items() if k != 'latency_ms' or v} for n,s in self.skills.items()}
        raise NotImplementedError(method)
