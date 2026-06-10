"""Handler for supervisor."""
import sys, os, json, time, logging
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))
from ai.supervisor import SupervisorWatchdog
from .config import Config

logger = logging.getLogger(__name__)

class Handler:
    def __init__(self, config: Config):
        self.config = config
        self.watchdog = SupervisorWatchdog()
        self._watch_task: object = None

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
        if method == "heartbeat":
            return (self.watchdog.heartbeat(), {'ok': True})[1]
        if method == "record_failure":  # params: error
            return (self.watchdog.record_failure(params.get('error', 'unknown')), {'failures': self.watchdog._failures})[1]
        if method == "record_success":
            return (self.watchdog.record_success(), {'ok': True})[1]
        if method == "status":
            return {'failures': self.watchdog._failures, 'last_heartbeat': getattr(self.watchdog, '_last_heartbeat', 0)}
        if method == "start_watch":  # params: interval
            return (lambda: (setattr(self, '_watch_task', self._watch_task or __import__('asyncio').create_task(self.watchdog.watch(params.get('interval', 5.0)))), {'watching': True})[1])()
        raise NotImplementedError(method)
