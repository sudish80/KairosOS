"""Handler for context-manager."""
import sys
import os
import logging
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))
from ai.context_manager import SlidingContextManager
from .config import Config

logger = logging.getLogger(__name__)

class Handler:
    def __init__(self, config: Config):
        self.config = config
        self.manager = SlidingContextManager()

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
        if method == "add_entry":  # params: role,content,metadata
            return (self.manager.add_entry(params['role'], params['content'], params.get('metadata')), {'ok': True})[1]
        if method == "get_context":
            return {'context': self.manager.get_context()}
        if method == "stats":
            return self.manager.stats()
        if method == "reset":  # params: max_tokens,compression_ratio
            return (setattr(self, 'manager', SlidingContextManager(params.get('max_tokens', 8192), params.get('compression_ratio', 0.3))), {'ok': True})[1]
        raise NotImplementedError(method)
