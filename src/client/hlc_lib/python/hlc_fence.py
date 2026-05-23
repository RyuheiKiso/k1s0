#!/usr/bin/env python3
"""
hlc_fence.py — k1s0-hlc: HLC フェンス（メモリバリア等価）ユーティリティ

製造業プラットフォーム k1s0 において、HLC タイムスタンプを用いた「フェンス」機構を提供する。

HLCFence は特定の HLCTimestamp を「フェンスポイント」として記録し、
以降の操作が必ずそのフェンスポイントより後の HLC を持つことを保証する。

分散システムでの利用例:
  - バッチ処理の完了確認: バッチが完了した HLC 以降のクエリのみを受け付ける
  - テナント境界フェンス: テナント A の書き込みが完了した後にのみ読み取りを許可する
  - スキーマ移行フェンス: スキーマ移行完了 HLC より後のトランザクションのみを新スキーマで処理する
"""

from __future__ import annotations

# dataclasses: フェンス状態の値オブジェクトに使用する
import dataclasses
# threading: スレッドセーフなフェンス操作に使用する
import threading
# time: フェンスのタイムアウト待機に使用する
import time
# typing: 型ヒントに使用する
from typing import Callable, Optional

# hlc モジュールから必要なクラスをインポートする
from .hlc import HLCClock, HLCTimestamp, EPOCH


@dataclasses.dataclass(frozen=True)
class FencePoint:
    """単一のフェンスポイントを表す不変データクラス。"""

    # ts: フェンスポイントの HLC タイムスタンプ
    ts: HLCTimestamp
    # label: フェンスポイントの説明ラベル（デバッグ用）
    label: str
    # created_wall_ms: フェンスが作成されたシステム時刻（デバッグ用のみ）
    created_wall_ms: int


class HLCFence:
    """
    HLC フェンス: 特定の HLCTimestamp を境界として、後続操作の HLC を保証するクラス。

    スレッドセーフ: threading.Condition で内部状態を保護し、wait_until_after() で
    フェンス通過を待機できる。
    """

    def __init__(self, clock: HLCClock) -> None:
        """HLCClock を受け取って HLCFence を初期化する。"""
        # clock をインスタンス変数に保存する
        self._clock: HLCClock = clock
        # _fence_ts: 現在のフェンスポイントの HLC タイムスタンプ
        self._fence_ts: HLCTimestamp = EPOCH
        # _condition: wait_until_after() の待機に使用する Condition オブジェクト
        self._condition: threading.Condition = threading.Condition(threading.Lock())
        # _fence_history: 過去のフェンスポイントの履歴（デバッグ用）
        self._fence_history: list = []

    def set_fence(self, label: str = "") -> FencePoint:
        """現在の HLC タイムスタンプをフェンスポイントとして設定する。"""
        # 現在の HLC タイムスタンプを取得する
        ts = self._clock.now()
        # システム時刻をミリ秒で取得する（デバッグ用のみ）
        created_wall_ms = int(time.time() * 1000)
        # FencePoint を生成する
        fp = FencePoint(ts=ts, label=label, created_wall_ms=created_wall_ms)
        # Condition を取得してフェンスポイントを更新する
        with self._condition:
            # フェンスポイントを更新する（新しい値が古い値より後の場合のみ更新する）
            if ts > self._fence_ts:
                self._fence_ts = ts
            # フェンス履歴に追加する（最大 100 件まで保持する）
            self._fence_history.append(fp)
            if len(self._fence_history) > 100:
                # 古いエントリを削除する
                self._fence_history.pop(0)
            # 待機中のスレッドに通知する
            self._condition.notify_all()
        # FencePoint を返す
        return fp

    def advance_to(self, ts: HLCTimestamp, label: str = "") -> FencePoint:
        """指定した HLC タイムスタンプをフェンスポイントとして設定する（set_fence の明示的バージョン）。"""
        # システム時刻をミリ秒で取得する（デバッグ用のみ）
        created_wall_ms = int(time.time() * 1000)
        # FencePoint を生成する
        fp = FencePoint(ts=ts, label=label, created_wall_ms=created_wall_ms)
        # Condition を取得してフェンスポイントを更新する
        with self._condition:
            # フェンスポイントを更新する（新しい値が古い値より後の場合のみ更新する）
            if ts > self._fence_ts:
                self._fence_ts = ts
            # フェンス履歴に追加する
            self._fence_history.append(fp)
            if len(self._fence_history) > 100:
                self._fence_history.pop(0)
            # 待機中のスレッドに通知する
            self._condition.notify_all()
        # FencePoint を返す
        return fp

    def current_fence(self) -> HLCTimestamp:
        """現在のフェンスポイントの HLC タイムスタンプを返す。"""
        # Condition を取得してスレッドセーフに読み取る
        with self._condition:
            return self._fence_ts

    def is_past_fence(self, ts: HLCTimestamp) -> bool:
        """指定した HLC タイムスタンプがフェンスポイントより後かどうかを返す。"""
        # Condition を取得してスレッドセーフに比較する
        with self._condition:
            return ts > self._fence_ts

    def wait_until_after(
        self,
        target_ts: HLCTimestamp,
        timeout_s: float = 30.0,
        check_interval_s: float = 0.05,
    ) -> bool:
        """
        現在のフェンスポイントが target_ts を超えるまで待機する。

        引数:
          target_ts        : 待機対象の HLC タイムスタンプ
          timeout_s        : 最大待機時間（秒）
          check_interval_s : ポーリング間隔（秒）

        戻り値:
          True: フェンスが target_ts を超えた場合
          False: タイムアウトした場合
        """
        # 待機の期限時刻を計算する（wall clock でのタイムアウト）
        deadline = time.monotonic() + timeout_s
        # Condition を取得してフェンスポイントを確認する
        with self._condition:
            # フェンスポイントが既に target_ts を超えている場合はすぐに True を返す
            while self._fence_ts <= target_ts:
                # 残り待機時間を計算する
                remaining = deadline - time.monotonic()
                # タイムアウトした場合は False を返す
                if remaining <= 0:
                    return False
                # 待機時間を計算する（残り時間と check_interval の小さい方）
                wait_time = min(remaining, check_interval_s)
                # 通知を待機する（timeout 付き）
                self._condition.wait(timeout=wait_time)
            # フェンスポイントが target_ts を超えた場合は True を返す
            return True

    def fence_history(self) -> list:
        """フェンスポイントの履歴（最大 100 件）を返す。"""
        # Condition を取得してスレッドセーフにコピーを返す
        with self._condition:
            return list(self._fence_history)


def make_fence_predicate(fence: HLCFence) -> Callable[[HLCTimestamp], bool]:
    """
    フェンスポイントより後かどうかを判定する Callable を返す。

    フィルタやガードとして使用できる純粋な関数に変換するためのヘルパー。
    """
    # is_past_fence メソッドをラップして返す
    return lambda ts: fence.is_past_fence(ts)
