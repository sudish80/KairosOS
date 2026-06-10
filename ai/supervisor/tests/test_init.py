"""Tests for kairos-supervisor AI microservice."""
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ..config import Config
from ..handler import Handler


def test_config_default_socket():
    cfg = Config()
    assert "supervisor" in cfg.socket_path


def test_handler_ping():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "ping", "id": 1})
    assert "result" in resp


def test_handler_heartbeat():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "heartbeat", "id": 1})
    assert resp.get("result", {}).get("status") == "ok"


def test_handler_record_failure():
    h = Handler(Config())
    resp = h.handle({
        "jsonrpc": "2.0", "method": "record_failure",
        "params": {"service": "test", "error": "oops"}, "id": 1
    })
    assert "result" in resp


def test_handler_record_success():
    h = Handler(Config())
    resp = h.handle({
        "jsonrpc": "2.0", "method": "record_success",
        "params": {"service": "test"}, "id": 1
    })
    assert "result" in resp


def test_handler_missing_params():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "record_failure", "id": 1})
    assert resp.get("error", {}).get("code") == -32602
