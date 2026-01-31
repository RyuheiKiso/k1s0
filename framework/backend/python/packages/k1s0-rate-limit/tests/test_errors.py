"""Tests for the error hierarchy."""

from __future__ import annotations

from datetime import timedelta

from k1s0_rate_limit.errors import RateLimitError, RateLimitExceededError


class TestRateLimitErrors:
    """Tests for rate limit error classes."""

    def test_base_error_attributes(self) -> None:
        err = RateLimitError("rate_limit.test", "test detail", is_retryable=True)
        assert err.error_code == "rate_limit.test"
        assert err.detail == "test detail"
        assert err.is_retryable is True
        assert str(err) == "test detail"

    def test_exceeded_error_defaults(self) -> None:
        retry_after = timedelta(seconds=5)
        err = RateLimitExceededError(retry_after=retry_after)
        assert err.error_code == "rate_limit.exceeded"
        assert err.retry_after == retry_after
        assert err.is_retryable is True

    def test_exceeded_error_custom_detail(self) -> None:
        err = RateLimitExceededError(
            retry_after=timedelta(seconds=1),
            detail="Custom message",
        )
        assert err.detail == "Custom message"
