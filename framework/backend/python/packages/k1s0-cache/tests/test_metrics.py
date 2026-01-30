"""Tests for CacheMetrics."""

from __future__ import annotations

import pytest

from k1s0_cache.metrics import CacheMetrics


class TestCacheMetrics:
    def test_initial_state(self) -> None:
        m = CacheMetrics()
        assert m.hit_count == 0
        assert m.miss_count == 0
        assert m.operation_count == 0
        assert m.hit_rate == 0.0

    def test_all_hits(self) -> None:
        m = CacheMetrics()
        m.record_hit()
        m.record_hit()
        assert m.hit_rate == 1.0

    def test_all_misses(self) -> None:
        m = CacheMetrics()
        m.record_miss()
        m.record_miss()
        assert m.hit_rate == 0.0

    def test_mixed(self) -> None:
        m = CacheMetrics()
        m.record_hit()
        m.record_miss()
        m.record_hit()
        m.record_miss()
        assert m.hit_rate == pytest.approx(0.5)
