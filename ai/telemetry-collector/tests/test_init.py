"""Tests for kairos-telemetry-collector AI microservice."""
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ..config import Config
from ..handler import Handler


def test_config_default_socket():
    cfg = Config()
    assert "telemetry" in cfg.socket_path


def test_record_metric():
    h = Handler(Config())
    resp = h.handle({
        "jsonrpc": "2.0", "method": "record",
        "params": {"name": "test_metric", "value": 42.0, "tags": {"host": "localhost"}}, "id": 1
    })
    assert "result" in resp


def test_record_invalid_value():
    h = Handler(Config())
    resp = h.handle({
        "jsonrpc": "2.0", "method": "record",
        "params": {"name": "bad", "value": "not_a_number"}, "id": 1
    })
    assert resp.get("error", {}).get("code") == -32602


def test_query_empty():
    h = Handler(Config())
    resp = h.handle({
        "jsonrpc": "2.0", "method": "query",
        "params": {"name": "nonexistent"}, "id": 1
    })
    assert "result" in resp


def test_record_and_query():
    h = Handler(Config())
    h.handle({"jsonrpc": "2.0", "method": "record",
              "params": {"name": "cpu_temp", "value": 75.0, "tags": {"core": "0"}}, "id": 1})
    resp = h.handle({"jsonrpc": "2.0", "method": "query",
                     "params": {"name": "cpu_temp"}, "id": 2})
    assert "result" in resp
    assert len(resp["result"].get("values", [])) > 0


def test_stats():
    h = Handler(Config())
    h.handle({"jsonrpc": "2.0", "method": "record",
              "params": {"name": "m1", "value": 1.0, "tags": {}}, "id": 1})
    h.handle({"jsonrpc": "2.0", "method": "record",
              "params": {"name": "m1", "value": 2.0, "tags": {}}, "id": 2})
    resp = h.handle({"jsonrpc": "2.0", "method": "stats",
                     "params": {"name": "m1"}, "id": 3})
    assert "result" in resp
    stats = resp["result"]
    assert stats.get("count", 0) >= 2


def test_missing_name_on_record():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "record",
                     "params": {"value": 1.0}, "id": 1})
    assert resp.get("error", {}).get("code") == -32602
