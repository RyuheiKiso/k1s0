"""Tests for SagaBuilder DSL."""

from __future__ import annotations

from typing import Any

import pytest

from k1s0_consensus.saga import BackoffStrategy, RetryPolicy, SagaBuilder, SagaStep


class _DummyStep(SagaStep[dict[str, Any]]):
    """Minimal saga step for builder tests."""

    def __init__(self, step_name: str) -> None:
        self._name = step_name

    @property
    def name(self) -> str:
        return self._name

    async def execute(self, context: dict[str, Any]) -> dict[str, Any]:
        return context

    async def compensate(self, context: dict[str, Any]) -> dict[str, Any]:
        return context


class TestSagaBuilder:
    """Tests for the SagaBuilder fluent API."""

    def test_build_with_single_step(self) -> None:
        """A saga with a single step should build successfully."""
        definition = SagaBuilder("test-saga").step(_DummyStep("step-1")).build()
        assert definition.name == "test-saga"
        assert len(definition.steps) == 1
        assert definition.steps[0].name == "step-1"

    def test_build_with_multiple_steps(self) -> None:
        """Steps should be preserved in insertion order."""
        definition = (
            SagaBuilder("multi-saga")
            .step(_DummyStep("a"))
            .step(_DummyStep("b"))
            .step(_DummyStep("c"))
            .build()
        )
        names = [s.name for s in definition.steps]
        assert names == ["a", "b", "c"]

    def test_build_with_no_steps_raises(self) -> None:
        """Building without any steps should raise ValueError."""
        with pytest.raises(ValueError, match="at least one step"):
            SagaBuilder("empty-saga").build()

    def test_custom_retry_policy(self) -> None:
        """A custom retry policy should be applied to the definition."""
        policy = RetryPolicy(
            max_retries=5,
            backoff_strategy=BackoffStrategy.LINEAR,
            initial_delay_ms=200,
            max_delay_ms=10_000,
            multiplier=1.5,
        )
        definition = (
            SagaBuilder("retry-saga")
            .step(_DummyStep("s"))
            .with_retry(policy)
            .build()
        )
        assert definition.retry_policy.max_retries == 5
        assert definition.retry_policy.backoff_strategy == BackoffStrategy.LINEAR

    def test_default_retry_policy(self) -> None:
        """Without with_retry, the default policy should be used."""
        definition = SagaBuilder("default-saga").step(_DummyStep("s")).build()
        assert definition.retry_policy.max_retries == 3
        assert definition.retry_policy.backoff_strategy == BackoffStrategy.EXPONENTIAL


class TestRetryPolicy:
    """Tests for RetryPolicy.delay_ms calculation."""

    def test_fixed_backoff(self) -> None:
        """Fixed backoff returns the same delay regardless of attempt."""
        policy = RetryPolicy(backoff_strategy=BackoffStrategy.FIXED, initial_delay_ms=500)
        assert policy.delay_ms(0) == 500
        assert policy.delay_ms(5) == 500

    def test_linear_backoff(self) -> None:
        """Linear backoff increases linearly."""
        policy = RetryPolicy(
            backoff_strategy=BackoffStrategy.LINEAR,
            initial_delay_ms=100,
            multiplier=2.0,
            max_delay_ms=1000,
        )
        assert policy.delay_ms(0) == 100
        assert policy.delay_ms(1) == 300
        assert policy.delay_ms(2) == 500

    def test_exponential_backoff(self) -> None:
        """Exponential backoff doubles each attempt."""
        policy = RetryPolicy(
            backoff_strategy=BackoffStrategy.EXPONENTIAL,
            initial_delay_ms=100,
            multiplier=2.0,
            max_delay_ms=50_000,
        )
        assert policy.delay_ms(0) == 100
        assert policy.delay_ms(1) == 200
        assert policy.delay_ms(2) == 400
        assert policy.delay_ms(3) == 800

    def test_max_delay_cap(self) -> None:
        """Delay should never exceed max_delay_ms."""
        policy = RetryPolicy(
            backoff_strategy=BackoffStrategy.EXPONENTIAL,
            initial_delay_ms=1000,
            multiplier=10.0,
            max_delay_ms=5000,
        )
        assert policy.delay_ms(5) == 5000

    @pytest.mark.parametrize(
        ("strategy", "attempt", "expected"),
        [
            (BackoffStrategy.FIXED, 0, 100),
            (BackoffStrategy.FIXED, 10, 100),
            (BackoffStrategy.EXPONENTIAL, 0, 100),
            (BackoffStrategy.EXPONENTIAL, 3, 800),
        ],
    )
    def test_parametrized_delays(
        self,
        strategy: BackoffStrategy,
        attempt: int,
        expected: int,
    ) -> None:
        """Parameterized backoff delay calculations."""
        policy = RetryPolicy(
            backoff_strategy=strategy,
            initial_delay_ms=100,
            multiplier=2.0,
            max_delay_ms=50_000,
        )
        assert policy.delay_ms(attempt) == expected
