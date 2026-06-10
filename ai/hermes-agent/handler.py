"""Handler for hermes-agent."""
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
        self.sessions = {}

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
        if method == "create_session":
            return (lambda sid='sess_' + str(len(self.sessions) + 1): (self.sessions.__setitem__(sid, {'id': sid, 'ts': time.time()}), {'session_id': sid})[1])()
        if method == "chat":  # params: message
            return {'response': 'KairosOS AI agent ready', 'echo': params.get('message', '')}
        if method == "list_sessions":
            return {'sessions': list(self.sessions.keys())}
        if method == "metrics":
            return {'sessions': len(self.sessions)}
        raise NotImplementedError(method)
