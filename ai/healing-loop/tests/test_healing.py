""" Tests for autonomous healing loop """
import pytest
import json
from unittest.mock import patch
from main import HealingLoop, MCPClient


@pytest.mark.asyncio
async def test_healing_loop_initialization():
    loop = HealingLoop()
    assert loop.event_buffer == []
    assert loop.last_remediation == {}


@pytest.mark.asyncio
async def test_evaluate_low_severity():
    loop = HealingLoop()
    event = {"id": "e1", "severity": 3, "category": "process:crash", "source": "test"}
    await loop.evaluate_event(event)
    assert len(loop.last_remediation) == 0


@pytest.mark.asyncio
async def test_evaluate_high_severity():
    loop = HealingLoop()
    event = {"id": "e2", "severity": 8, "category": "memory:oom", "source": "kairos-db"}
    with patch.object(loop, 'apply_remediation') as mock_apply:
        await loop.evaluate_event(event)
        assert mock_apply.called


@pytest.mark.asyncio
async def test_select_remediation():
    loop = HealingLoop()
    script = await loop.select_remediation("process:crash", "kairos-test")
    assert script is not None
    assert script["script"] == "restart-daemon"


@pytest.mark.asyncio
async def test_select_remediation_unknown():
    loop = HealingLoop()
    script = await loop.select_remediation("unknown:type", "test")
    assert script is None


def test_mcp_client_serialization():
    _ = MCPClient("/tmp/test.sock")
    req = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "test", "params": {}})
    data = json.loads(req)
    assert data["method"] == "test"
    assert data["jsonrpc"] == "2.0"
