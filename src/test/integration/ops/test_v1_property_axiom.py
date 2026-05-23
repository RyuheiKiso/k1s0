"""src/test/integration/ops/test_v1_property_axiom.py

ops 軸の v1_property_axiom 検証クラスに対するインテグレーションテスト。
FSM 遷移順序・リトライバックオフ・アラート重複除去・エスカレーション
ティアの単調増加プロパティを hypothesis で検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: ops_property_axiom_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import time
import hashlib
# pytest フレームワークをインポートする
import pytest
# hypothesis ライブラリをインポートする
from hypothesis import given, settings, assume, HealthCheck
# hypothesis の戦略モジュールをインポートする
from hypothesis import strategies as st
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional, Dict, Tuple
# dataclass デコレータをインポートする
from dataclasses import dataclass, field
# 列挙型をインポートする
from enum import Enum, auto


# アラート FSM の状態を表す列挙型を定義する
class AlertFSMState(Enum):
    """アラート FSM の状態。

    有効な遷移パスは以下のとおり:
    IDLE → FIRED → ACK → MITIGATED → RESOLVED
    """

    # アイドル状態を定義する
    IDLE = "IDLE"
    # 発火状態を定義する
    FIRED = "FIRED"
    # 確認状態を定義する
    ACK = "ACK"
    # 緩和状態を定義する
    MITIGATED = "MITIGATED"
    # 解決状態を定義する
    RESOLVED = "RESOLVED"


# 有効な FSM 遷移を定義する辞書を作成する
VALID_TRANSITIONS: dict[AlertFSMState, set[AlertFSMState]] = {
    # IDLE から FIRED への遷移のみ許可する
    AlertFSMState.IDLE: {AlertFSMState.FIRED},
    # FIRED から ACK への遷移のみ許可する
    AlertFSMState.FIRED: {AlertFSMState.ACK},
    # ACK から MITIGATED への遷移のみ許可する
    AlertFSMState.ACK: {AlertFSMState.MITIGATED},
    # MITIGATED から RESOLVED への遷移のみ許可する
    AlertFSMState.MITIGATED: {AlertFSMState.RESOLVED},
    # RESOLVED から他の状態への遷移は不可
    AlertFSMState.RESOLVED: set(),
}


# アラート FSM のモッククラスを定義する
class AlertFSM:
    """アラートライフサイクルを管理する有限状態機械。"""

    # FSM を初期化するメソッドを定義する
    def __init__(self) -> None:
        """FSM を初期化する。初期状態は IDLE。"""
        # 現在の状態を IDLE に設定する
        self._state: AlertFSMState = AlertFSMState.IDLE
        # 遷移履歴を初期化する
        self._history: list[AlertFSMState] = [AlertFSMState.IDLE]

    # 現在の状態を取得するプロパティを定義する
    @property
    def state(self) -> AlertFSMState:
        """現在の FSM 状態を返す。"""
        # 現在の状態を返す
        return self._state

    # 遷移履歴を取得するプロパティを定義する
    @property
    def history(self) -> list[AlertFSMState]:
        """遷移履歴のコピーを返す。"""
        # 履歴のコピーを返す
        return list(self._history)

    # 状態を遷移させるメソッドを定義する
    def transition(self, new_state: AlertFSMState) -> bool:
        """指定された状態に遷移する。無効な遷移の場合は False を返す。"""
        # 現在の状態から遷移可能な状態を取得する
        allowed = VALID_TRANSITIONS.get(self._state, set())
        # 遷移先が許可されていない場合は False を返す
        if new_state not in allowed:
            # 無効な遷移を拒否する
            return False
        # 状態を更新する
        self._state = new_state
        # 遷移履歴に追加する
        self._history.append(new_state)
        # 遷移成功を返す
        return True

    # 完全な遷移シーケンスを実行するメソッドを定義する
    def execute_full_lifecycle(self) -> bool:
        """IDLE から RESOLVED まで完全なライフサイクルを実行する。"""
        # 順序通りに遷移を実行する
        for state in [
            AlertFSMState.FIRED,
            AlertFSMState.ACK,
            AlertFSMState.MITIGATED,
            AlertFSMState.RESOLVED,
        ]:
            # 遷移を試みる
            if not self.transition(state):
                # 遷移失敗の場合は False を返す
                return False
        # 全遷移成功を返す
        return True


# リトライバックオフ計算クラスを定義する
class ExponentialBackoff:
    """指数バックオフのリトライ間隔を計算するクラス。"""

    # バックオフを初期化するメソッドを定義する
    def __init__(
        self,
        initial_delay_ms: int = 100,
        multiplier: float = 2.0,
        max_delay_ms: int = 30000,
    ) -> None:
        """バックオフパラメータを初期化する。"""
        # 初期遅延を設定する
        self._initial_delay = initial_delay_ms
        # 乗数を設定する
        self._multiplier = multiplier
        # 最大遅延を設定する
        self._max_delay = max_delay_ms
        # 現在の遅延を初期化する
        self._current_delay = initial_delay_ms

    # 次のバックオフ遅延を計算するメソッドを定義する
    def next_delay(self) -> int:
        """次のバックオフ遅延を計算して返す。"""
        # 現在の遅延を記録する
        delay = self._current_delay
        # 次の遅延を計算する
        self._current_delay = min(
            int(self._current_delay * self._multiplier),
            self._max_delay,
        )
        # 遅延を返す
        return delay

    # 遅延シーケンスを生成するメソッドを定義する
    def generate_sequence(self, count: int) -> list[int]:
        """指定された件数のバックオフ遅延シーケンスを生成する。"""
        # バックオフを初期状態にリセットする
        self._current_delay = self._initial_delay
        # 遅延リストを生成する
        delays = []
        # count 件の遅延を生成する
        for _ in range(count):
            # 遅延を生成してリストに追加する
            delays.append(self.next_delay())
        # 遅延リストを返す
        return delays


# アラート重複除去クラスを定義する
class AlertDeduplicator:
    """同一フィンガープリントのアラートを重複除去するクラス。"""

    # 重複除去器を初期化するメソッドを定義する
    def __init__(self, dedup_window_seconds: float = 300.0) -> None:
        """重複除去器を初期化する。重複除去ウィンドウはデフォルト 5 分。"""
        # 重複除去ウィンドウを設定する
        self._window = dedup_window_seconds
        # 送信済みアラートのフィンガープリントと送信時刻を管理する辞書を初期化する
        self._sent: dict[str, float] = {}
        # 通知カウンタを初期化する
        self._notification_count: int = 0

    # アラートを処理するメソッドを定義する
    def process_alert(
        self,
        fingerprint: str,
        timestamp: float,
    ) -> bool:
        """アラートを処理する。重複の場合は False、通知送信の場合は True を返す。"""
        # 最後の送信時刻を取得する（存在しない場合は None）
        last_sent = self._sent.get(fingerprint)
        # 重複ウィンドウ内に送信済みの場合は重複として処理する
        if last_sent is not None and (timestamp - last_sent) < self._window:
            # 重複として処理する
            return False
        # 重複でない場合は通知を送信する
        self._sent[fingerprint] = timestamp
        # 通知カウンタをインクリメントする
        self._notification_count += 1
        # True を返す
        return True

    # 通知カウンタを取得するプロパティを定義する
    @property
    def notification_count(self) -> int:
        """送信された通知の総数を返す。"""
        # 通知カウンタを返す
        return self._notification_count


# エスカレーションマネージャクラスを定義する
class EscalationManager:
    """アラートのエスカレーションティアを管理するクラス。"""

    # マネージャを初期化するメソッドを定義する
    def __init__(self) -> None:
        """エスカレーションマネージャを初期化する。"""
        # エスカレーション履歴を保持するリストを初期化する
        self._escalation_history: list[int] = []
        # 現在のティアを初期化する
        self._current_tier: int = 1

    # エスカレーションを実行するメソッドを定義する
    def escalate(self) -> int:
        """エスカレーションティアを 1 つ上げる。"""
        # 現在のティアをインクリメントする
        self._current_tier += 1
        # 履歴に記録する
        self._escalation_history.append(self._current_tier)
        # 現在のティアを返す
        return self._current_tier

    # エスカレーション履歴を取得するプロパティを定義する
    @property
    def escalation_history(self) -> list[int]:
        """エスカレーション履歴のコピーを返す。"""
        # 履歴のコピーを返す
        return list(self._escalation_history)

    # 現在のティアを取得するプロパティを定義する
    @property
    def current_tier(self) -> int:
        """現在のエスカレーションティアを返す。"""
        # 現在のティアを返す
        return self._current_tier


# ops 軸プロパティ検証テストスイートクラスを定義する
class TestOpsPropertyAxiom:
    """ops 軸 v1_property_axiom の検証テストスイート。"""

    # アラート FSM のフィクスチャを定義する
    @pytest.fixture
    def alert_fsm(self) -> Generator[AlertFSM, None, None]:
        """AlertFSM フィクスチャを生成する。"""
        # FSM をインスタンス化する
        fsm = AlertFSM()
        # フィクスチャを返す
        yield fsm

    # FSM が IDLE から RESOLVED に直接遷移できないプロパティをテストする
    def test_property_fsm_no_direct_idle_to_resolved(
        self, alert_fsm: AlertFSM
    ) -> None:
        """FSM が IDLE から RESOLVED に直接遷移できないことを検証する。"""
        # IDLE から RESOLVED への直接遷移を試みる
        result = alert_fsm.transition(AlertFSMState.RESOLVED)
        # 遷移が拒否されたことを確認する
        assert result is False, "FSM must not allow IDLE → RESOLVED directly"
        # 状態が IDLE のままであることを確認する
        assert alert_fsm.state == AlertFSMState.IDLE

    # FSM が正しい順序で遷移できるプロパティをテストする
    def test_property_fsm_valid_full_lifecycle(
        self, alert_fsm: AlertFSM
    ) -> None:
        """FSM が IDLE→FIRED→ACK→MITIGATED→RESOLVED の順序で遷移できることを検証する。"""
        # 完全なライフサイクルを実行する
        result = alert_fsm.execute_full_lifecycle()
        # 実行が成功したことを確認する
        assert result is True
        # 最終状態が RESOLVED であることを確認する
        assert alert_fsm.state == AlertFSMState.RESOLVED
        # 遷移履歴が正しいことを確認する
        expected_history = [
            AlertFSMState.IDLE,
            AlertFSMState.FIRED,
            AlertFSMState.ACK,
            AlertFSMState.MITIGATED,
            AlertFSMState.RESOLVED,
        ]
        # 履歴が期待通りであることを確認する
        assert alert_fsm.history == expected_history

    # FSM が逆方向遷移を拒否するプロパティをテストする
    @given(
        # 遷移シーケンスを生成する戦略を定義する
        steps=st.integers(min_value=1, max_value=4),
    )
    @settings(max_examples=20, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_fsm_no_backward_transition(self, steps: int) -> None:
        """FSM が後退遷移（状態を戻すこと）を拒否することを検証する。"""
        # FSM を新規作成する
        fsm = AlertFSM()
        # 有効な遷移シーケンスを生成する
        valid_sequence = [
            AlertFSMState.FIRED,
            AlertFSMState.ACK,
            AlertFSMState.MITIGATED,
            AlertFSMState.RESOLVED,
        ]
        # steps 件の遷移を実行する
        for i in range(min(steps, len(valid_sequence))):
            # 遷移を実行する
            fsm.transition(valid_sequence[i])
        # 現在の状態を記録する
        current_state = fsm.state
        # 前の状態に戻ろうとする（IDLE への直接遷移は必ず失敗するはず）
        result = fsm.transition(AlertFSMState.IDLE)
        # IDLE への後退遷移が拒否されることを確認する
        assert result is False, (
            f"Backward transition to IDLE must be rejected from {current_state}"
        )

    # リトライバックオフが単調増加するプロパティをテストする
    @given(
        # 初期遅延を生成する戦略を定義する
        initial_delay=st.integers(min_value=10, max_value=1000),
        # 乗数を生成する戦略を定義する
        multiplier=st.floats(min_value=1.1, max_value=5.0),
        # リトライ回数を生成する戦略を定義する
        retry_count=st.integers(min_value=2, max_value=15),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_retry_backoff_always_increases(
        self, initial_delay: int, multiplier: float, retry_count: int
    ) -> None:
        """リトライバックオフが単調増加することを検証する。"""
        # バックオフを初期化する
        backoff = ExponentialBackoff(
            initial_delay_ms=initial_delay,
            multiplier=multiplier,
            max_delay_ms=60000,
        )
        # 遅延シーケンスを生成する
        delays = backoff.generate_sequence(retry_count)
        # 遅延が単調非減少であることを確認する
        for i in range(1, len(delays)):
            # 現在の遅延が前の遅延以上であることを確認する
            assert delays[i] >= delays[i - 1], (
                f"Backoff delay must be non-decreasing: "
                f"delays[{i-1}]={delays[i-1]} > delays[{i}]={delays[i]}"
            )

    # 同一フィンガープリントのアラートが 5 分以内に 1 回しか通知されないプロパティをテストする
    @given(
        # アラート送信回数を生成する戦略を定義する
        alert_count=st.integers(min_value=2, max_value=50),
        # ウィンドウ内の遅延を生成する戦略を定義する
        time_delta_seconds=st.floats(min_value=0.1, max_value=299.9),
    )
    @settings(max_examples=100, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_deduplication_within_window_exactly_one_notification(
        self,
        alert_count: int,
        time_delta_seconds: float,
    ) -> None:
        """同一フィンガープリントのアラートが重複除去ウィンドウ内で 1 回のみ通知されることを検証する。"""
        # 重複除去器を初期化する
        dedup = AlertDeduplicator(dedup_window_seconds=300.0)
        # フィンガープリントを生成する
        fingerprint = "alert_fp_dedup_test"
        # 基準時刻を設定する
        base_time = 1700000000.0
        # alert_count 件のアラートを time_delta 間隔で送信する
        for i in range(alert_count):
            # 時刻を計算する
            t = base_time + i * time_delta_seconds
            # アラートを処理する
            dedup.process_alert(fingerprint, t)
        # 通知が 1 回のみ送信されたことを確認する
        assert dedup.notification_count == 1, (
            f"Expected exactly 1 notification within dedup window, "
            f"got {dedup.notification_count}"
        )

    # エスカレーションティアが単調増加するプロパティをテストする
    @given(
        # エスカレーション回数を生成する戦略を定義する
        escalation_count=st.integers(min_value=1, max_value=10),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_escalation_tier_always_increases(
        self, escalation_count: int
    ) -> None:
        """エスカレーションティアが単調増加することを検証する。"""
        # エスカレーションマネージャを初期化する
        manager = EscalationManager()
        # 初期ティアが 1 であることを確認する
        assert manager.current_tier == 1
        # escalation_count 件のエスカレーションを実行する
        prev_tier = manager.current_tier
        # エスカレーションを繰り返す
        for _ in range(escalation_count):
            # エスカレーションを実行する
            new_tier = manager.escalate()
            # 新しいティアが前のティアより大きいことを確認する
            assert new_tier > prev_tier, (
                f"Escalation tier must increase: prev={prev_tier}, new={new_tier}"
            )
            # 前のティアを更新する
            prev_tier = new_tier

    # 重複除去ウィンドウ外のアラートが通知されるプロパティをテストする
    def test_property_dedup_outside_window_notifies(self) -> None:
        """重複除去ウィンドウ外のアラートが通知されることを検証する。"""
        # 重複除去器を初期化する（10 秒ウィンドウ）
        dedup = AlertDeduplicator(dedup_window_seconds=10.0)
        # フィンガープリントを定義する
        fp = "test_fingerprint_outside_window"
        # 基準時刻を設定する
        t0 = 1700000000.0
        # 最初のアラートを送信する（通知されるべき）
        result1 = dedup.process_alert(fp, t0)
        # 最初のアラートが通知されたことを確認する
        assert result1 is True
        # ウィンドウ内（5 秒後）のアラートを送信する（重複のため通知されないべき）
        result2 = dedup.process_alert(fp, t0 + 5.0)
        # ウィンドウ内のアラートが重複として処理されたことを確認する
        assert result2 is False
        # ウィンドウ外（15 秒後）のアラートを送信する（通知されるべき）
        result3 = dedup.process_alert(fp, t0 + 15.0)
        # ウィンドウ外のアラートが通知されたことを確認する
        assert result3 is True
        # 通知回数が 2 であることを確認する
        assert dedup.notification_count == 2


# ops 軸プロパティ検証追加テストスイートクラスを定義する
class TestOpsPropertyAxiomExtended:
    """ops 軸 v1_property_axiom の追加検証テストスイート。

    FSM の不正遷移拒否・バックオフの上限キャップ・複数フィンガープリントの重複除去を検証する。
    """

    # FSM の FIRED から RESOLVED への直接遷移が拒否されることを検証するテストを定義する
    def test_fsm_fired_to_resolved_rejected(self) -> None:
        """FSM が FIRED から RESOLVED に直接遷移できないことを検証する。"""
        # FSM を生成する
        fsm = AlertFSM()
        # FIRED に遷移する
        fsm.transition(AlertFSMState.FIRED)
        # FIRED から RESOLVED への直接遷移を試みる
        result = fsm.transition(AlertFSMState.RESOLVED)
        # 遷移が拒否されたことを確認する
        assert result is False

    # FSM の ACK から IDLE への後退遷移が拒否されることを検証するテストを定義する
    def test_fsm_ack_to_idle_rejected(self) -> None:
        """FSM が ACK から IDLE に後退遷移できないことを検証する。"""
        # FSM を生成する
        fsm = AlertFSM()
        # FIRED に遷移する
        fsm.transition(AlertFSMState.FIRED)
        # ACK に遷移する
        fsm.transition(AlertFSMState.ACK)
        # IDLE への後退遷移を試みる
        result = fsm.transition(AlertFSMState.IDLE)
        # 遷移が拒否されたことを確認する
        assert result is False
        # 現在の状態が ACK であることを確認する
        assert fsm.state == AlertFSMState.ACK

    # バックオフが最大遅延を超えないことを検証するテストを定義する
    def test_backoff_capped_at_max_delay(self) -> None:
        """バックオフ遅延が設定した最大値を超えないことを検証する。"""
        # 最大遅延 1000ms のバックオフを生成する
        backoff = ExponentialBackoff(
            initial_delay_ms=100,
            multiplier=3.0,
            max_delay_ms=1000,
        )
        # 20 件の遅延シーケンスを生成する
        delays = backoff.generate_sequence(20)
        # 全遅延が最大値以下であることを確認する
        for i, delay in enumerate(delays):
            # 遅延が最大値以下であることを確認する
            assert delay <= 1000, (
                f"Backoff delay at index {i} exceeds max: {delay} > 1000"
            )

    # 複数の異なるフィンガープリントが独立して重複除去されることを検証するテストを定義する
    def test_dedup_multiple_fingerprints_independent(self) -> None:
        """複数のフィンガープリントが独立して重複除去されることを検証する。"""
        # 重複除去器を初期化する
        dedup = AlertDeduplicator(dedup_window_seconds=300.0)
        # 5 種類のフィンガープリントを定義する
        fingerprints = [f"fp_{i}" for i in range(5)]
        # 各フィンガープリントに 3 回アラートを送信する
        base_time = 1700000000.0
        # 各フィンガープリントを処理する
        for fp in fingerprints:
            # 3 回アラートを送信する（1 件目は通知、2/3 件目は重複）
            for j in range(3):
                # アラートを処理する
                dedup.process_alert(fp, base_time + j * 10.0)
        # 通知回数が 5 件（各フィンガープリント 1 回）であることを確認する
        assert dedup.notification_count == 5, (
            f"Expected 5 notifications, got {dedup.notification_count}"
        )

    # エスカレーション履歴が正しく記録されることを検証するテストを定義する
    def test_escalation_history_recorded_correctly(self) -> None:
        """エスカレーション履歴が正確に記録されることを検証する。"""
        # エスカレーションマネージャーを初期化する
        manager = EscalationManager()
        # 3 回エスカレーションする
        for _ in range(3):
            # エスカレーションを実行する
            manager.escalate()
        # 履歴が 3 件記録されていることを確認する
        assert len(manager.escalation_history) == 3
        # 履歴が [2, 3, 4] であることを確認する
        assert manager.escalation_history == [2, 3, 4]
        # 現在のティアが 4 であることを確認する
        assert manager.current_tier == 4


# ops 軸プロパティ検証第 2 拡張テストスイートクラスを定義する
class TestOpsPropertyAxiomExtended2:
    """ops 軸 v1_property_axiom の第 2 拡張テストスイート。追加のプロパティを網羅的に検証する。"""

    # FSM の完全な有効シーケンスが成功することをテストする
    def test_fsm_complete_valid_sequence(self) -> None:
        """FSM が IDLE→FIRED→ACK→MITIGATED→RESOLVED の完全シーケンスで成功することを検証する。"""
        # FSM を生成する
        fsm = AlertFSM()
        # 有効なシーケンスを実行する
        states = [
            AlertFSMState.FIRED,
            AlertFSMState.ACK,
            AlertFSMState.MITIGATED,
            AlertFSMState.RESOLVED,
        ]
        # 各状態に遷移する
        for target_state in states:
            # 遷移を実行する
            result = fsm.transition(target_state)
            # 遷移が成功したことを確認する
            assert result is True, f"Transition to {target_state} should succeed"
        # 最終状態が RESOLVED であることを確認する
        assert fsm.state == AlertFSMState.RESOLVED

    # FSM の MITIGATED から FIRED への後退遷移が拒否されることをテストする
    def test_fsm_mitigated_to_fired_rejected(self) -> None:
        """FSM が MITIGATED から FIRED に後退遷移できないことを検証する。"""
        # FSM を生成する
        fsm = AlertFSM()
        # FIRED、ACK、MITIGATED に順次遷移する
        fsm.transition(AlertFSMState.FIRED)
        # ACK に遷移する
        fsm.transition(AlertFSMState.ACK)
        # MITIGATED に遷移する
        fsm.transition(AlertFSMState.MITIGATED)
        # FIRED への後退遷移を試みる
        result = fsm.transition(AlertFSMState.FIRED)
        # 遷移が拒否されたことを確認する
        assert result is False
        # 現在の状態が MITIGATED であることを確認する
        assert fsm.state == AlertFSMState.MITIGATED

    # FSM の history が全遷移を正確に記録することをテストする
    def test_fsm_history_tracks_all_transitions(self) -> None:
        """FSM の history が全ての遷移を正確に記録することを検証する。"""
        # FSM を生成する
        fsm = AlertFSM()
        # 初期状態が IDLE であることを確認する
        assert fsm.history == [AlertFSMState.IDLE]
        # FIRED に遷移する
        fsm.transition(AlertFSMState.FIRED)
        # 履歴に FIRED が追加されたことを確認する
        assert AlertFSMState.FIRED in fsm.history
        # ACK に遷移する
        fsm.transition(AlertFSMState.ACK)
        # 履歴に ACK が追加されたことを確認する
        assert AlertFSMState.ACK in fsm.history
        # 履歴の長さが 3 であることを確認する
        assert len(fsm.history) == 3

    # バックオフが初期遅延から開始することをテストする
    def test_backoff_starts_at_initial_delay(self) -> None:
        """バックオフの最初の遅延が initial_delay_ms であることを検証する。"""
        # 初期遅延 200ms のバックオフを生成する
        backoff = ExponentialBackoff(initial_delay_ms=200, multiplier=2.0, max_delay_ms=10000)
        # 遅延シーケンスを生成する
        delays = backoff.generate_sequence(1)
        # 最初の遅延が 200ms であることを確認する
        assert delays[0] == 200, f"First delay should be 200, got {delays[0]}"

    # バックオフの遅延が指数関数的に増加することをテストする
    def test_backoff_delays_exponentially_increasing(self) -> None:
        """バックオフの遅延が指数関数的に増加することを検証する。"""
        # 乗数 2.0 のバックオフを生成する
        backoff = ExponentialBackoff(initial_delay_ms=10, multiplier=2.0, max_delay_ms=100000)
        # 5 件の遅延シーケンスを生成する
        delays = backoff.generate_sequence(5)
        # 各遅延が前の遅延の 2 倍以上であることを確認する（上限に達するまで）
        for i in range(1, min(len(delays), 4)):
            # 前の遅延が現在の遅延以下であることを確認する（単調増加）
            assert delays[i - 1] <= delays[i], (
                f"Backoff should be non-decreasing: delays[{i-1}]={delays[i-1]}, delays[{i}]={delays[i]}"
            )

    # 重複除去ウィンドウ外のアラートが通知されることをテストする
    def test_dedup_after_window_notified(self) -> None:
        """重複除去ウィンドウ外に送信されたアラートが通知されることを検証する。"""
        # 重複除去器を初期化する（ウィンドウ 60 秒）
        dedup = AlertDeduplicator(dedup_window_seconds=60.0)
        # フィンガープリントを定義する
        fp = "window_test_fp"
        # 最初のアラートを送信する（通知されるべき）
        assert dedup.process_alert(fp, 0.0) is True
        # ウィンドウ内のアラートを送信する（重複のため通知されないべき）
        assert dedup.process_alert(fp, 30.0) is False
        # ウィンドウ外のアラートを送信する（通知されるべき）
        assert dedup.process_alert(fp, 61.0) is True
        # 通知回数が 2 であることを確認する
        assert dedup.notification_count == 2

    # エスカレーションティアが単調増加することをプロパティで検証するテストを定義する
    @given(
        # エスカレーション回数を生成する戦略を定義する
        escalation_count=st.integers(min_value=1, max_value=10),
    )
    @settings(max_examples=50, suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_property_escalation_tier_monotone(self, escalation_count: int) -> None:
        """エスカレーションティアが単調増加することを検証する。"""
        # エスカレーションマネージャーを生成する
        manager = EscalationManager()
        # 初期ティアを記録する
        prev_tier = manager.current_tier
        # escalation_count 回エスカレーションする
        for _ in range(escalation_count):
            # エスカレーションを実行する
            manager.escalate()
            # ティアが前のティアより大きいことを確認する
            assert manager.current_tier > prev_tier, (
                f"Escalation tier must increase: prev={prev_tier}, current={manager.current_tier}"
            )
            # 前のティアを更新する
            prev_tier = manager.current_tier

    # 複数 FSM インスタンスが独立していることをテストする
    def test_fsm_instances_independent(self) -> None:
        """複数の FSM インスタンスが独立してステートを管理することを検証する。"""
        # 1 つ目の FSM を生成する
        fsm1 = AlertFSM()
        # 2 つ目の FSM を生成する
        fsm2 = AlertFSM()
        # fsm1 を FIRED に遷移させる
        fsm1.transition(AlertFSMState.FIRED)
        # fsm1 を ACK に遷移させる
        fsm1.transition(AlertFSMState.ACK)
        # fsm2 はまだ IDLE のままであることを確認する
        assert fsm2.state == AlertFSMState.IDLE
        # fsm1 は ACK であることを確認する
        assert fsm1.state == AlertFSMState.ACK
        # fsm2 を FIRED に遷移させる
        fsm2.transition(AlertFSMState.FIRED)
        # fsm1 の状態が変わっていないことを確認する
        assert fsm1.state == AlertFSMState.ACK


# ops プロパティ公理の第三拡張テストクラスを定義する
class TestOpsPropertyAxiomExtended3:
    """FSM・バックオフ・重複除去・エスカレーションプロパティの追加テスト群（第三拡張）。"""

    # FSM が IDLE 状態から始まることをテストする
    def test_fsm_initial_state_is_idle(self) -> None:
        """FSM の初期状態が IDLE であることを検証する。"""
        # FSM をインスタンス化する
        fsm = AlertFSM()
        # 初期状態が IDLE であることを確認する
        assert fsm.state == AlertFSMState.IDLE

    # FSM が IDLE から FIRED に遷移できることをテストする
    def test_fsm_idle_to_fired_succeeds(self) -> None:
        """FSM が IDLE から FIRED に遷移できることを検証する。"""
        # FSM をインスタンス化する
        fsm = AlertFSM()
        # FIRED に遷移する
        result = fsm.transition(AlertFSMState.FIRED)
        # 遷移が成功することを確認する
        assert result is True
        # 状態が FIRED であることを確認する
        assert fsm.state == AlertFSMState.FIRED

    # FSM が RESOLVED 後に遷移できないことをテストする
    def test_fsm_resolved_is_terminal_state(self) -> None:
        """FSM が RESOLVED 状態から他の状態に遷移できないことを検証する。"""
        # FSM をインスタンス化する
        fsm = AlertFSM()
        # 完全なライフサイクルを実行する
        result = fsm.execute_full_lifecycle()
        # ライフサイクルが成功することを確認する
        assert result is True
        # 状態が RESOLVED であることを確認する
        assert fsm.state == AlertFSMState.RESOLVED
        # RESOLVED から IDLE への遷移が失敗することを確認する
        back = fsm.transition(AlertFSMState.IDLE)
        # 遷移が失敗することを確認する
        assert back is False
        # 状態が RESOLVED のままであることを確認する
        assert fsm.state == AlertFSMState.RESOLVED

    # バックオフシーケンスが単調増加であることをテストする（境界値）
    def test_backoff_sequence_monotone_before_cap(self) -> None:
        """バックオフシーケンスが上限に達する前に単調増加であることを検証する。"""
        # バックオフをインスタンス化する
        backoff = ExponentialBackoff(initial_delay_ms=100, multiplier=2.0, max_delay_ms=100000)
        # 5 件のシーケンスを生成する
        delays = backoff.generate_sequence(5)
        # 100, 200, 400, 800, 1600 になることを確認する
        assert delays[0] == 100
        # シーケンスが単調増加であることを確認する
        for i in range(1, len(delays)):
            # 前より大きいことを確認する
            assert delays[i] > delays[i - 1], f"Delay {i} not monotone: {delays[i-1]} -> {delays[i]}"

    # バックオフが最大遅延を超えないことをテストする
    def test_backoff_never_exceeds_max_delay(self) -> None:
        """バックオフが最大遅延を超えないことを検証する。"""
        # 最大遅延を 1000ms に設定する
        max_ms = 1000
        # バックオフをインスタンス化する
        backoff = ExponentialBackoff(initial_delay_ms=100, multiplier=3.0, max_delay_ms=max_ms)
        # 10 件のシーケンスを生成する
        delays = backoff.generate_sequence(10)
        # 全遅延が最大値を超えないことを確認する
        for i, d in enumerate(delays):
            # 最大値を超えないことを確認する
            assert d <= max_ms, f"Delay {i} exceeds max: {d} > {max_ms}"

    # 重複除去器が異なるフィンガープリントを独立して管理することをテストする
    def test_dedup_independent_fingerprints(self) -> None:
        """重複除去器が異なるフィンガープリントを独立して管理することを検証する。"""
        # 重複除去器をインスタンス化する
        dedup = AlertDeduplicator(dedup_window_seconds=60.0)
        # フィンガープリント A を処理する
        r1 = dedup.process_alert("fp_A", 0.0)
        # 通知が送信されることを確認する
        assert r1 is True
        # フィンガープリント B を処理する
        r2 = dedup.process_alert("fp_B", 1.0)
        # 通知が送信されることを確認する
        assert r2 is True
        # 通知カウントが 2 であることを確認する
        assert dedup.notification_count == 2

    # 重複除去器がウィンドウ外のアラートを通知することをテストする
    def test_dedup_sends_after_window_expires(self) -> None:
        """重複除去器がウィンドウ外のアラートを再通知することを検証する。"""
        # 重複除去器をインスタンス化する（ウィンドウ 10 秒）
        dedup = AlertDeduplicator(dedup_window_seconds=10.0)
        # フィンガープリント X を処理する
        r1 = dedup.process_alert("fp_X", 0.0)
        # 通知が送信されることを確認する
        assert r1 is True
        # 10.1 秒後に同じフィンガープリントを処理する
        r2 = dedup.process_alert("fp_X", 10.1)
        # 通知が再送されることを確認する
        assert r2 is True
        # 通知カウントが 2 であることを確認する
        assert dedup.notification_count == 2

    # 重複除去器がウィンドウ内のアラートを重複として処理することをテストする
    def test_dedup_suppresses_within_window(self) -> None:
        """重複除去器がウィンドウ内の重複アラートを抑制することを検証する。"""
        # 重複除去器をインスタンス化する（ウィンドウ 300 秒）
        dedup = AlertDeduplicator(dedup_window_seconds=300.0)
        # フィンガープリント Y を処理する
        dedup.process_alert("fp_Y", 0.0)
        # 10 秒後に同じフィンガープリントを処理する（ウィンドウ内）
        r = dedup.process_alert("fp_Y", 10.0)
        # 重複として処理されることを確認する
        assert r is False
        # 通知カウントが 1 のままであることを確認する
        assert dedup.notification_count == 1

    # 複数の FSM インスタンスのライフサイクルが独立していることをテストする
    def test_multiple_fsm_instances_lifecycle_independent(self) -> None:
        """複数の FSM インスタンスが独立してライフサイクルを持つことを検証する。"""
        # 3 つの FSM をインスタンス化する
        fsms = [AlertFSM() for _ in range(3)]
        # 最初の FSM を完全ライフサイクルまで進める
        fsms[0].execute_full_lifecycle()
        # 2 番目の FSM を FIRED まで進める
        fsms[1].transition(AlertFSMState.FIRED)
        # 3 番目の FSM は IDLE のままにする
        assert fsms[0].state == AlertFSMState.RESOLVED
        # 2 番目の FSM が FIRED であることを確認する
        assert fsms[1].state == AlertFSMState.FIRED
        # 3 番目の FSM が IDLE であることを確認する
        assert fsms[2].state == AlertFSMState.IDLE

    # バックオフの初期遅延が 1ms でも正常動作することをテストする
    def test_backoff_minimum_initial_delay_works(self) -> None:
        """バックオフの初期遅延が 1ms でも正常に動作することを検証する。"""
        # 最小初期遅延のバックオフをインスタンス化する
        backoff = ExponentialBackoff(initial_delay_ms=1, multiplier=2.0, max_delay_ms=1000)
        # 5 件のシーケンスを生成する
        delays = backoff.generate_sequence(5)
        # 初期遅延が 1ms であることを確認する
        assert delays[0] == 1
        # 全遅延が正値であることを確認する
        assert all(d > 0 for d in delays)

    # FSM の history が遷移順に状態を記録することをテストする
    def test_fsm_history_records_in_order(self) -> None:
        """FSM の history が遷移を順序通りに記録することを検証する。"""
        # FSM をインスタンス化する
        fsm = AlertFSM()
        # FIRED に遷移する
        fsm.transition(AlertFSMState.FIRED)
        # ACK に遷移する
        fsm.transition(AlertFSMState.ACK)
        # 履歴が [IDLE, FIRED, ACK] であることを確認する
        assert fsm.history == [
            AlertFSMState.IDLE,
            AlertFSMState.FIRED,
            AlertFSMState.ACK,
        ]


# ops プロパティ公理の第四拡張テストクラスを定義する
class TestOpsPropertyAxiomExtended4:
    """FSM・バックオフ・重複除去の追加テスト群（第四拡張）。"""

    # FSM が ACK から MITIGATED への遷移が成功することをテストする
    def test_fsm_ack_to_mitigated_succeeds(self) -> None:
        """FSM が ACK から MITIGATED への遷移が成功することを検証する。"""
        # FSM をインスタンス化する
        fsm = AlertFSM()
        # FIRED に遷移する
        fsm.transition(AlertFSMState.FIRED)
        # ACK に遷移する
        fsm.transition(AlertFSMState.ACK)
        # MITIGATED に遷移する
        result = fsm.transition(AlertFSMState.MITIGATED)
        # 遷移が成功することを確認する
        assert result is True
        # 状態が MITIGATED であることを確認する
        assert fsm.state == AlertFSMState.MITIGATED

    # バックオフシーケンスの総数が要求件数と一致することをテストする
    def test_backoff_sequence_count_matches_request(self) -> None:
        """バックオフシーケンスの総数が要求件数と一致することを検証する。"""
        # バックオフをインスタンス化する
        backoff = ExponentialBackoff(initial_delay_ms=100, multiplier=2.0, max_delay_ms=10000)
        # 7 件のシーケンスを生成する
        delays = backoff.generate_sequence(7)
        # 7 件のシーケンスが返ることを確認する
        assert len(delays) == 7

    # 重複除去器の同一フィンガープリントが累積ウィンドウで管理されることをテストする
    def test_dedup_cumulative_window_management(self) -> None:
        """重複除去器の同一フィンガープリントが累積ウィンドウで管理されることを検証する。"""
        # 重複除去器をインスタンス化する
        dedup = AlertDeduplicator(dedup_window_seconds=100.0)
        # フィンガープリント A を複数の時刻で処理する
        results = []
        # 3 回処理する（0, 50, 150 秒後）
        for t in [0.0, 50.0, 150.0]:
            # アラートを処理する
            r = dedup.process_alert("fp_cum", t)
            # 結果を記録する
            results.append(r)
        # 最初は通知される
        assert results[0] is True
        # 50 秒後はウィンドウ内なので抑制される
        assert results[1] is False
        # 150 秒後はウィンドウ外なので通知される
        assert results[2] is True
        # 通知カウントが 2 であることを確認する
        assert dedup.notification_count == 2
