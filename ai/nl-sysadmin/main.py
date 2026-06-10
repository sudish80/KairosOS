""" Step 4: Natural Language Sysadmin
Connects kairos-llm to MCP to query all daemons in natural language
"""
import asyncio
import json
import logging
import subprocess
from pathlib import Path

logger = logging.getLogger("nl-sysadmin")

MCP_SOCKET = "/var/run/kairos/mcp.sock"
DAEMON_METHODS = {
    "kairos-bpf": ["bpf:get_anomalies", "bpf:get_metrics", "bpf:get_programs"],
    "kairos-recovery": ["recovery:status", "recovery:partitions", "recovery:ota_status"],
    "kairos-mesh": ["mesh:peers", "mesh:status", "mesh:latency"],
    "kairos-db": ["db:stats", "db:memory", "db:queries"],
    "kairos-apply": ["apply:status", "apply:generations", "apply:pending"],
    "git-logger": ["git:log", "git:status", "git:diff"],
    "kairos-tui": ["tui:sessions", "tui:display"],
    "kairos-orchestrator": ["orchestrator:tasks", "orchestrator:queue"],
    "kairos-llm": ["llm:models", "llm:stats", "llm:cache"],
}

SYSTEM_CONTEXT = """
You are KairosOS, an autonomous operating system. You have access to these daemons:
{bpf} — eBPF telemetry and anomaly detection
{recovery} — A/B partition management and OTA updates
{mesh} — WireGuard mesh networking
{db} — Vector database and memory bus
{apply} — Declarative configuration state applier
{git_logger} — /etc versioning via git
{tui} — Terminal UI and framebuffer
{orchestrator} — Multi-agent DAG task scheduler
{llm} — Local LLM runtime

You can query any daemon via MCP protocol and execute remediation via kairos-apply.
Respond with JSON commands to execute.
"""


class McpClient:
    def __init__(self, socket_path=MCP_SOCKET):
        self.socket_path = socket_path

    async def query(self, method: str, params: dict = None) -> dict:
        try:
            reader, writer = await asyncio.open_unix_connection(self.socket_path)
            req = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params or {}})
            writer.write(req.encode() + b"\n")
            await writer.drain()
            data = await reader.read(65536)
            writer.close()
            return json.loads(data)
        except Exception as e:
            return {"error": str(e)}

    async def query_all_daemons(self) -> dict:
        results = {}
        for daemon, methods in DAEMON_METHODS.items():
            for method in methods[:2]:
                try:
                    result = await self.query(method)
                    results[f"{daemon}:{method}"] = result
                except Exception:
                    pass
        return results


class NlSysadmin:
    def __init__(self):
        self.mcp = McpClient()
        self.context = SYSTEM_CONTEXT.format(
            bpf="kairos-bpf",
            recovery="kairos-recovery",
            mesh="kairos-mesh",
            db="kairos-db",
            apply="kairos-apply",
            git_logger="git-logger",
            tui="kairos-tui",
            orchestrator="kairos-orchestrator",
            llm="kairos-llm",
        )

    def format_context(self, daemon_data: dict) -> str:
        lines = ["Current system state:\n"]
        for key, value in daemon_data.items():
            if isinstance(value, dict) and "error" not in value:
                lines.append(f"  {key}: OK")
            elif isinstance(value, dict) and "error" in value:
                lines.append(f"  {key}: ERROR - {value['error']}")
        return "\n".join(lines)

    async def process_query(self, natural_language: str) -> str:
        daemon_data = await self.mcp.query_all_daemons()
        state = self.format_context(daemon_data)

        prompt = f"{self.context}\n\n{state}\n\nUser query: {natural_language}\n\nRespond with:"

        try:
            llm_result = await self.mcp.query("llm:generate", {
                "prompt": prompt,
                "max_tokens": 500,
                "temperature": 0.1,
            })
            response = llm_result.get("result", {}).get("text", "No response from LLM")

            commands = self.extract_commands(response)
            for cmd in commands:
                await self.mcp.query(cmd["method"], cmd.get("params", {}))

            return response
        except Exception as e:
            return f"Error processing query: {e}"

    def extract_commands(self, text: str) -> list[dict]:
        commands = []
        for line in text.split("\n"):
            line = line.strip()
            if line.startswith("!") and ":" in line:
                parts = line[1:].split(" ", 1)
                method = parts[0]
                params = {}
                if len(parts) > 1:
                    try:
                        params = json.loads(parts[1])
                    except json.JSONDecodeError:
                        params = {"raw": parts[1]}
                commands.append({"method": method, "params": params})
        return commands


async def main():
    logging.basicConfig(level=logging.INFO)
    ns = NlSysadmin()
    print("Natural Language Sysadmin ready. Type 'exit' to quit.")
    while True:
        try:
            query = input("> ")
            if query.lower() in ("exit", "quit"):
                break
            response = await ns.process_query(query)
            print(f"\n{response}\n")
        except KeyboardInterrupt:
            break


if __name__ == "__main__":
    asyncio.run(main())
