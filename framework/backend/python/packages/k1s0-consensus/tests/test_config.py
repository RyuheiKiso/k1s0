"""Tests for consensus configuration loading."""

from __future__ import annotations

from pathlib import Path

import pytest

from k1s0_consensus.config import ConsensusConfig, load_consensus_config


class TestLoadConsensusConfig:
    """Tests for the YAML configuration loader."""

    def test_load_full_config(self, config_yaml_path: Path) -> None:
        """Loading a complete config file should populate all fields."""
        cfg = load_consensus_config(config_yaml_path)

        assert cfg.leader.lease_duration_ms == 10000
        assert cfg.leader.renew_interval_ms == 3000
        assert cfg.leader.table_name == "test_leader_lease"

        assert cfg.lock.default_ttl_ms == 20000
        assert cfg.lock.retry_delay_ms == 100
        assert cfg.lock.max_retries == 5

        assert cfg.saga.instance_table_name == "test_saga_instance"
        assert cfg.saga.step_table_name == "test_saga_step"
        assert cfg.saga.backoff.initial_delay_ms == 50
        assert cfg.saga.backoff.multiplier == 3.0
        assert cfg.saga.dead_letter.max_retries == 5
        assert cfg.saga.dead_letter.table_name == "test_dead_letter"

        assert cfg.redis.host == "redis.test"
        assert cfg.redis.port == 6380
        assert cfg.redis.password_file == "/var/run/secrets/k1s0/redis_password"

        assert cfg.choreography.default_timeout_ms == 30000

    def test_load_missing_file_returns_defaults(self, tmp_path: Path) -> None:
        """A missing config file should return all defaults."""
        cfg = load_consensus_config(tmp_path / "nonexistent.yaml")
        default = ConsensusConfig()
        assert cfg == default

    def test_load_empty_file_returns_defaults(self, tmp_path: Path) -> None:
        """An empty YAML file should return all defaults."""
        empty = tmp_path / "empty.yaml"
        empty.write_text("", encoding="utf-8")
        cfg = load_consensus_config(empty)
        assert cfg == ConsensusConfig()

    def test_load_partial_config(self, tmp_path: Path) -> None:
        """A partial config should merge with defaults."""
        partial = tmp_path / "partial.yaml"
        partial.write_text(
            "consensus:\n  leader:\n    lease_duration_ms: 5000\n",
            encoding="utf-8",
        )
        cfg = load_consensus_config(partial)
        assert cfg.leader.lease_duration_ms == 5000
        # Other leader fields should be defaults
        assert cfg.leader.renew_interval_ms == 5000
        # Lock should be fully default
        assert cfg.lock.default_ttl_ms == 30000

    def test_frozen_config(self, config_yaml_path: Path) -> None:
        """Config dataclasses should be immutable."""
        cfg = load_consensus_config(config_yaml_path)
        with pytest.raises(AttributeError):
            cfg.leader.lease_duration_ms = 999  # type: ignore[misc]


class TestConsensusConfigDefaults:
    """Tests for default configuration values."""

    def test_default_leader_config(self) -> None:
        """Default leader config should have sensible values."""
        cfg = ConsensusConfig()
        assert cfg.leader.lease_duration_ms == 15_000
        assert cfg.leader.renew_interval_ms == 5_000
        assert cfg.leader.table_name == "k1s0_leader_lease"

    def test_default_lock_config(self) -> None:
        """Default lock config should have sensible values."""
        cfg = ConsensusConfig()
        assert cfg.lock.default_ttl_ms == 30_000
        assert cfg.lock.max_retries == 10

    def test_default_redis_config(self) -> None:
        """Default Redis config should point to localhost."""
        cfg = ConsensusConfig()
        assert cfg.redis.host == "localhost"
        assert cfg.redis.port == 6379
        assert cfg.redis.password_file == ""
        assert cfg.redis.ssl is False
