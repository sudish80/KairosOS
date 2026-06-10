"""JSON-RPC 2.0 MCP server over Unix socket."""
import asyncio, json, logging, signal, traceback, os
from .config import Config
from .handler import Handler

logger = logging.getLogger(__name__)

async def _handle(reader, writer, handler):
    while True:
        try:
            line = await reader.readline()
            if not line:
                break
            req = json.loads(line.decode().strip())
            resp = await handler.handle(req)
            writer.write((json.dumps(resp) + "\n").encode())
            await writer.drain()
        except json.JSONDecodeError as e:
            err = {"jsonrpc": "2.0", "error": {"code": -32700, "message": str(e)}, "id": None}
            writer.write((json.dumps(err) + "\n").encode())
            await writer.drain()
        except Exception as e:
            logger.error("unhandled: %s", traceback.format_exc())
            err = {"jsonrpc": "2.0", "error": {"code": -32603, "message": str(e)}, "id": None}
            writer.write((json.dumps(err) + "\n").encode())
            await writer.drain()
    try:
        writer.close()
    except Exception:
        pass

async def serve(cfg):
    handler = Handler(cfg)
    try:
        os.unlink(cfg.socket_path)
    except FileNotFoundError:
        pass
    server = await asyncio.start_unix_server(lambda r, w: _handle(r, w, handler), path=cfg.socket_path)
    logger.info("listening on %s", cfg.socket_path)
    loop = asyncio.get_running_loop()
    stop = loop.create_future()
    loop.add_signal_handler(signal.SIGTERM, lambda: stop.set_result(None))
    loop.add_signal_handler(signal.SIGINT, lambda: stop.set_result(None))
    async with server:
        await stop
    logger.info("shutdown complete")
