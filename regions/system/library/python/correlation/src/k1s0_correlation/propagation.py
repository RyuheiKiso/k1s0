"""contextvars を使った相関コンテキスト伝播"""

from __future__ import annotations

import contextvars
from typing import Any

from .context import CorrelationContext
from .headers import X_CORRELATION_ID, X_REQUEST_ID, X_TRACE_ID

_correlation_context_var: contextvars.ContextVar[CorrelationContext | None] = (
    contextvars.ContextVar("correlation_context", default=None)
)

_CorrelationToken = contextvars.Token[CorrelationContext | None]


def set_correlation_context(ctx: CorrelationContext) -> _CorrelationToken:
    """現在のコンテキストに相関コンテキストをセットする。"""
    return _correlation_context_var.set(ctx)


def get_correlation_context() -> CorrelationContext | None:
    """現在のコンテキストから相関コンテキストを取得する。"""
    return _correlation_context_var.get()


def reset_correlation_context(token: contextvars.Token[Any]) -> None:
    """set_correlation_context で取得したトークンでリセットする。"""
    _correlation_context_var.reset(token)


def extract_from_headers(headers: dict[str, str]) -> CorrelationContext:
    """HTTPヘッダーから CorrelationContext を抽出する。

    ヘッダーが存在しない場合は新規生成する。
    """
    correlation_id = headers.get(X_CORRELATION_ID) or headers.get(X_CORRELATION_ID.lower())
    trace_id = headers.get(X_TRACE_ID) or headers.get(X_TRACE_ID.lower())
    request_id = headers.get(X_REQUEST_ID) or headers.get(X_REQUEST_ID.lower())

    ctx = CorrelationContext()
    if correlation_id:
        ctx = CorrelationContext(
            correlation_id=correlation_id,
            trace_id=trace_id or ctx.trace_id,
            request_id=request_id,
        )
    elif trace_id:
        ctx = CorrelationContext(
            trace_id=trace_id,
            request_id=request_id,
        )
    elif request_id:
        ctx = CorrelationContext(request_id=request_id)

    return ctx


def inject_into_headers(ctx: CorrelationContext, headers: dict[str, str]) -> None:
    """CorrelationContext の値を HTTPヘッダー辞書に注入する（in-place）。"""
    headers[X_CORRELATION_ID] = ctx.correlation_id
    headers[X_TRACE_ID] = ctx.trace_id
    if ctx.request_id is not None:
        headers[X_REQUEST_ID] = ctx.request_id
