"""テスト用フィクスチャビルダー。"""

from __future__ import annotations

import random
import uuid


class FixtureBuilder:
    """テスト用フィクスチャビルダー。"""

    @staticmethod
    def uuid() -> str:
        """ランダム UUID を生成する。"""
        return str(uuid.uuid4())

    @staticmethod
    def email() -> str:
        """ランダムなテスト用メールアドレスを生成する。"""
        return f"test-{uuid.uuid4().hex[:8]}@example.com"

    @staticmethod
    def name() -> str:
        """ランダムなテスト用ユーザー名を生成する。"""
        return f"user-{uuid.uuid4().hex[:8]}"

    @staticmethod
    def int_value(min_val: int = 0, max_val: int = 100) -> int:
        """指定範囲のランダム整数を生成する。"""
        if min_val >= max_val:
            return min_val
        return random.randint(min_val, max_val - 1)

    @staticmethod
    def tenant_id() -> str:
        """テスト用テナント ID を生成する。"""
        return f"tenant-{uuid.uuid4().hex[:8]}"
