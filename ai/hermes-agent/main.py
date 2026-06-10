#!/usr/bin/env python3
import asyncio
import logging
import argparse
from .config import Config
from .server import serve

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(name)s: %(message)s")
logger = logging.getLogger("kairos.hermes-agent")

def main():
    parser = argparse.ArgumentParser(description="Hermes AI agent entry point")
    parser.add_argument("--socket", default=None)
    args = parser.parse_args()
    cfg = Config(socket_path=args.socket)
    logging.getLogger().setLevel(getattr(logging, cfg.log_level.upper(), logging.INFO))
    logger.info("starting kairos-hermes-agent")
    asyncio.run(serve(cfg))

if __name__ == "__main__":
    main()
