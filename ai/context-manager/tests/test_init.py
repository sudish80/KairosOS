"""Tests for kairos-context-manager AI microservice."""
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ..config import Config
from ..handler import Handler


def test_config_default_socket():
    cfg = Config()
    assert "context" in cfg.socket_path


def test_handler_initialized():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "ping", "id": 1})
    assert "result" in resp or "error" in resp


def test_handler_invalid_jsonrpc():
    h = Handler(Config())
    resp = h.handle({"id": 1})
    assert resp.get("error", {}).get("code") in (-32600,)


def test_handler_context_append():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "append", "params": {"role": "user", "content": "hello"}, "id": 1})
    assert "result" in resp


def test_handler_context_get():
    h = Handler(Config())
    h.handle({"jsonrpc": "2.0", "method": "append", "params": {"role": "user", "content": "hi"}, "id": 1})
    resp = h.handle({"jsonrpc": "2.0", "method": "get", "params": {"n": 5}, "id": 2})
    assert "result" in resp
