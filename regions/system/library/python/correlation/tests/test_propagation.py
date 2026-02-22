"""contextvars 伝播のユニットテスト"""

from k1s0_correlation.context import CorrelationContext
from k1s0_correlation.headers import X_CORRELATION_ID, X_REQUEST_ID, X_TRACE_ID
from k1s0_correlation.propagation import (
    extract_from_headers,
    get_correlation_context,
    inject_into_headers,
    reset_correlation_context,
    set_correlation_context,
)


def test_set_and_get_correlation_context() -> None:
    """セットしたコンテキストを取得できること。"""
    ctx = CorrelationContext(correlation_id="my-id", trace_id="abc123")
    token = set_correlation_context(ctx)
    try:
        result = get_correlation_context()
        assert result is ctx
    finally:
        reset_correlation_context(token)


def test_get_returns_none_when_not_set() -> None:
    """未設定時に None が返ること。"""
    # 注意: 他のテストが token をリセットしている前提
    # 新しい contextvars コンテキストで実行
    import contextvars

    ctx_copy = contextvars.copy_context()
    result = ctx_copy.run(get_correlation_context)
    assert result is None


def test_extract_from_headers_with_all_fields() -> None:
    """全ヘッダーが存在する場合の抽出。"""
    headers = {
        X_CORRELATION_ID: "corr-123",
        X_TRACE_ID: "trace-abc",
        X_REQUEST_ID: "req-789",
    }
    ctx = extract_from_headers(headers)
    assert ctx.correlation_id == "corr-123"
    assert ctx.trace_id == "trace-abc"
    assert ctx.request_id == "req-789"


def test_extract_from_headers_with_no_headers() -> None:
    """ヘッダーなしの場合は新規生成されること。"""
    ctx = extract_from_headers({})
    assert ctx.correlation_id
    assert ctx.trace_id


def test_extract_from_headers_with_only_correlation_id() -> None:
    """correlation_id のみの場合。"""
    headers = {X_CORRELATION_ID: "corr-only"}
    ctx = extract_from_headers(headers)
    assert ctx.correlation_id == "corr-only"
    assert ctx.trace_id  # 自動生成


def test_inject_into_headers() -> None:
    """ヘッダー辞書に値が注入されること。"""
    ctx = CorrelationContext(
        correlation_id="inject-corr",
        trace_id="inject-trace",
        request_id="inject-req",
    )
    headers: dict[str, str] = {}
    inject_into_headers(ctx, headers)
    assert headers[X_CORRELATION_ID] == "inject-corr"
    assert headers[X_TRACE_ID] == "inject-trace"
    assert headers[X_REQUEST_ID] == "inject-req"


def test_inject_into_headers_without_request_id() -> None:
    """request_id なしの場合は X-Request-ID が注入されないこと。"""
    ctx = CorrelationContext(correlation_id="c", trace_id="t")
    headers: dict[str, str] = {}
    inject_into_headers(ctx, headers)
    assert X_CORRELATION_ID in headers
    assert X_TRACE_ID in headers
    assert X_REQUEST_ID not in headers
