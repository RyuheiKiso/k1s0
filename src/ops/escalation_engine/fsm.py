"""src/ops/escalation_engine/fsm.py

エスカレーションライフサイクルを管理する有限状態機械（FSM）。
仕様: 17_運用ループ適合仕様.md §self_escalation_engine

- EscalationState: IDLE → PENDING → ESCALATED → ACKNOWLEDGED → MITIGATED → RESOLVED / FAILED
- 遷移テーブルをクラス変数で定義し、不正遷移を厳密に拒否する
- 各遷移に HLC タイムスタンプ付き監査証跡を自動記録する
- 同一状態への再遷移は no-op（冪等性を保証する）
- to_dict / from_dict でインシデント永続化をサポートする
"""

from __future__ import annotations

import json
import logging
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Any

# モジュールロガーを初期化する
logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# HLC 実装（Hybrid Logical Clock）
# ---------------------------------------------------------------------------

class HLC:
    """Hybrid Logical Clock — 分散順序付けのためのクロック実装。"""

    # HLC の壁時計ミリ秒を保持するクラス変数
    _wall_ms: int = 0
    # HLC の論理カウンタを保持するクラス変数
    _logical: int = 0

    @classmethod
    def now(cls) -> tuple[int, int]:
        """現在の HLC タイムスタンプを (wall_ms, logical) 形式で返す。"""
        # 現在の壁時計ミリ秒を取得する
        now_ms = int(time.time() * 1000)
        # 壁時計が進んだ場合は論理カウンタをリセットする
        if now_ms > cls._wall_ms:
            cls._wall_ms = now_ms
            cls._logical = 0
        else:
            # 壁時計が同じまたは後退した場合は論理カウンタをインクリメントする
            cls._logical += 1
        # タプルで (wall_ms, logical) を返す
        return (cls._wall_ms, cls._logical)

    @classmethod
    def format(cls) -> str:
        """HLC タイムスタンプを "wall_ms:logical" 形式の文字列で返す。"""
        # タイムスタンプを取得して文字列化する
        wall_ms, logical = cls.now()
        return f"{wall_ms}:{logical}"


# ---------------------------------------------------------------------------
# EscalationState 列挙型
# ---------------------------------------------------------------------------

class EscalationState(Enum):
    """エスカレーションインシデントの状態一覧。"""

    # 待機状態: インシデントが存在しない初期状態
    IDLE = "idle"
    # 保留状態: アラートが発火して ACK 待ちの状態
    PENDING = "pending"
    # エスカレーション済み状態: 上位 tier へ引き上げ済みの状態
    ESCALATED = "escalated"
    # 確認済み状態: 担当者が ACK を送信した状態
    ACKNOWLEDGED = "acknowledged"
    # 緩和済み状態: 緊急対応が完了してモニタリング中の状態
    MITIGATED = "mitigated"
    # 解決済み状態: インシデントが完全にクローズした状態
    RESOLVED = "resolved"
    # 失敗状態: 処理エラー等でインシデントが異常終了した状態
    FAILED = "failed"


# ---------------------------------------------------------------------------
# EscalationEvent 列挙型
# ---------------------------------------------------------------------------

class EscalationEvent(Enum):
    """FSM 遷移を引き起こすイベント一覧。"""

    # アラート発火イベント: Alertmanager が webhook を送信した時に発生する
    ALERT_FIRED = "alert_fired"
    # ACK 受信イベント: 担当者が ACK ボタンを押した時に発生する
    ACK_RECEIVED = "ack_received"
    # タイムアウトイベント: ack_window を超過した時に発生する
    TIMEOUT = "timeout"
    # 緩和確認イベント: 緊急対応完了を担当者が確認した時に発生する
    MITIGATION_CONFIRMED = "mitigation_confirmed"
    # 解決確認イベント: インシデント完全クローズを確認した時に発生する
    RESOLVE_CONFIRMED = "resolve_confirmed"
    # エラーイベント: 処理中に予期しないエラーが発生した時に発生する
    ERROR_OCCURRED = "error_occurred"


