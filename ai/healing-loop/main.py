""" Step 2: Autonomous Healing Loop
Bridges eBPF anomaly events -> MCP -> knowledge graph -> skill-evolver -> supervisor -> kairos-apply
"""
import asyncio
import json
import logging
import time
from pathlib import Path

SOCKET_PATH = "/var/run/kairos/mcp.sock"
HEALING_SCRIPTS_DIR = Path("/var/lib/kairos/healing")

logger = logging.getLogger("healing-loop")


class MCPClient:
    def __init__(self, socket_path: str = SOCKET_PATH):
        self.socket_path = socket_path

    async def call(self, method: str, params: dict = None) -> dict:
        reader, writer = await asyncio.open_unix_connection(self.socket_path)
        req = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params or {}})
        writer.write(req.encode() + b"\n")
        await writer.drain()
        data = await reader.read(65536)
        writer.close()
        return json.loads(data)


class HealingLoop:
    def __init__(self):
        self.mcp = MCPClient()
        self.event_buffer: list[dict] = []
        self.last_remediation: dict[str, float] = {}

    async def poll_anomalies(self):
        """Poll kairos-bpf anomaly events via MCP"""
        try:
            result = await self.mcp.call("bpf:get_anomalies", {"count": 10})
            events = result.get("result", {}).get("anomalies", [])
            for event in events:
                await self.evaluate_event(event)
        except Exception as e:
            logger.error(f"Failed to poll anomalies: {e}")

    async def evaluate_event(self, event: dict):
        severity = event.get("severity", 0)
        category = event.get("category", "unknown")
        source = event.get("source", "unknown")
        event_id = event.get("id", str(time.time()))

        # Debounce: skip if same source recently remediated
        if source in self.last_remediation:
            if time.time() - self.last_remediation[source] < 300:
                logger.debug(f"Skipping {source}, recently remediated")
                return

        if severity >= 7:
            logger.warning(f"High severity anomaly: {event_id} ({category}) on {source}")
            script = await self.select_remediation(category, source)
            if script:
                await self.apply_remediation(event_id, source, script)
                self.last_remediation[source] = time.time()

    async def select_remediation(self, category: str, source: str) -> dict | None:
        knowledge = {
            "process:crash": {"script": "restart-daemon", "timeout": 30},
            "memory:oom": {"script": "oom-remediate", "timeout": 60},
            "network:drop": {"script": "network-reset", "timeout": 45},
            "disk:full": {"script": "logrotate-force", "timeout": 30},
            "thermal:throttle": {"script": "throttle-recover", "timeout": 120},
        }
        return knowledge.get(category)

    async def apply_remediation(self, event_id: str, target: str, script: dict):
        logger.info(f"Applying remediation {script['script']} to {target}")
        try:
            result = await self.mcp.call("apply:execute", {
                "event_id": event_id,
                "target": target,
                "script": script["script"],
                "timeout": script["timeout"],
            })
            logger.info(f"Remediation result: {result}")
        except Exception as e:
            logger.error(f"Remediation failed: {e}")

    async def run(self):
        logger.info("Starting autonomous healing loop")
        while True:
            await self.poll_anomalies()
            await asyncio.sleep(5)


async def main():
    logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(name)s] %(levelname)s: %(message)s")
    loop = HealingLoop()
    await loop.run()


if __name__ == "__main__":
    asyncio.run(main())
