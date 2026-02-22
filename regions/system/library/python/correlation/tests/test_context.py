"""CorrelationContext のユニットテスト"""

import pytest
from k1s0_correlation.context import CorrelationContext


def test_context_creates_with_defaults() -> None:
    """デフォルト値で CorrelationContext が作成できること。"""
    ctx = CorrelationContext()
    assert ctx.correlation_id
    assert ctx.trace_id
    assert ctx.request_id is None


def test_context_creates_with_explicit_values() -> None:
    """明示的な値で CorrelationContext が作成できること。"""
    ctx = CorrelationContext(
        correlation_id="test-id",
        trace_id="abc123",
        request_id="req-456",
    )
    assert ctx.correlation_id == "test-id"
    assert ctx.trace_id == "abc123"
    assert ctx.request_id == "req-456"


def test_context_raises_on_empty_correlation_id() -> None:
    """空の correlation_id で ValueError が発生すること。"""
    with pytest.raises(ValueError, match="correlation_id cannot be empty"):
        CorrelationContext(correlation_id="", trace_id="abc")


def test_context_raises_on_empty_trace_id() -> None:
    """空の trace_id で ValueError が発生すること。"""
    with pytest.raises(ValueError, match="trace_id cannot be empty"):
        CorrelationContext(correlation_id="test", trace_id="")


def test_context_unique_ids() -> None:
    """異なるインスタンスで一意の ID が生成されること。"""
    ctx1 = CorrelationContext()
    ctx2 = CorrelationContext()
    assert ctx1.correlation_id != ctx2.correlation_id
    assert ctx1.trace_id != ctx2.trace_id