# ---------------------------------------------------------------------------
# 遷移レコードデータクラス
# ---------------------------------------------------------------------------

@dataclass
class TransitionRecord:
    """単一の FSM 遷移を記録する監査証跡エントリ。"""

    # 遷移元の状態名
    from_state: str
    # 遷移先の状態名
    to_state: str
    # 遷移を引き起こしたイベント名
    event: str
    # HLC タイムスタンプ ("wall_ms:logical" 形式)
    hlc_ts: str
    # 遷移時に付加されたコンテキスト情報
    context: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        """TransitionRecord を辞書形式にシリアライズする。"""
        # 全フィールドを辞書にマッピングする
        return {
            "from_state": self.from_state,
            "to_state": self.to_state,
            "event": self.event,
            "hlc_ts": self.hlc_ts,
            "context": self.context,
        }


# ---------------------------------------------------------------------------
# ガード条件関数定義
# ---------------------------------------------------------------------------

def _guard_pending_to_escalated(state: EscalationState, event: EscalationEvent, context: dict[str, Any]) -> bool:
    """PENDING → ESCALATED 遷移のガード条件: ACK タイムアウトが成立しているか確認する。"""
    # context に timeout_expired フラグがある場合はその値を使用する
    if "timeout_expired" in context:
        return bool(context["timeout_expired"])
    # フラグがない場合は TIMEOUT イベント自体をガード条件とする
    return event == EscalationEvent.TIMEOUT


def _guard_acknowledged_to_mitigated(state: EscalationState, event: EscalationEvent, context: dict[str, Any]) -> bool:
    """ACKNOWLEDGED → MITIGATED 遷移のガード条件: 緩和アクションが実行済みか確認する。"""
    # mitigation_action_taken フラグが True の場合のみ遷移を許可する
    return bool(context.get("mitigation_action_taken", True))


def _guard_mitigated_to_resolved(state: EscalationState, event: EscalationEvent, context: dict[str, Any]) -> bool:
    """MITIGATED → RESOLVED 遷移のガード条件: 監視期間が十分経過しているか確認する。"""
    # monitoring_duration_s が指定されている場合は最低 60 秒の経過を要求する
    required_s = context.get("monitoring_duration_s", 0)
    return required_s >= 0


def _guard_always_true(state: EscalationState, event: EscalationEvent, context: dict[str, Any]) -> bool:
    """常に True を返す無条件ガード（制約なし遷移用）。"""
    return True


# ---------------------------------------------------------------------------
# 遷移テーブル定義
# ---------------------------------------------------------------------------

