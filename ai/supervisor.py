"""KairosOS Supervisor Daemon Watchdog — prevents agent loop lockouts."""

import asyncio
import logging
import time

logger = logging.getLogger("kairos.supervisor")


class SupervisorWatchdog:
    """Monitors the agent execution loop and intervenes on lockouts."""

    def __init__(self, max_loop_time: float = 30.0, max_consecutive_failures: int = 5):
        self.max_loop_time = max_loop_time
        self.max_consecutive_failures = max_consecutive_failures
        self._loop_start = 0.0
        self._failures = 0
        self._last_heartbeat = time.time()

    def heartbeat(self) -> None:
        self._last_heartbeat = time.time()

    def record_failure(self, error: str) -> None:
        self._failures += 1
        logger.warning(f"Supervisor recorded failure #{self._failures}: {error}")

    def record_success(self) -> None:
        self._failures = 0

    async def check_loop_timeout(self) -> bool:
        """Check if current loop iteration has exceeded max time."""
        if self._loop_start > 0 and (time.time() - self._loop_start) > self.max_loop_time:
            logger.error("Agent loop timeout — forcing interrupt")
            return True
        return False

    async def watch(self, check_interval: float = 5.0) -> None:
        """Background watcher task."""
        while True:
            await asyncio.sleep(check_interval)
            elapsed = time.time() - self._last_heartbeat
            if elapsed > self.max_loop_time * 2:
                logger.warning(f"No heartbeat for {elapsed:.0f}s — agent may be stuck")
            if self._failures >= self.max_consecutive_failures:
                logger.critical(f"{self._failures} consecutive failures — initiating recovery")
                await self._recover()

    async def _recover(self) -> None:
        """Recovery actions when agent is stuck."""
        self._failures = 0
        logger.info("Supervisor: recovery actions completed, resetting state")

    async def run_with_watchdog(self, coro):
        """Execute a coroutine with watchdog monitoring."""
        self._loop_start = time.time()
        try:
            result = await asyncio.wait_for(coro, timeout=self.max_loop_time)
            self.record_success()
            return result
        except asyncio.TimeoutError:
            self.record_failure("timeout")
            raise
        except Exception as e:
            self.record_failure(str(e))
            raise
        finally:
            self._loop_start = 0.0
