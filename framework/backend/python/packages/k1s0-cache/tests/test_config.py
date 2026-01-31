"""Tests for CacheConfig."""

from __future__ import annotations

from k1s0_cache.config import CacheConfig


class TestCacheConfig:
    """Tests for CacheConfig defaults and overrides."""

    def test_defaults(self) -> None:
        config = CacheConfig()
        assert config.host == "localhost"
        assert config.port == 6379
        assert config.db == 0
        assert config.prefix == ""
        assert config.pool_size == 10
        assert config.default_ttl == 3600
        assert config.connect_timeout == 5.0

    def test_custom_values(self) -> None:
        config = CacheConfig(
            host="redis.local",
            port=6380,
            db=2,
            prefix="myapp",
            pool_size=20,
            default_ttl=600,
            connect_timeout=2.0,
        )
        assert config.host == "redis.local"
        assert config.port == 6380
        assert config.db == 2
        assert config.prefix == "myapp"
        assert config.pool_size == 20
        assert config.default_ttl == 600
        assert config.connect_timeout == 2.0
