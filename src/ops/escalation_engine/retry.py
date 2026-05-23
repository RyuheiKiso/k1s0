"""src/ops/escalation_engine/retry.py

エスカレーション通知のリトライ・バックオフロジック。
仕様: 17_運用ループ適合仕様.md §self_escalation_engine

- RetryPolicy: 最大試行回数・基底遅延・バックオフ係数・ジッタを定義する
- ExponentialBackoff: 切り詰め指数バックオフ + ジッタを計算する
- RetryScheduler: 複数通知の並行リトライキューを管理する
- DLQ: max_attempts 超過後はデッドレターキュー（ファイルベース）へ転送する
- CircuitBreaker: N 回連続失敗でトリップ、半開状態で自動回復を試みる
"""

from __future__ import annotations

import json
import logging
import math
import os
import random
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable

# モジュールロガーを初期化する
logger = logging.getLogger(__name__)

# DLQ ファイルのデフォルト保存先ディレクトリ
_DLQ_DIR_DEFAULT = Path(os.environ.get("RETRY_DLQ_DIR", "/tmp/k1s0_dlq"))


# ---------------------------------------------------------------------------
# RetryPolicy データクラス
# ---------------------------------------------------------------------------

@dataclass
class RetryPolicy:
    """通知リトライのポリシー設定を保持するデータクラス。"""

    # 最大試行回数（初回送信を含まない）
    max_attempts: int = 5
    # 基底遅延秒数（最初のリトライ待機時間の基準値）
    base_delay_s: float = 1.0
    # 最大遅延秒数（バックオフの上限キャップ値）
    max_delay_s: float = 300.0
    # バックオフ係数（各リトライで遅延を乗算する倍率）
    backoff_factor: float = 2.0
    # ジッタ係数（0.0〜1.0、0.0 でジッタなし）
    jitter: float = 0.3
    # サーキットブレーカーのトリップ閾値（連続失敗回数）
    circuit_breaker_threshold: int = 5
    # サーキットブレーカーの半開状態への移行待機時間（秒）
    circuit_breaker_reset_s: float = 60.0

    def validate(self) -> None:
        """ポリシー設定値の妥当性を検証する。不正な値は ValueError を送出する。"""
        # max_attempts は 1 以上でなければならない
        if self.max_attempts < 1:
            raise ValueError(f"max_attempts は 1 以上必須: {self.max_attempts}")
        # base_delay_s は正の値でなければならない
        if self.base_delay_s <= 0:
            raise ValueError(f"base_delay_s は正の値必須: {self.base_delay_s}")
        # max_delay_s は base_delay_s 以上でなければならない
        if self.max_delay_s < self.base_delay_s:
            raise ValueError(f"max_delay_s >= base_delay_s 必須: {self.max_delay_s} < {self.base_delay_s}")
        # backoff_factor は 1.0 以上でなければならない
        if self.backoff_factor < 1.0:
            raise ValueError(f"backoff_factor は 1.0 以上必須: {self.backoff_factor}")
        # jitter は 0.0〜1.0 の範囲でなければならない
        if not (0.0 <= self.jitter <= 1.0):
            raise ValueError(f"jitter は 0.0〜1.0 の範囲必須: {self.jitter}")


# ---------------------------------------------------------------------------
# RetryState データクラス
# ---------------------------------------------------------------------------

@dataclass
class RetryState:
    """単一の通知リトライ状態を追跡するデータクラス。"""

    # 通知 ID（一意識別子）
    notification_id: str
    # 現在の試行回数（0 = まだ試行していない）
    attempt_count: int = 0
    # 最後の試行時刻（Unix タイムスタンプ秒）
    last_attempt_at: float = 0.0
    # 次の試行予定時刻（Unix タイムスタンプ秒）
    next_attempt_at: float = 0.0
    # 最後のエラーメッセージ（エラーがない場合は空文字列）
    last_error: str = ""
    # 成功フラグ（True の場合はリトライ完了）
    succeeded: bool = False
    # DLQ 転送フラグ（True の場合はデッドレターキューに転送済み）
    in_dlq: bool = False
    # 通知ペイロード（元の通知内容を保持する）
    payload: dict[str, Any] = field(default_factory=dict)

    def is_exhausted(self, policy: RetryPolicy) -> bool:
        """試行回数が最大に達したか確認する。"""
        # attempt_count が max_attempts 以上の場合は枯渇
        return self.attempt_count >= policy.max_attempts

    def is_ready(self, now: float | None = None) -> bool:
        """次の試行実行時刻に達しているか確認する。"""
        # 現在時刻を取得する
        current = now if now is not None else time.time()
        # next_attempt_at が 0 の場合（初回）は即時実行可能
        if self.next_attempt_at == 0.0:
            return True
        # 現在時刻が next_attempt_at 以上の場合は実行可能
        return current >= self.next_attempt_at

    def to_dict(self) -> dict[str, Any]:
        """RetryState を辞書形式にシリアライズする。"""
        return {
            "notification_id": self.notification_id,
            "attempt_count": self.attempt_count,
            "last_attempt_at": self.last_attempt_at,
            "next_attempt_at": self.next_attempt_at,
            "last_error": self.last_error,
            "succeeded": self.succeeded,
            "in_dlq": self.in_dlq,
            "payload": self.payload,
        }


