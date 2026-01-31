"""Consensus configuration loaded from YAML files."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml


@dataclass(frozen=True)
class BackoffConfig:
    """Backoff configuration for retries.

    Attributes:
        initial_delay_ms: Initial delay in milliseconds before the first retry.
        max_delay_ms: Maximum delay in milliseconds between retries.
        multiplier: Multiplier applied to the delay after each retry.
    """

    initial_delay_ms: int = 100
    max_delay_ms: int = 30_000
    multiplier: float = 2.0


@dataclass(frozen=True)
class RedisConfig:
    """Redis connection configuration.

    Attributes:
        host: Redis server hostname.
        port: Redis server port.
        db: Redis database index.
        password_file: Path to a file containing the Redis password.
        ssl: Whether to use TLS for the connection.
        socket_timeout_ms: Socket timeout in milliseconds.
    """

    host: str = "localhost"
    port: int = 6379
    db: int = 0
    password_file: str = ""
    ssl: bool = False
    socket_timeout_ms: int = 5000


@dataclass(frozen=True)
class LeaderConfig:
    """Leader election configuration.

    Attributes:
        lease_duration_ms: Duration of the leadership lease in milliseconds.
        renew_interval_ms: Interval between lease renewal attempts in milliseconds.
        table_name: Database table for lease persistence.
    """

    lease_duration_ms: int = 15_000
    renew_interval_ms: int = 5_000
    table_name: str = "k1s0_leader_lease"


@dataclass(frozen=True)
class LockConfig:
    """Distributed lock configuration.

    Attributes:
        default_ttl_ms: Default lock time-to-live in milliseconds.
        retry_delay_ms: Delay between lock acquisition retries in milliseconds.
        max_retries: Maximum number of acquisition attempts.
        table_name: Database table for lock persistence.
    """

    default_ttl_ms: int = 30_000
    retry_delay_ms: int = 200
    max_retries: int = 10
    table_name: str = "k1s0_distributed_lock"


@dataclass(frozen=True)
class DeadLetterConfig:
    """Dead-letter queue configuration for failed sagas.

    Attributes:
        max_retries: Maximum retries before dead-lettering a saga.
        table_name: Database table for dead-letter entries.
    """

    max_retries: int = 3
    table_name: str = "k1s0_saga_dead_letter"


@dataclass(frozen=True)
class SagaConfig:
    """Saga orchestrator configuration.

    Attributes:
        instance_table_name: Database table for saga instances.
        step_table_name: Database table for saga step records.
        backoff: Backoff configuration for step retries.
        dead_letter: Dead-letter queue configuration.
    """

    instance_table_name: str = "k1s0_saga_instance"
    step_table_name: str = "k1s0_saga_step"
    backoff: BackoffConfig = field(default_factory=BackoffConfig)
    dead_letter: DeadLetterConfig = field(default_factory=DeadLetterConfig)


@dataclass(frozen=True)
class ChoreographyConfig:
    """Choreography-based saga configuration.

    Attributes:
        default_timeout_ms: Default timeout for event handlers in milliseconds.
    """

    default_timeout_ms: int = 60_000


@dataclass(frozen=True)
class ConsensusConfig:
    """Top-level consensus configuration.

    Attributes:
        leader: Leader election settings.
        lock: Distributed lock settings.
        saga: Saga orchestrator settings.
        redis: Redis connection settings.
        choreography: Choreography saga settings.
    """

    leader: LeaderConfig = field(default_factory=LeaderConfig)
    lock: LockConfig = field(default_factory=LockConfig)
    saga: SagaConfig = field(default_factory=SagaConfig)
    redis: RedisConfig = field(default_factory=RedisConfig)
    choreography: ChoreographyConfig = field(default_factory=ChoreographyConfig)


def _read_section(data: dict[str, Any], key: str) -> dict[str, Any]:
    """Read a nested section from a dictionary, returning empty dict if absent."""
    value = data.get(key, {})
    if not isinstance(value, dict):
        return {}
    return value


def load_consensus_config(path: str | Path) -> ConsensusConfig:
    """Load consensus configuration from a YAML file.

    The YAML file should contain a top-level ``consensus`` key with nested
    sections for ``leader``, ``lock``, ``saga``, ``redis``, and
    ``choreography``.

    Example YAML::

        consensus:
          leader:
            lease_duration_ms: 15000
          lock:
            default_ttl_ms: 30000
          redis:
            host: redis.local
            password_file: /var/run/secrets/k1s0/redis_password

    Args:
        path: Path to the YAML configuration file.

    Returns:
        A fully populated ConsensusConfig instance.
    """
    config_path = Path(path)
    if not config_path.exists():
        return ConsensusConfig()

    with config_path.open("r", encoding="utf-8") as fh:
        raw: dict[str, Any] = yaml.safe_load(fh) or {}

    consensus_raw = _read_section(raw, "consensus")
    if not consensus_raw:
        return ConsensusConfig()

    leader_raw = _read_section(consensus_raw, "leader")
    lock_raw = _read_section(consensus_raw, "lock")
    saga_raw = _read_section(consensus_raw, "saga")
    redis_raw = _read_section(consensus_raw, "redis")
    choreo_raw = _read_section(consensus_raw, "choreography")

    backoff_raw = _read_section(saga_raw, "backoff")
    dl_raw = _read_section(saga_raw, "dead_letter")

    return ConsensusConfig(
        leader=LeaderConfig(**{k: v for k, v in leader_raw.items() if k in LeaderConfig.__dataclass_fields__}),
        lock=LockConfig(**{k: v for k, v in lock_raw.items() if k in LockConfig.__dataclass_fields__}),
        saga=SagaConfig(
            instance_table_name=saga_raw.get("instance_table_name", "k1s0_saga_instance"),
            step_table_name=saga_raw.get("step_table_name", "k1s0_saga_step"),
            backoff=BackoffConfig(
                **{k: v for k, v in backoff_raw.items() if k in BackoffConfig.__dataclass_fields__}
            ),
            dead_letter=DeadLetterConfig(
                **{k: v for k, v in dl_raw.items() if k in DeadLetterConfig.__dataclass_fields__}
            ),
        ),
        redis=RedisConfig(**{k: v for k, v in redis_raw.items() if k in RedisConfig.__dataclass_fields__}),
        choreography=ChoreographyConfig(
            **{k: v for k, v in choreo_raw.items() if k in ChoreographyConfig.__dataclass_fields__}
        ),
    )
