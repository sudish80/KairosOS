"""Handler for confidence."""
import sys
import os
import logging
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))
from ai.confidence import ConfidenceScorer
from .config import Config

logger = logging.getLogger(__name__)

class Handler:
    def __init__(self, config: Config):
        self.config = config
        self.scorer = ConfidenceScorer()

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
        if method == "score":  # params: response
            return self.scorer.score(params.get('response', ''), params.get('context'))
        if method == "should_auto_execute":  # params: response
            return {'auto_execute': self.scorer.should_auto_execute(params.get('response', ''))}
        if method == "set_thresholds":  # params: thresholds
            return (lambda: (setattr(self, 'scorer', ConfidenceScorer(params.get('thresholds', {}))), {'ok': True})[1])()
        if method == "metrics":
            return {'model': 'regex', 'patterns': len(ConfidenceScorer.UNCERTAINTY_PATTERNS)}
        raise NotImplementedError(method)
