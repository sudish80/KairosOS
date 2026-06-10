"""Handler for task-scheduler."""
import sys
import os
import logging
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))
from ai.task_scheduler import DAGScheduler, DAGTask
from .config import Config

logger = logging.getLogger(__name__)

class Handler:
    def __init__(self, config: Config):
        self.config = config
        self.scheduler = DAGScheduler()

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
        if method == "add_task":  # params: name,dependencies,timeout,payload
            return (lambda n=params['name'], deps=params.get('dependencies',[]), to=params.get('timeout',30): (self.scheduler.add_task(DAGTask(n, lambda p=params.get('payload',{}): p, deps, to)), {'ok': True})[1])()
        if method == "execute":
            return __import__('asyncio').run(self.scheduler.execute())
        if method == "status":
            return {'tasks': len(self.scheduler.tasks), 'completed': len(self.scheduler._completed), 'running': len(self.scheduler._running)}
        if method == "reset":
            return (setattr(self, 'scheduler', DAGScheduler()), {'ok': True})[1]
        raise NotImplementedError(method)
