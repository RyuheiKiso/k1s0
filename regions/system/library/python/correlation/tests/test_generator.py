"""ID生成ユーティリティのユニットテスト"""

import re

from k1s0_correlation.generator import generate_correlation_id, generate_trace_id


def test_generate_correlation_id_format() -> None:
    """UUID v4 形式の相関IDが生成されること。"""
    cid = generate_correlation_id()
    uuid_pattern = r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
    assert re.match(uuid_pattern, cid), f"Expected UUID v4 format, got: {cid}"


def test_generate_trace_id_format() -> None:
    """32文字の16進数トレースIDが生成されること。"""
    tid = generate_trace_id()
    assert len(tid) == 32
    assert all(c in "0123456789abcdef" for c in tid)


def test_generate_correlation_id_unique() -> None:
    """連続呼び出しで一意の ID が生成されること。"""
    ids = {generate_correlation_id() for _ in range(100)}
    assert len(ids) == 100


def test_generate_trace_id_unique() -> None:
    """連続呼び出しで一意のトレース ID が生成されること。"""
    ids = {generate_trace_id() for _ in range(100)}
    assert len(ids) == 100
