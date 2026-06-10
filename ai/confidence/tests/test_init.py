"""Tests for kairos-confidence AI microservice."""
import json
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ..config import Config
from ..handler import Handler


def test_config_default_socket():
    cfg = Config()
    assert "confidence" in cfg.socket_path


def test_handler_initialized():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "ping", "id": 1})
    result = json.loads(json.dumps(resp))
    assert "result" in result or "error" in result


def test_handler_missing_method():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "id": 1})
    assert resp.get("error", {}).get("code") == -32600


def test_handler_unsupported_method():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "nonexistent", "id": 1})
    assert resp.get("error", {}).get("code") == -32601


def test_config_env_override():
    os.environ["KAIROS_CONFIDENCE_ENDPOINT"] = "/tmp/test.sock"
    cfg = Config()
    assert cfg.socket_path == "/tmp/test.sock"
    del os.environ["KAIROS_CONFIDENCE_ENDPOINT"]
