"""KairosOS Hierarchical DAG Task Scheduler — executes complex OS tasks as parallel DAGs."""

import asyncio
import time
from collections import defaultdict


class DAGTask:
    """A single node in the task dependency graph."""

    def __init__(self, name: str, coro, dependencies: list[str] | None = None, timeout: float = 30):
        self.name = name
        self.coro = coro
        self.dependencies = dependencies or []
        self.timeout = timeout
        self.result = None
        self.error = None
        self.started_at = None
        self.completed_at = None

    @property
    def duration(self):
        if self.started_at and self.completed_at:
            return self.completed_at - self.started_at
        return None


class DAGScheduler:
    """Executes tasks in dependency order, parallelizing where possible."""

    def __init__(self):
        self.tasks: dict[str, DAGTask] = {}
        self._completed = set()
        self._running = set()

    def add_task(self, task: DAGTask) -> None:
        self.tasks[task.name] = task

    def _ready_tasks(self) -> list[DAGTask]:
        ready = []
        for name, task in self.tasks.items():
            if name in self._completed or name in self._running:
                continue
            if all(dep in self._completed for dep in task.dependencies):
                ready.append(task)
        return ready

    async def execute(self) -> dict[str, any]:
        total = len(self.tasks)
        while len(self._completed) < total:
            ready = self._ready_tasks()
            if not ready:
                if self._running:
                    await asyncio.sleep(0.1)
                    continue
                raise RuntimeError("Deadlock in task graph")
            tasks = []
            for task in ready:
                self._running.add(task.name)
                task.started_at = time.time()
                tasks.append(self._run_task(task))
            results = await asyncio.gather(*tasks, return_exceptions=True)
            for task, result in zip(ready, results):
                self._running.discard(task.name)
                task.completed_at = time.time()
                if isinstance(result, Exception):
                    task.error = result
                else:
                    task.result = result
                self._completed.add(task.name)
        return {name: {"result": t.result, "error": str(t.error) if t.error else None, "duration": t.duration}
                for name, t in self.tasks.items()}

    async def _run_task(self, task: DAGTask) -> any:
        try:
            return await asyncio.wait_for(task.coro(), timeout=task.timeout)
        except Exception as e:
            task.error = e
            raise
