"""Tests for kairos-hermes-agent AI microservice."""
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ..config import Config
from ..handler import Handler


def test_config_default_socket():
    cfg = Config()
    assert "hermes" in cfg.socket_path


def test_create_session():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "create_session", "id": 1})
    assert "result" in resp
    session_id = resp["result"].get("session_id")
    assert session_id is not None


def test_chat_invalid_session():
    h = Handler(Config())
    resp = h.handle({
        "jsonrpc": "2.0", "method": "chat",
        "params": {"session_id": "nonexistent", "message": "hello"}, "id": 1
    })
    assert resp.get("error", {}).get("code") == -32602


def test_list_sessions_empty():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "list_sessions", "id": 1})
    assert "result" in resp


def test_create_and_chat():
    h = Handler(Config())
    create = h.handle({"jsonrpc": "2.0", "method": "create_session", "id": 1})
    sid = create["result"]["session_id"]
    chat = h.handle({
        "jsonrpc": "2.0", "method": "chat",
        "params": {"session_id": sid, "message": "hello"}, "id": 2
    })
    assert "result" in chat
