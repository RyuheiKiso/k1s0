"""テスト用アサーションヘルパー。"""

from __future__ import annotations

import json
from typing import Any


class AssertionHelper:
    """テスト用アサーションヘルパー。"""

    @staticmethod
    def assert_json_contains(actual: Any, expected: Any) -> None:
        """JSON 部分一致アサーション。"""
        if isinstance(actual, str):
            actual = json.loads(actual)
        if isinstance(expected, str):
            expected = json.loads(expected)
        if not _json_contains(actual, expected):
            raise AssertionError(
                f"JSON partial match failed.\n"
                f"Actual: {json.dumps(actual)}\n"
                f"Expected: {json.dumps(expected)}"
            )

    @staticmethod
    def assert_event_emitted(events: list[dict[str, Any]], event_type: str) -> None:
        """イベント一覧に指定タイプのイベントが含まれるか検証する。"""
        found = any(e.get("type") == event_type for e in events)
        if not found:
            raise AssertionError(f"Event '{event_type}' not found in events")


class AssertionError(Exception):
    """アサーションエラー型。"""


def _json_contains(actual: Any, expected: Any) -> bool:
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            return False
        return all(
            k in actual and _json_contains(actual[k], v)
            for k, v in expected.items()
        )
    if isinstance(expected, list):
        if not isinstance(actual, list):
            return False
        return all(
            any(_json_contains(av, ev) for av in actual) for ev in expected
        )
    return actual == expected