# ---------------------------------------------------------------------------
# ExponentialBackoff クラス
# ---------------------------------------------------------------------------

class ExponentialBackoff:
    """切り詰め指数バックオフ + ジッタを計算するクラス。

    Formula: delay = min(base * factor^(attempt-1), max) * (1 + jitter * random())
    """

    def __init__(self, policy: RetryPolicy) -> None:
        """ExponentialBackoff を RetryPolicy で初期化する。"""
        # ポリシー設定を検証してから保持する
        policy.validate()
        self._policy = policy

    def compute_delay(self, attempt: int) -> float:
        """指定試行番号のバックオフ遅延秒数を計算する。

        Args:
            attempt: 試行番号（1 始まり: 1 = 1 回目のリトライ）

        Returns:
            適用すべき遅延秒数（ジッタ込み）
        """
        # 試行番号が 0 以下の場合は 0 を返す
        if attempt <= 0:
            return 0.0
        # 基底遅延 × バックオフ係数^(attempt-1) を計算する
        raw_delay = self._policy.base_delay_s * (self._policy.backoff_factor ** (attempt - 1))
        # max_delay_s でキャップする
        capped_delay = min(raw_delay, self._policy.max_delay_s)
        # ジッタを加算する（0〜jitter * capped_delay の範囲）
        jitter_amount = capped_delay * self._policy.jitter * random.random()
        # ジッタ込みの遅延を返す
        return capped_delay + jitter_amount

    def compute_next_attempt_at(self, attempt: int, now: float | None = None) -> float:
        """次の試行予定時刻（Unix タイムスタンプ）を計算する。"""
        # 現在時刻を取得する
        current = now if now is not None else time.time()
        # 遅延を計算して現在時刻に加算する
        delay = self.compute_delay(attempt)
        return current + delay

    def get_delays_preview(self, n: int) -> list[float]:
        """最初の n 回分のバックオフ遅延秒数のプレビューを返す（ジッタなし）。"""
        # ジッタなしで各試行の遅延を計算する
        delays = []
        for i in range(1, n + 1):
            raw = self._policy.base_delay_s * (self._policy.backoff_factor ** (i - 1))
            delays.append(min(raw, self._policy.max_delay_s))
        return delays


# ---------------------------------------------------------------------------
# DeadLetterQueue クラス
# ---------------------------------------------------------------------------

