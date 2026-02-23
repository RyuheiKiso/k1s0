"""リトライ設定"""

from __future__ import annotations

import random
from dataclasses import dataclass


@dataclass
class RetryConfig:
    """リトライポリシー設定。"""

    max_attempts: int = 3
    initial_delay: float = 0.1
    max_delay: float = 30.0
    multiplier: float = 2.0
    jitter: bool = True

    def compute_delay(self, attempt: int) -> float:
        """リトライ間隔を秒単位で計算する。"""
        base = self.initial_delay * (self.multiplier**attempt)
        capped = min(base, self.max_delay)
        if self.jitter:
            return capped * (0.9 + random.random() * 0.2)
        return capped
