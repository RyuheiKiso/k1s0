"""FastAPI middleware integration for rate limiting."""

from __future__ import annotations

from k1s0_rate_limit.errors import RateLimitExceededError
from k1s0_rate_limit.rate_limiter import RateLimiter


class RateLimitMiddleware:
    """Middleware that enforces rate limiting on incoming requests.

    Raises :class:`RateLimitExceededError` when the rate limit is exceeded,
    allowing the presentation layer to convert it to an appropriate HTTP 429
    or gRPC ``RESOURCE_EXHAUSTED`` response.

    Args:
        limiter: The rate limiter instance to use for enforcement.

    Example::

        middleware = RateLimitMiddleware(TokenBucket(TokenBucketConfig()))
        await middleware.check()  # raises RateLimitExceededError if exceeded
    """

    def __init__(self, limiter: RateLimiter) -> None:
        self._limiter = limiter

    async def check(self) -> None:
        """Check the rate limit and raise if exceeded.

        Raises:
            RateLimitExceededError: If the rate limit has been exceeded.
        """
        if not await self._limiter.try_acquire():
            raise RateLimitExceededError(
                retry_after=self._limiter.time_until_available(),
            )
