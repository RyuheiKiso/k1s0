"""Shared pytest fixtures for k1s0-consensus tests."""

from __future__ import annotations

import tempfile
from pathlib import Path
from typing import Any

import pytest
import yaml

from k1s0_consensus.config import ConsensusConfig, load_consensus_config
from k1s0_consensus.fencing import FencingValidator
from k1s0_consensus.saga import BackoffStrategy, RetryPolicy, SagaBuilder, SagaStep


@pytest.fixture()
def fencing_validator() -> FencingValidator:
    """Provide a fresh FencingValidator."""
    return FencingValidator()


@pytest.fixture()
def default_config() -> ConsensusConfig:
    """Provide a default ConsensusConfig."""
    return ConsensusConfig()


@pytest.fixture()
def config_yaml_path(tmp_path: Path) -> Path:
    """Create a temporary YAML config file and return its path."""
    data = {
        "consensus": {
            "leader": {
                "lease_duration_ms": 10000,
                "renew_interval_ms": 3000,
                "table_name": "test_leader_lease",
            },
            "lock": {
                "default_ttl_ms": 20000,
                "retry_delay_ms": 100,
                "max_retries": 5,
            },
            "saga": {
                "instance_table_name": "test_saga_instance",
                "step_table_name": "test_saga_step",
                "backoff": {
                    "initial_delay_ms": 50,
                    "max_delay_ms": 10000,
                    "multiplier": 3.0,
                },
                "dead_letter": {
                    "max_retries": 5,
                    "table_name": "test_dead_letter",
                },
            },
            "redis": {
                "host": "redis.test",
                "port": 6380,
                "password_file": "/var/run/secrets/k1s0/redis_password",
            },
            "choreography": {
                "default_timeout_ms": 30000,
            },
        }
    }
    config_file = tmp_path / "config.yaml"
    config_file.write_text(yaml.dump(data), encoding="utf-8")
    return config_file


class _SuccessStep(SagaStep[dict[str, Any]]):
    """A saga step that always succeeds, appending its name to a list."""

    def __init__(self, step_name: str) -> None:
        self._name = step_name

    @property
    def name(self) -> str:
        return self._name

    async def execute(self, context: dict[str, Any]) -> dict[str, Any]:
        steps = context.get("executed", [])
        steps.append(self._name)
        return {**context, "executed": steps}

    async def compensate(self, context: dict[str, Any]) -> dict[str, Any]:
        compensated = context.get("compensated", [])
        compensated.append(self._name)
        return {**context, "compensated": compensated}


class _FailingStep(SagaStep[dict[str, Any]]):
    """A saga step that always raises an exception."""

    def __init__(self, step_name: str) -> None:
        self._name = step_name

    @property
    def name(self) -> str:
        return self._name

    async def execute(self, context: dict[str, Any]) -> dict[str, Any]:
        msg = f"Step {self._name} intentionally failed"
        raise RuntimeError(msg)

    async def compensate(self, context: dict[str, Any]) -> dict[str, Any]:
        compensated = context.get("compensated", [])
        compensated.append(self._name)
        return {**context, "compensated": compensated}


@pytest.fixture()
def success_step() -> type[_SuccessStep]:
    """Provide the SuccessStep class for building test sagas."""
    return _SuccessStep


@pytest.fixture()
def failing_step() -> type[_FailingStep]:
    """Provide the FailingStep class for building test sagas."""
    return _FailingStep
