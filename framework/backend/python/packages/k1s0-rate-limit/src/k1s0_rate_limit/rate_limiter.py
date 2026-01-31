"""Abstract base class for rate limiters."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from datetime import timedelta


@dataclass(frozen=True)
class RateLimitStats:
    """Statistics for a rate limiter instance.

    Attributes:
        allowed: Number of requests that were allowed.
        rejected: Number of requests that were rejected.
        total: Total number of requests processed.
        available: Current number of available tokens/slots.
    """

    allowed: int
    rejected: int
    total: int
    available: int


class RateLimiter(ABC):
    """Abstract rate limiter interface.

    Concrete implementations must provide token bucket or sliding window
    semantics.
    """

    @abstractmethod
    async def try_acquire(self) -> bool:
        """Attempt to acquire permission for a single request.

        Returns:
            ``True`` if the request is allowed, ``False`` otherwise.
        """
        ...

    @abstractmethod
    def time_until_available(self) -> timedelta:
        """Return the estimated wait time until the next token is available.

        Returns:
            A ``timedelta`` representing the wait duration. Returns
            ``timedelta()`` if a token is immediately available.
        """
        ...

    @abstractmethod
    def available_tokens(self) -> int:
        """Return the number of tokens currently available.

        Returns:
            Non-negative integer count of available tokens.
        """
        ...

    @abstractmethod
    def stats(self) -> RateLimitStats:
        """Return current rate limiter statistics.

        Returns:
            A snapshot of allowed, rejected, total, and available counts.
        """
        ...