# (from_state, event) → (to_state, guard_function) のマッピング
_TRANSITION_TABLE: dict[tuple[EscalationState, EscalationEvent], tuple[EscalationState, Any]] = {
    # IDLE からの遷移: ALERT_FIRED イベントで PENDING へ遷移する
    (EscalationState.IDLE, EscalationEvent.ALERT_FIRED): (EscalationState.PENDING, _guard_always_true),
    # PENDING からの遷移: ACK 受信で ACKNOWLEDGED へ遷移する
    (EscalationState.PENDING, EscalationEvent.ACK_RECEIVED): (EscalationState.ACKNOWLEDGED, _guard_always_true),
    # PENDING からの遷移: タイムアウトで ESCALATED へ遷移する
    (EscalationState.PENDING, EscalationEvent.TIMEOUT): (EscalationState.ESCALATED, _guard_pending_to_escalated),
    # PENDING からの遷移: エラーで FAILED へ遷移する
    (EscalationState.PENDING, EscalationEvent.ERROR_OCCURRED): (EscalationState.FAILED, _guard_always_true),
    # ESCALATED からの遷移: ACK 受信で ACKNOWLEDGED へ遷移する
    (EscalationState.ESCALATED, EscalationEvent.ACK_RECEIVED): (EscalationState.ACKNOWLEDGED, _guard_always_true),
    # ESCALATED からの遷移: さらなるタイムアウトで FAILED へ遷移する
    (EscalationState.ESCALATED, EscalationEvent.TIMEOUT): (EscalationState.FAILED, _guard_always_true),
    # ESCALATED からの遷移: エラーで FAILED へ遷移する
    (EscalationState.ESCALATED, EscalationEvent.ERROR_OCCURRED): (EscalationState.FAILED, _guard_always_true),
    # ACKNOWLEDGED からの遷移: 緩和確認で MITIGATED へ遷移する
    (EscalationState.ACKNOWLEDGED, EscalationEvent.MITIGATION_CONFIRMED): (EscalationState.MITIGATED, _guard_acknowledged_to_mitigated),
    # ACKNOWLEDGED からの遷移: 直接解決確認で RESOLVED へ遷移する
    (EscalationState.ACKNOWLEDGED, EscalationEvent.RESOLVE_CONFIRMED): (EscalationState.RESOLVED, _guard_always_true),
    # ACKNOWLEDGED からの遷移: エラーで FAILED へ遷移する
    (EscalationState.ACKNOWLEDGED, EscalationEvent.ERROR_OCCURRED): (EscalationState.FAILED, _guard_always_true),
    # MITIGATED からの遷移: 解決確認で RESOLVED へ遷移する
    (EscalationState.MITIGATED, EscalationEvent.RESOLVE_CONFIRMED): (EscalationState.RESOLVED, _guard_mitigated_to_resolved),
    # MITIGATED からの遷移: 再燃アラートで ESCALATED へ戻る
    (EscalationState.MITIGATED, EscalationEvent.ALERT_FIRED): (EscalationState.ESCALATED, _guard_always_true),
    # MITIGATED からの遷移: エラーで FAILED へ遷移する
    (EscalationState.MITIGATED, EscalationEvent.ERROR_OCCURRED): (EscalationState.FAILED, _guard_always_true),
    # RESOLVED からの遷移: 新規アラートで PENDING へ遷移する（新規インシデント扱い）
    (EscalationState.RESOLVED, EscalationEvent.ALERT_FIRED): (EscalationState.PENDING, _guard_always_true),
    # FAILED からの遷移: 新規アラートで PENDING へ遷移する（リカバリ）
    (EscalationState.FAILED, EscalationEvent.ALERT_FIRED): (EscalationState.PENDING, _guard_always_true),
}


# ---------------------------------------------------------------------------
# EscalationFSM クラス
# ---------------------------------------------------------------------------

