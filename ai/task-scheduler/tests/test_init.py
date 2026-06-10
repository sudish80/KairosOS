"""Tests for kairos-task-scheduler AI microservice."""
import json
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ..config import Config
from ..handler import Handler


def test_config_default_socket():
    cfg = Config()
    assert "scheduler" in cfg.socket_path


def test_add_task():
    h = Handler(Config())
    resp = h.handle({
        "jsonrpc": "2.0", "method": "add_task",
        "params": {"name": "task1", "command": "echo hello", "deps": []}, "id": 1
    })
    assert "result" in resp


def test_add_task_with_deps():
    h = Handler(Config())
    h.handle({"jsonrpc": "2.0", "method": "add_task",
              "params": {"name": "a", "command": "echo a", "deps": []}, "id": 1})
    resp = h.handle({"jsonrpc": "2.0", "method": "add_task",
                     "params": {"name": "b", "command": "echo b", "deps": ["a"]}, "id": 2})
    assert "result" in resp


def test_add_task_cyclic_dependency():
    h = Handler(Config())
    h.handle({"jsonrpc": "2.0", "method": "add_task",
              "params": {"name": "x", "command": "echo x", "deps": ["y"]}, "id": 1})
    h.handle({"jsonrpc": "2.0", "method": "add_task",
              "params": {"name": "y", "command": "echo y", "deps": ["x"]}, "id": 2})
    resp = h.handle({"jsonrpc": "2.0", "method": "execute", "id": 3})
    assert "error" in resp  # cyclic graph should fail


def test_execute_empty():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "execute", "id": 1})
    assert "result" in resp


def test_status_no_tasks():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "status", "id": 1})
    assert "result" in resp


def test_missing_command():
    h = Handler(Config())
    resp = h.handle({"jsonrpc": "2.0", "method": "add_task",
                     "params": {"name": "bad", "deps": []}, "id": 1})
    assert resp.get("error", {}).get("code") == -32602
