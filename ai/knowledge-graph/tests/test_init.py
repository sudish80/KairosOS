"""Tests for kairos-knowledge-graph AI microservice."""
import json
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ..config import Config
from ..handler import Handler


def test_config_default_socket():
    cfg = Config()
    assert "knowledge" in cfg.socket_path


def test_add_entity():
    h = Handler(Config())
    resp = h.handle({
        "jsonrpc": "2.0", "method": "add_entity",
        "params": {"id": "e1", "type": "test", "properties": {"name": "foo"}}, "id": 1
    })
    assert "result" in resp


def test_add_duplicate_entity():
    h = Handler(Config())
    h.handle({"jsonrpc": "2.0", "method": "add_entity",
              "params": {"id": "e2", "type": "test", "properties": {}}, "id": 1})
    resp = h.handle({"jsonrpc": "2.0", "method": "add_entity",
                     "params": {"id": "e2", "type": "test", "properties": {}}, "id": 2})
    assert resp.get("error", {}).get("code") == -32602


def test_add_relation():
    h = Handler(Config())
    h.handle({"jsonrpc": "2.0", "method": "add_entity",
              "params": {"id": "a", "type": "node", "properties": {}}, "id": 1})
    h.handle({"jsonrpc": "2.0", "method": "add_entity",
              "params": {"id": "b", "type": "node", "properties": {}}, "id": 2})
    resp = h.handle({"jsonrpc": "2.0", "method": "add_relation",
                     "params": {"subject": "a", "predicate": "connects_to", "object": "b"}, "id": 3})
    assert "result" in resp


def test_search():
    h = Handler(Config())
    h.handle({"jsonrpc": "2.0", "method": "add_entity",
              "params": {"id": "search1", "type": "test", "properties": {"name": "alice"}}, "id": 1})
    resp = h.handle({"jsonrpc": "2.0", "method": "search",
                     "params": {"query": "alice"}, "id": 2})
    assert "result" in resp


def test_missing_id_on_add_entity():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "add_entity",
                     "params": {"type": "test", "properties": {}}, "id": 1})
    assert resp.get("error", {}).get("code") == -32602
