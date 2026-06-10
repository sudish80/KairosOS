"""KairosOS Confidence Thresholding — fall back to user if agent confidence is too low."""

import re


class ConfidenceScorer:
    """Scores agent response confidence based on signal quality indicators."""

    UNCERTAINTY_PATTERNS = [
        r"(?i)\b(maybe|perhaps|not sure|uncertain|could be|might be|i think|possibly|probably)\b",
        r"(?i)\b(not enough information|insufficient data|i don't know|i\'m not sure)\b",
        r"\?$",
        r"(?i)\b(would you like me to|should i|do you want)\b",
    ]

    def __init__(self, thresholds: dict[str, float] | None = None):
        self.thresholds = thresholds or {
            "auto_execute": 0.8,
            "suggest": 0.5,
            "ask_user": 0.0,
        }

    def score(self, response: str, context: dict | None = None) -> dict:
        """Score the confidence of a response. Returns score 0.0-1.0 and action."""
        score = 1.0

        # Penalize for uncertainty language
        for pattern in self.UNCERTAINTY_PATTERNS:
            if re.search(pattern, response):
                score -= 0.15

        # Penalize for very short responses
        if len(response.split()) < 3:
            score -= 0.2

        # Penalize for question-like responses
        if response.strip().endswith("?"):
            score -= 0.1

        # Boost for concrete data
        if re.search(r'\d+[.%]|\b[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}', response):
            score += 0.1

        score = max(0.0, min(1.0, score))

        if score >= self.thresholds["auto_execute"]:
            action = "auto_execute"
        elif score >= self.thresholds["suggest"]:
            action = "suggest"
        else:
            action = "ask_user"

        return {"score": round(score, 3), "action": action, "thresholds": self.thresholds}

    def should_auto_execute(self, response: str) -> bool:
        return self.score(response)["action"] == "auto_execute"