class DeadLetterQueue:
    """最大試行回数を超過した通知を記録するファイルベースの DLQ。"""

    def __init__(self, dlq_dir: Path | None = None) -> None:
        """DeadLetterQueue を初期化する。"""
        # DLQ ディレクトリを設定する
        self._dlq_dir = dlq_dir or _DLQ_DIR_DEFAULT
        # DLQ ディレクトリが存在しない場合は作成する
        self._dlq_dir.mkdir(parents=True, exist_ok=True)
        # DLQ への書き込み件数を追跡するカウンタを初期化する
        self._dlq_count: int = 0

    @property
    def dlq_dir(self) -> Path:
        """DLQ ディレクトリのパスを返す。"""
        return self._dlq_dir

    @property
    def dlq_count(self) -> int:
        """DLQ に書き込まれた通知の総件数を返す。"""
        return self._dlq_count

    def enqueue(self, state: RetryState, reason: str = "max_attempts_exceeded") -> Path:
        """通知を DLQ に転送してファイルに記録する。

        Args:
            state: 転送する通知の RetryState
            reason: DLQ 転送の理由

        Returns:
            書き込んだ DLQ エントリファイルのパス
        """
        # DLQ エントリを構築する
        entry = {
            "notification_id": state.notification_id,
            "reason": reason,
            "attempt_count": state.attempt_count,
            "last_error": state.last_error,
            "last_attempt_at": state.last_attempt_at,
            "enqueued_at": time.time(),
            "payload": state.payload,
        }
        # ファイル名を生成する（notification_id + タイムスタンプ）
        ts_int = int(time.time() * 1000)
        filename = f"dlq_{state.notification_id}_{ts_int}.json"
        filepath = self._dlq_dir / filename
        # JSON 形式でファイルに書き込む
        with open(filepath, "w", encoding="utf-8") as fh:
            json.dump(entry, fh, ensure_ascii=False, indent=2)
        # DLQ カウンタをインクリメントする
        self._dlq_count += 1
        # DLQ 転送完了をログに記録する
        logger.warning(
            "DLQ: 通知転送 notification_id=%s attempts=%d reason=%s path=%s",
            state.notification_id, state.attempt_count, reason, filepath
        )
        return filepath

    def list_entries(self) -> list[dict[str, Any]]:
        """DLQ に保存されている全エントリを読み込んで返す。"""
        # DLQ ディレクトリの JSON ファイルを一覧取得する
        entries = []
        for json_file in sorted(self._dlq_dir.glob("dlq_*.json")):
            try:
                with open(json_file, "r", encoding="utf-8") as fh:
                    entries.append(json.load(fh))
            except Exception as exc:
                logger.error("DLQ: ファイル読み込みエラー path=%s error=%s", json_file, exc)
        return entries

    def clear(self) -> int:
        """DLQ の全ファイルを削除してエントリ数を返す。"""
        # DLQ ファイルを全て削除する
        count = 0
        for json_file in self._dlq_dir.glob("dlq_*.json"):
            try:
                json_file.unlink()
                count += 1
            except Exception as exc:
                logger.error("DLQ: ファイル削除エラー path=%s error=%s", json_file, exc)
        return count


# ---------------------------------------------------------------------------
# CircuitBreaker クラス
# ---------------------------------------------------------------------------

class CircuitBreakerState:
    """サーキットブレーカーの状態定数を定義するクラス。"""

    # 閉状態: 通常動作中（通知を送信する）
    CLOSED = "closed"
    # 開状態: 連続失敗でトリップ（通知をブロックする）
    OPEN = "open"
    # 半開状態: リセット試行中（1 回だけ通知を試みる）
    HALF_OPEN = "half_open"


class CircuitBreaker:
    """通知プロバイダーの連続失敗を検知してトリップするサーキットブレーカー。"""

    def __init__(self, policy: RetryPolicy, name: str = "default") -> None:
        """CircuitBreaker を初期化する。"""
        # ポリシー設定を保持する
        self._policy = policy
        # ブレーカーの名前（ログ識別用）
        self._name = name
        # 現在の状態を閉状態で初期化する
        self._state = CircuitBreakerState.CLOSED
        # 連続失敗カウンタを初期化する
        self._consecutive_failures: int = 0
        # 最後にトリップした時刻を初期化する
        self._tripped_at: float = 0.0
        # 成功カウンタを初期化する
        self._success_count: int = 0
        # 失敗カウンタを初期化する
        self._failure_count: int = 0

    @property
    def state(self) -> str:
        """現在のサーキットブレーカー状態を返す。"""
        return self._state

    @property
    def consecutive_failures(self) -> int:
        """現在の連続失敗カウンタの値を返す。"""
        return self._consecutive_failures

    def is_open(self) -> bool:
        """サーキットブレーカーが開状態か確認する。"""
        # 開状態の場合は True を返す
        if self._state == CircuitBreakerState.OPEN:
            # リセット時間を経過した場合は半開状態に移行する
            if time.time() - self._tripped_at >= self._policy.circuit_breaker_reset_s:
                self._state = CircuitBreakerState.HALF_OPEN
                logger.info("CircuitBreaker[%s]: OPEN → HALF_OPEN 移行", self._name)
                return False
            return True
        # 半開状態は通過を許可する
        return False

    def record_success(self) -> None:
        """成功を記録してサーキットブレーカーをリセットする。"""
        # 成功カウンタをインクリメントする
        self._success_count += 1
        # 連続失敗カウンタをリセットする
        self._consecutive_failures = 0
        # 半開状態から閉状態に移行する
        if self._state == CircuitBreakerState.HALF_OPEN:
            self._state = CircuitBreakerState.CLOSED
            logger.info("CircuitBreaker[%s]: HALF_OPEN → CLOSED 移行（回復）", self._name)
        # 閉状態でも success を記録する
        elif self._state == CircuitBreakerState.CLOSED:
            pass

    def record_failure(self, error: str = "") -> None:
        """失敗を記録してサーキットブレーカーのトリップを判定する。"""
        # 失敗カウンタをインクリメントする
        self._failure_count += 1
        # 連続失敗カウンタをインクリメントする
        self._consecutive_failures += 1
        # 半開状態で失敗した場合は開状態に戻す
        if self._state == CircuitBreakerState.HALF_OPEN:
            self._state = CircuitBreakerState.OPEN
            self._tripped_at = time.time()
            logger.warning(
                "CircuitBreaker[%s]: HALF_OPEN → OPEN 再トリップ error=%s", self._name, error
            )
            return
        # 連続失敗回数が閾値に達した場合はトリップする
        if self._consecutive_failures >= self._policy.circuit_breaker_threshold:
            self._state = CircuitBreakerState.OPEN
            self._tripped_at = time.time()
            logger.error(
                "CircuitBreaker[%s]: CLOSED → OPEN トリップ consecutive_failures=%d error=%s",
                self._name, self._consecutive_failures, error
            )

    def get_metrics(self) -> dict[str, Any]:
        """サーキットブレーカーのメトリクスを辞書形式で返す。"""
        return {
            "name": self._name,
            "state": self._state,
            "consecutive_failures": self._consecutive_failures,
            "success_count": self._success_count,
            "failure_count": self._failure_count,
            "tripped_at": self._tripped_at,
        }


