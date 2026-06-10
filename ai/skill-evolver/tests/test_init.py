"""Tests for kairos-skill-evolver AI microservice."""
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ..config import Config
from ..handler import Handler


def test_config_default_socket():
    cfg = Config()
    assert "evolver" in cfg.socket_path


def test_create_skill():
    h = Handler(Config())
    resp = h.handle({
        "jsonrpc": "2.0", "method": "create_skill",
        "params": {"name": "test_skill", "prompt": "do something"}, "id": 1
    })
    assert "result" in resp


def test_create_duplicate_skill():
    h = Handler(Config())
    h.handle({"jsonrpc": "2.0", "method": "create_skill",
              "params": {"name": "dup", "prompt": "x"}, "id": 1})
    resp = h.handle({"jsonrpc": "2.0", "method": "create_skill",
                     "params": {"name": "dup", "prompt": "y"}, "id": 2})
    assert resp.get("error", {}).get("code") == -32602


def test_get_skill():
    h = Handler(Config())
    h.handle({"jsonrpc": "2.0", "method": "create_skill",
              "params": {"name": "getme", "prompt": "test"}, "id": 1})
    resp = h.handle({"jsonrpc": "2.0", "method": "get_skill",
                     "params": {"name": "getme"}, "id": 2})
    assert "result" in resp


def test_get_nonexistent_skill():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "get_skill",
                     "params": {"name": "nope"}, "id": 1})
    assert resp.get("error", {}).get("code") == -32602


def test_list_skills():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "list_skills", "id": 1})
    assert "result" in resp