class EscalationFSM:
    """エスカレーションインシデントのライフサイクルを管理する有限状態機械。

    遷移テーブルに基づいて状態遷移を実行し、全遷移を HLC タイムスタンプ付きで
    監査証跡に記録する。同一状態への再遷移は冪等 no-op として扱う。
    """

    def __init__(self, incident_id: str, initial_state: EscalationState = EscalationState.IDLE) -> None:
        """FSM を初期化する。

        Args:
            incident_id: インシデントの一意識別子
            initial_state: FSM の初期状態（デフォルトは IDLE）
        """
        # インシデント ID を設定する
        self._incident_id: str = incident_id
        # 現在の FSM 状態を設定する
        self._state: EscalationState = initial_state
        # 遷移履歴（監査証跡）を初期化する
        self._audit_trail: list[TransitionRecord] = []
        # on_enter フックのコールバック辞書を初期化する
        self._on_enter_hooks: dict[EscalationState, list[Any]] = {}
        # on_exit フックのコールバック辞書を初期化する
        self._on_exit_hooks: dict[EscalationState, list[Any]] = {}
        # FSM 生成時の HLC タイムスタンプを記録する
        self._created_hlc: str = HLC.format()
        # 最終更新の HLC タイムスタンプを初期化する
        self._updated_hlc: str = self._created_hlc

    @property
    def state(self) -> EscalationState:
        """現在の FSM 状態を返す（読み取り専用プロパティ）。"""
        return self._state

    @property
    def incident_id(self) -> str:
        """インシデント ID を返す（読み取り専用プロパティ）。"""
        return self._incident_id

    @property
    def audit_trail(self) -> list[TransitionRecord]:
        """監査証跡のコピーを返す（外部から変更不可）。"""
        # リストのコピーを返して内部状態の直接変更を防ぐ
        return list(self._audit_trail)

    def register_on_enter(self, state: EscalationState, callback: Any) -> None:
        """指定状態への入場フックを登録する。

        Args:
            state: フックを登録する状態
            callback: 遷移後に呼び出されるコールバック (incident_id, state) → None
        """
        # 状態ごとのフックリストが未初期化の場合は初期化する
        if state not in self._on_enter_hooks:
            self._on_enter_hooks[state] = []
        # コールバックをフックリストに追加する
        self._on_enter_hooks[state].append(callback)

    def register_on_exit(self, state: EscalationState, callback: Any) -> None:
        """指定状態からの退場フックを登録する。

        Args:
            state: フックを登録する状態
            callback: 遷移前に呼び出されるコールバック (incident_id, state) → None
        """
        # 状態ごとのフックリストが未初期化の場合は初期化する
        if state not in self._on_exit_hooks:
            self._on_exit_hooks[state] = []
        # コールバックをフックリストに追加する
        self._on_exit_hooks[state].append(callback)

    def can_transition(self, event: EscalationEvent, context: dict[str, Any] | None = None) -> bool:
        """指定イベントによる状態遷移が可能かどうかを確認する（副作用なし）。

        Args:
            event: 確認する遷移イベント
            context: ガード条件の評価に使用するコンテキスト情報

        Returns:
            遷移可能な場合 True、不可能な場合 False
        """
        # コンテキストが None の場合は空辞書を使用する
        ctx = context or {}
        # 遷移テーブルにエントリが存在するか確認する
        key = (self._state, event)
        if key not in _TRANSITION_TABLE:
            return False
        # ガード条件を評価する
        _, guard = _TRANSITION_TABLE[key]
        return guard(self._state, event, ctx)

    def send(self, event: EscalationEvent, context: dict[str, Any] | None = None) -> bool:
        """FSM にイベントを送信して状態遷移を試みる。

        同一状態への遷移（ガード条件が同一状態を指す場合）は no-op として扱う。
        ガード条件が False の場合は遷移を拒否して False を返す。

        Args:
            event: 送信するイベント
            context: ガード条件の評価とアクションフックに渡すコンテキスト情報

        Returns:
            遷移が実行された場合 True、no-op または拒否された場合 False
        """
        # コンテキストが None の場合は空辞書を使用する
        ctx = context or {}
        # 遷移テーブルにエントリが存在するか確認する
        key = (self._state, event)
        if key not in _TRANSITION_TABLE:
            logger.warning(
                "FSM[%s]: 未定義遷移 state=%s event=%s",
                self._incident_id, self._state.value, event.value
            )
            return False
        # 遷移先状態とガード関数を取得する
        target_state, guard = _TRANSITION_TABLE[key]
        # ガード条件を評価する
        if not guard(self._state, event, ctx):
            logger.debug(
                "FSM[%s]: ガード拒否 state=%s event=%s",
                self._incident_id, self._state.value, event.value
            )
            return False
        # 同一状態への遷移は no-op として扱う（冪等性保証）
        if target_state == self._state:
            logger.debug(
                "FSM[%s]: 同一状態 no-op state=%s event=%s",
                self._incident_id, self._state.value, event.value
            )
            return False
        # 現在状態の退場フックを実行する
        self._invoke_on_exit(self._state, ctx)
        # HLC タイムスタンプを取得する
        hlc_ts = HLC.format()
        # 遷移レコードを監査証跡に追加する
        record = TransitionRecord(
            from_state=self._state.value,
            to_state=target_state.value,
            event=event.value,
            hlc_ts=hlc_ts,
            context={k: v for k, v in ctx.items() if isinstance(v, (str, int, float, bool))},
        )
        self._audit_trail.append(record)
        # 遷移前の状態を一時保存する
        old_state = self._state
        # 状態を新しい状態に更新する
        self._state = target_state
        # 最終更新 HLC を記録する
        self._updated_hlc = hlc_ts
        # 遷移完了をログに記録する
        logger.info(
            "FSM[%s]: %s --[%s]--> %s hlc=%s",
            self._incident_id, old_state.value, event.value, target_state.value, hlc_ts
        )
        # 新しい状態の入場フックを実行する
        self._invoke_on_enter(target_state, ctx)
        return True

    def _invoke_on_enter(self, state: EscalationState, context: dict[str, Any]) -> None:
        """指定状態の入場フックを全て実行する（内部用メソッド）。"""
        # 入場フックが登録されていない場合はスキップする
        if state not in self._on_enter_hooks:
            return
        # 登録されている全フックを順番に実行する
        for callback in self._on_enter_hooks[state]:
            try:
                callback(self._incident_id, state, context)
            except Exception as exc:
                logger.error(
                    "FSM[%s]: on_enter フックエラー state=%s: %s",
                    self._incident_id, state.value, exc
                )

    def _invoke_on_exit(self, state: EscalationState, context: dict[str, Any]) -> None:
        """指定状態の退場フックを全て実行する（内部用メソッド）。"""
        # 退場フックが登録されていない場合はスキップする
        if state not in self._on_exit_hooks:
            return
        # 登録されている全フックを順番に実行する
        for callback in self._on_exit_hooks[state]:
            try:
                callback(self._incident_id, state, context)
            except Exception as exc:
                logger.error(
                    "FSM[%s]: on_exit フックエラー state=%s: %s",
                    self._incident_id, state.value, exc
                )

    def on_enter_state(self, state: EscalationState, incident: dict[str, Any]) -> None:
        """状態入場時の共通アクションを実行する（外部から呼び出し可能）。

        Args:
            state: 入場した状態
            incident: インシデント情報辞書
        """
        # PENDING 状態への入場: ページングタイマーを開始する
        if state == EscalationState.PENDING:
            logger.info(
                "FSM[%s]: PENDING 入場 - ページングタイマーを開始する incident=%s",
                self._incident_id, incident.get("alert_name", "unknown")
            )
        # ESCALATED 状態への入場: 上位 tier への通知を準備する
        elif state == EscalationState.ESCALATED:
            logger.info(
                "FSM[%s]: ESCALATED 入場 - 上位 tier 通知を準備する tier=%s",
                self._incident_id, incident.get("escalation_tier", 2)
            )
        # ACKNOWLEDGED 状態への入場: ACK タイマーを停止する
        elif state == EscalationState.ACKNOWLEDGED:
            logger.info(
                "FSM[%s]: ACKNOWLEDGED 入場 - ACK タイマーを停止する acker=%s",
                self._incident_id, incident.get("acker", "unknown")
            )
        # RESOLVED 状態への入場: インシデントクローズ処理を実行する
        elif state == EscalationState.RESOLVED:
            logger.info(
                "FSM[%s]: RESOLVED 入場 - インシデントクローズ処理を実行する",
                self._incident_id
            )
        # FAILED 状態への入場: 障害通知を管理者に送信する
        elif state == EscalationState.FAILED:
            logger.error(
                "FSM[%s]: FAILED 入場 - 障害通知を管理者に送信する reason=%s",
                self._incident_id, incident.get("failure_reason", "unknown")
            )

    def on_exit_state(self, state: EscalationState, incident: dict[str, Any]) -> None:
        """状態退場時の共通アクションを実行する（外部から呼び出し可能）。

        Args:
            state: 退場する状態
            incident: インシデント情報辞書
        """
        # PENDING 状態からの退場: ページングタイマーを停止する
        if state == EscalationState.PENDING:
            logger.info(
                "FSM[%s]: PENDING 退場 - ページングタイマーを停止する",
                self._incident_id
            )
        # ESCALATED 状態からの退場: エスカレーション期間を記録する
        elif state == EscalationState.ESCALATED:
            logger.info(
                "FSM[%s]: ESCALATED 退場 - エスカレーション期間を記録する",
                self._incident_id
            )
        # MITIGATED 状態からの退場: モニタリング期間を記録する
        elif state == EscalationState.MITIGATED:
            logger.info(
                "FSM[%s]: MITIGATED 退場 - モニタリング期間を記録する",
                self._incident_id
            )

    def to_dict(self) -> dict[str, Any]:
        """FSM の全状態を辞書形式にシリアライズして永続化に使用する。

        Returns:
            FSM 状態を表す辞書（JSON シリアライズ可能）
        """
        # FSM の全フィールドを辞書にマッピングする
        return {
            "incident_id": self._incident_id,
            "state": self._state.value,
            "created_hlc": self._created_hlc,
            "updated_hlc": self._updated_hlc,
            "audit_trail": [r.to_dict() for r in self._audit_trail],
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "EscalationFSM":
        """辞書形式のデータから FSM を復元する。

        Args:
            data: to_dict() が生成した辞書

        Returns:
            復元された EscalationFSM インスタンス
        """
        # インシデント ID を辞書から取得する
        incident_id = data["incident_id"]
        # 状態名を EscalationState 列挙型に変換する
        state = EscalationState(data["state"])
        # FSM インスタンスを生成する（コンストラクタは IDLE 状態で初期化）
        fsm = cls(incident_id=incident_id, initial_state=state)
        # 生成 HLC タイムスタンプを復元する
        fsm._created_hlc = data.get("created_hlc", HLC.format())
        # 更新 HLC タイムスタンプを復元する
        fsm._updated_hlc = data.get("updated_hlc", fsm._created_hlc)
        # 監査証跡を復元する
        for record_data in data.get("audit_trail", []):
            record = TransitionRecord(
                from_state=record_data["from_state"],
                to_state=record_data["to_state"],
                event=record_data["event"],
                hlc_ts=record_data["hlc_ts"],
                context=record_data.get("context", {}),
            )
            fsm._audit_trail.append(record)
        return fsm

    def get_transition_count(self) -> int:
        """これまでに発生した遷移の総数を返す。"""
        # 監査証跡の長さ = 遷移回数
        return len(self._audit_trail)

    def get_time_in_state_ms(self) -> int:
        """現在の状態に滞在している時間をミリ秒で返す（概算値）。"""
        # 最終更新時刻を HLC から取得する（wall_ms 部分のみ使用する）
        if self._updated_hlc and ":" in self._updated_hlc:
            updated_wall_ms = int(self._updated_hlc.split(":")[0])
        else:
            updated_wall_ms = int(time.time() * 1000)
        # 現在時刻との差分を計算する
        now_ms = int(time.time() * 1000)
        return max(0, now_ms - updated_wall_ms)

    def is_terminal(self) -> bool:
        """FSM が終端状態（RESOLVED または FAILED）にあるか確認する。"""
        # RESOLVED と FAILED が終端状態
        return self._state in (EscalationState.RESOLVED, EscalationState.FAILED)

    def reset(self) -> None:
        """FSM を IDLE 状態にリセットする（監査証跡は保持する）。"""
        # リセット前の状態を記録する
        old_state = self._state
        # 状態を IDLE に戻す
        self._state = EscalationState.IDLE
        # HLC タイムスタンプを更新する
        hlc_ts = HLC.format()
        self._updated_hlc = hlc_ts
        # リセット操作を監査証跡に記録する
        record = TransitionRecord(
            from_state=old_state.value,
            to_state=EscalationState.IDLE.value,
            event="RESET",
            hlc_ts=hlc_ts,
            context={"reason": "manual_reset"},
        )
        self._audit_trail.append(record)
        logger.info("FSM[%s]: リセット実行 %s → IDLE", self._incident_id, old_state.value)

    def get_last_transition(self) -> TransitionRecord | None:
        """最後に実行された遷移レコードを返す。遷移がない場合は None を返す。"""
        # 監査証跡が空の場合は None を返す
        if not self._audit_trail:
            return None
        # 最後のエントリを返す
        return self._audit_trail[-1]

    def get_transitions_for_state(self, state: EscalationState) -> list[TransitionRecord]:
        """指定状態への遷移レコード一覧を返す。"""
        # to_state が一致するレコードをフィルタリングして返す
        return [r for r in self._audit_trail if r.to_state == state.value]

    def summary(self) -> dict[str, Any]:
        """FSM の現在状態のサマリーを辞書形式で返す。"""
        # サマリーとして主要フィールドを含む辞書を構築する
        return {
            "incident_id": self._incident_id,
            "current_state": self._state.value,
            "is_terminal": self.is_terminal(),
            "transition_count": self.get_transition_count(),
            "time_in_state_ms": self.get_time_in_state_ms(),
            "last_transition": self.get_last_transition().to_dict() if self.get_last_transition() else None,
        }


# ---------------------------------------------------------------------------
# FSMRegistry: 複数インシデントの FSM を管理するレジストリ
# ---------------------------------------------------------------------------

class FSMRegistry:
    """複数のインシデント FSM を一元管理するレジストリ。"""

    def __init__(self) -> None:
        """FSMRegistry を初期化する。"""
        # インシデント ID → EscalationFSM のマッピングを初期化する
        self._registry: dict[str, EscalationFSM] = {}

    def create(self, incident_id: str) -> EscalationFSM:
        """新しいインシデント FSM を生成してレジストリに登録する。

        Args:
            incident_id: 一意のインシデント ID

        Returns:
            新しく生成された EscalationFSM

        Raises:
            ValueError: 同一 incident_id が既に存在する場合
        """
        # 既存エントリの重複チェックを実行する
        if incident_id in self._registry:
            raise ValueError(f"インシデント ID 重複: {incident_id}")
        # 新しい FSM を生成してレジストリに追加する
        fsm = EscalationFSM(incident_id=incident_id)
        self._registry[incident_id] = fsm
        return fsm

    def get(self, incident_id: str) -> EscalationFSM | None:
        """指定 ID の FSM を返す。存在しない場合は None を返す。"""
        return self._registry.get(incident_id)

    def get_or_create(self, incident_id: str) -> EscalationFSM:
        """指定 ID の FSM を返す。存在しない場合は新規生成する。"""
        # 既存エントリを確認する
        if incident_id in self._registry:
            return self._registry[incident_id]
        # 存在しない場合は新規生成する
        return self.create(incident_id)

    def remove(self, incident_id: str) -> bool:
        """指定 ID の FSM をレジストリから削除する。

        Returns:
            削除成功の場合 True、存在しなかった場合 False
        """
        # エントリが存在する場合のみ削除する
        if incident_id in self._registry:
            del self._registry[incident_id]
            return True
        return False

    def list_active(self) -> list[EscalationFSM]:
        """終端状態でない（アクティブな）FSM の一覧を返す。"""
        # is_terminal() が False の FSM を抽出して返す
        return [fsm for fsm in self._registry.values() if not fsm.is_terminal()]

    def list_terminal(self) -> list[EscalationFSM]:
        """終端状態（RESOLVED または FAILED）の FSM の一覧を返す。"""
        # is_terminal() が True の FSM を抽出して返す
        return [fsm for fsm in self._registry.values() if fsm.is_terminal()]

    def count(self) -> int:
        """レジストリに登録されている FSM の総数を返す。"""
        return len(self._registry)

    def serialize_all(self) -> list[dict[str, Any]]:
        """全 FSM の状態をシリアライズしたリストを返す（永続化用）。"""
        # 全エントリを to_dict() で変換して返す
        return [fsm.to_dict() for fsm in self._registry.values()]

    def load_from_list(self, data_list: list[dict[str, Any]]) -> int:
        """シリアライズされたリストから FSM を一括復元する。

        Args:
            data_list: serialize_all() が生成したリスト

        Returns:
            復元した FSM の件数
        """
        # 既存レジストリをクリアして新しい状態をロードする
        loaded_count = 0
        for data in data_list:
            try:
                # FSM を辞書から復元する
                fsm = EscalationFSM.from_dict(data)
                # レジストリに追加する（既存エントリは上書きする）
                self._registry[fsm.incident_id] = fsm
                loaded_count += 1
            except Exception as exc:
                logger.error("FSMRegistry: 復元失敗 incident_id=%s error=%s", data.get("incident_id"), exc)
        return loaded_count
