"""相関ID・トレースID生成ユーティリティ"""

from __future__ import annotations

import uuid


def generate_correlation_id() -> str:
    """UUID v4 形式の相関IDを生成する。"""
    return str(uuid.uuid4())


def generate_trace_id() -> str:
    """32文字の16進数トレースIDを生成する。"""
    return uuid.uuid4().hex