# ---------------------------------------------------------------------------
# RetryScheduler クラス
# ---------------------------------------------------------------------------

class RetryScheduler:
    """複数の通知の並行リトライキューを管理するスケジューラー。"""

    def __init__(
        self,
        policy: RetryPolicy | None = None,
        dlq_dir: Path | None = None,
        send_fn: Callable[[str, dict[str, Any]], bool] | None = None,
    ) -> None:
        """RetryScheduler を初期化する。

        Args:
            policy: リトライポリシー設定（None の場合はデフォルト値を使用する）
            dlq_dir: DLQ ファイル保存先ディレクトリ
            send_fn: 実際の通知送信関数 (notification_id, payload) → bool
        """
        # リトライポリシーを設定する（未指定の場合はデフォルト値を使用する）
        self._policy = policy or RetryPolicy()
        # バックオフ計算機を初期化する
        self._backoff = ExponentialBackoff(self._policy)
        # DLQ を初期化する
        self._dlq = DeadLetterQueue(dlq_dir)
        # サーキットブレーカーを初期化する
        self._circuit_breaker = CircuitBreaker(self._policy, name="notification")
        # 通知 ID → RetryState のキュー辞書を初期化する
        self._queue: dict[str, RetryState] = {}
        # 通知送信関数を保持する（None の場合は dry-run モード）
        self._send_fn = send_fn
        # 成功件数カウンタを初期化する
        self._success_count: int = 0
        # 失敗件数カウンタを初期化する
        self._failure_count: int = 0

    @property
    def success_count(self) -> int:
        """成功した通知送信の総件数を返す。"""
        return self._success_count

    @property
    def failure_count(self) -> int:
        """失敗した通知送信の総件数を返す。"""
        return self._failure_count

    @property
    def dlq_count(self) -> int:
        """DLQ に転送された通知の総件数を返す。"""
        return self._dlq.dlq_count

    def enqueue(self, notification_id: str, payload: dict[str, Any]) -> RetryState:
        """通知をリトライキューに追加する。

        Args:
            notification_id: 通知の一意識別子
            payload: 通知ペイロード辞書

        Returns:
            新しく生成された RetryState
        """
        # 既存エントリが存在する場合は上書きする
        state = RetryState(
            notification_id=notification_id,
            payload=payload,
            next_attempt_at=time.time(),
        )
        # キューに追加する
        self._queue[notification_id] = state
        logger.debug("RetryScheduler: キュー追加 notification_id=%s", notification_id)
        return state

    def process_due(self, now: float | None = None) -> list[str]:
        """実行時刻に達した全通知のリトライを実行する。

        Args:
            now: 現在時刻（テスト用にオーバーライド可能）

        Returns:
            この呼び出しで処理した notification_id のリスト
        """
        # 現在時刻を取得する
        current = now if now is not None else time.time()
        # 処理した通知 ID のリストを初期化する
        processed_ids = []
        # 実行時刻に達した通知を収集する
        due_states = [
            state for state in self._queue.values()
            if state.is_ready(current) and not state.succeeded and not state.in_dlq
        ]
        # 各通知を処理する
        for state in due_states:
            self._execute_attempt(state, current)
            processed_ids.append(state.notification_id)
        return processed_ids

    def _execute_attempt(self, state: RetryState, now: float) -> None:
        """1 回の通知送信試行を実行する（内部メソッド）。"""
        # サーキットブレーカーが開状態の場合はスキップする
        if self._circuit_breaker.is_open():
            logger.warning(
                "RetryScheduler: サーキットブレーカー OPEN - スキップ notification_id=%s",
                state.notification_id
            )
            return
        # 試行回数をインクリメントする
        state.attempt_count += 1
        # 最後の試行時刻を更新する
        state.last_attempt_at = now
        # 通知送信を試みる
        try:
            success = self._try_send(state.notification_id, state.payload)
        except Exception as exc:
            # 例外を失敗として扱う
            success = False
            state.last_error = str(exc)
            logger.error(
                "RetryScheduler: 送信例外 notification_id=%s attempt=%d error=%s",
                state.notification_id, state.attempt_count, exc
            )
        # 成功の場合は処理完了とする
        if success:
            state.succeeded = True
            self._success_count += 1
            self._circuit_breaker.record_success()
            logger.info(
                "RetryScheduler: 送信成功 notification_id=%s attempt=%d",
                state.notification_id, state.attempt_count
            )
            return
        # 失敗の場合はカウンタを更新する
        self._failure_count += 1
        self._circuit_breaker.record_failure(state.last_error)
        # 最大試行回数に達した場合は DLQ へ転送する
        if state.is_exhausted(self._policy):
            state.in_dlq = True
            self._dlq.enqueue(state, reason="max_attempts_exceeded")
            return
        # 次の試行時刻をバックオフで計算する
        state.next_attempt_at = self._backoff.compute_next_attempt_at(state.attempt_count, now)
        logger.warning(
            "RetryScheduler: 送信失敗 notification_id=%s attempt=%d/%d next_at=%.1f error=%s",
            state.notification_id, state.attempt_count, self._policy.max_attempts,
            state.next_attempt_at, state.last_error
        )

    def _try_send(self, notification_id: str, payload: dict[str, Any]) -> bool:
        """実際の通知送信を試みる（内部メソッド）。"""
        # 送信関数が登録されていない場合は dry-run として成功扱いにする
        if self._send_fn is None:
            logger.debug("RetryScheduler: dry-run mode - 送信スキップ notification_id=%s", notification_id)
            return True
        # 登録された送信関数を呼び出す
        return self._send_fn(notification_id, payload)

    def get_queue_stats(self) -> dict[str, Any]:
        """キューの現在統計情報を辞書形式で返す。"""
        # キュー内の各状態別件数を集計する
        total = len(self._queue)
        pending = sum(1 for s in self._queue.values() if not s.succeeded and not s.in_dlq)
        succeeded = sum(1 for s in self._queue.values() if s.succeeded)
        in_dlq = sum(1 for s in self._queue.values() if s.in_dlq)
        return {
            "total": total,
            "pending": pending,
            "succeeded": succeeded,
            "in_dlq": in_dlq,
            "success_count": self._success_count,
            "failure_count": self._failure_count,
            "dlq_count": self.dlq_count,
            "circuit_breaker": self._circuit_breaker.get_metrics(),
        }

    def remove_completed(self) -> int:
        """完了済み（succeeded または in_dlq）の通知をキューから除去する。

        Returns:
            除去した通知の件数
        """
        # 完了済みエントリの ID を収集する
        completed_ids = [
            nid for nid, state in self._queue.items()
            if state.succeeded or state.in_dlq
        ]
        # キューから除去する
        for nid in completed_ids:
            del self._queue[nid]
        return len(completed_ids)

    def get_state(self, notification_id: str) -> RetryState | None:
        """指定 ID の RetryState を返す。存在しない場合は None を返す。"""
        return self._queue.get(notification_id)

    def cancel(self, notification_id: str) -> bool:
        """指定 ID の通知をキューからキャンセルして除去する。

        Returns:
            キャンセル成功の場合 True、存在しなかった場合 False
        """
        # キューに存在する場合のみ除去する
        if notification_id in self._queue:
            del self._queue[notification_id]
            logger.info("RetryScheduler: キャンセル notification_id=%s", notification_id)
            return True
        return False

    def run_once(self) -> dict[str, Any]:
        """1 サイクル分のリトライ処理を実行して統計を返す。"""
        # 実行時刻に達した通知を処理する
        processed = self.process_due()
        # 完了済みエントリを除去する
        removed = self.remove_completed()
        # 統計情報を返す
        stats = self.get_queue_stats()
        stats["processed_this_cycle"] = len(processed)
        stats["removed_this_cycle"] = removed
        return stats
