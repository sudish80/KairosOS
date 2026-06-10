"""KairosOS Sliding Context Manager — compresses old terminal/chat history into structured summaries."""

import json
import time
from collections import deque


class SlidingContextManager:
    """Maintains a sliding window of conversation history with automatic summarization."""

    def __init__(self, max_tokens: int = 8192, compression_ratio: float = 0.3):
        self.max_tokens = max_tokens
        self.compression_ratio = compression_ratio
        self.history: deque[dict] = deque()
        self.summaries: list[dict] = []
        self.current_tokens = 0

    def add_entry(self, role: str, content: str, metadata: dict | None = None) -> None:
        entry = {
            "role": role,
            "content": content,
            "timestamp": time.time(),
            "tokens": len(content) // 4,
            "metadata": metadata or {},
        }
        self.history.append(entry)
        self.current_tokens += entry["tokens"]
        self._maybe_compress()

    def _maybe_compress(self) -> None:
        if self.current_tokens <= self.max_tokens:
            return
        target = int(self.max_tokens * self.compression_ratio)
        removed_tokens = 0
        while self.current_tokens - removed_tokens > target and self.history:
            oldest = self.history.popleft()
            removed_tokens += oldest["tokens"]
            self.summaries.append({
                "summary": self._summarize(oldest["content"]),
                "timestamp": oldest["timestamp"],
                "original_tokens": oldest["tokens"],
            })
        self.current_tokens -= removed_tokens

    def _summarize(self, content: str) -> str:
        if len(content) <= 100:
            return content
        return content[:100] + "..."

    def get_context(self) -> list[dict]:
        context = []
        for s in self.summaries[-5:]:
            context.append({"role": "system", "content": f"[Summary: {s['summary']}]"})
        for e in self.history:
            context.append({"role": e["role"], "content": e["content"]})
        return context

    def stats(self) -> dict:
        return {
            "history_entries": len(self.history),
            "summaries": len(self.summaries),
            "current_tokens": self.current_tokens,
            "max_tokens": self.max_tokens,
        }
