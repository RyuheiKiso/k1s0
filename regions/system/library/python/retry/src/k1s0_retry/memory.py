"""CircuitBreaker 実装"""

from __future__ import annotations

import time
from enum import Enum


class CircuitBreakerState(Enum):
    """サーキットブレーカーの状態。"""

    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


class CircuitBreaker:
    """サーキットブレーカー。"""

    def __init__(
        self,
        failure_threshold: int = 5,
        success_threshold: int = 2,
        timeout: float = 30.0,
    ) -> None:
        self.failure_threshold = failure_threshold
        self.success_threshold = success_threshold
        self.timeout = timeout
        self._state = CircuitBreakerState.CLOSED
        self._failure_count = 0
        self._success_count = 0
        self._opened_at = 0.0

    @property
    def state(self) -> CircuitBreakerState:
        # Check for timeout transition before returning state
        if self._state == CircuitBreakerState.OPEN:
            if time.monotonic() - self._opened_at >= self.timeout:
                self._state = CircuitBreakerState.HALF_OPEN
                self._success_count = 0
        return self._state

    def is_open(self) -> bool:
        """サーキットブレーカーが OPEN 状態か確認する。"""
        if self._state == CircuitBreakerState.OPEN:
            if time.monotonic() - self._opened_at >= self.timeout:
                self._state = CircuitBreakerState.HALF_OPEN
                self._success_count = 0
                return False
            return True
        return False

    def record_success(self) -> None:
        """成功を記録する。"""
        if self._state == CircuitBreakerState.HALF_OPEN:
            self._success_count += 1
            if self._success_count >= self.success_threshold:
                self._state = CircuitBreakerState.CLOSED
                self._failure_count = 0
        elif self._state == CircuitBreakerState.CLOSED:
            self._failure_count = 0

    def record_failure(self) -> None:
        """失敗を記録する。"""
        self._failure_count += 1
        if self._failure_count >= self.failure_threshold:
            self._state = CircuitBreakerState.OPEN
            self._opened_at = time.monotonic()
            self._failure_count = 0
