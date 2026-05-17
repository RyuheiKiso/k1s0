"""tools/lock_yaml_generator/generate_conflict_tree.py

conflict_tree.lock.yaml 生成器。
11_クライアント状態適合仕様.md の 5 event × 4 subtype actions matrix を生成する。
手書き禁止 build artifact。
"""

from __future__ import annotations

import datetime
from pathlib import Path
from typing import Any

from tools.lock_yaml_generator.base_generator import BaseGenerator

# 5 event 定義（適合仕様 v1 conflict event カタログ）
_EVENTS: list[dict[str, Any]] = [
    {
        # server_truth_advance event の定義
        "event_id": "server_truth_advance",
        # trigger
        "trigger": "domain_event_received_or_api_response_received",
        # 条件
        "when": "server_truth_version_increases",
        # actions
        "actions": [
            "rollback_optimistic_if_present",
            "update_server_truth",
            "re_evaluate_pending_queue",
        ],
        # ステータス
        "status": "green",
    },
    {
        # optimistic_acknowledged event の定義
        "event_id": "optimistic_acknowledged",
        # trigger
        "trigger": "api_response_2xx_for_in_flight_mutation",
        # 条件
        "when": "optimistic_local_present",
        # actions
        "actions": [
            "promote_optimistic_to_server_truth",
            "delete_pending_queue_entry_by_idempotency_key",
        ],
        # ステータス
        "status": "green",
    },
    {
        # optimistic_rejected event の定義
        "event_id": "optimistic_rejected",
        # trigger
        "trigger": "api_response_4xx_business_error",
        # 条件
        "when": "optimistic_local_present",
        # actions（BusinessConflict の場合は dispatch_conflict_subtype も実行）
        "actions": [
            "rollback_optimistic",
            "present_business_error",
            "dispatch_conflict_subtype_if_business_conflict",
        ],
        # ステータス
        "status": "green",
    },
    {
        # pending_queue_resume event の定義
        "event_id": "pending_queue_resume",
        # trigger
        "trigger": "network_recovery_or_app_resume",
        # 条件
        "when": "pending_queue_non_empty",
        # actions
        "actions": [
            "send_queue_in_enqueue_order",
            "per_aggregate_serialize",
            "fire_optimistic_acknowledged_on_ack",
            "fire_optimistic_rejected_on_4xx",
        ],
        # ステータス
        "status": "green",
    },
    {
        # business_conflict_received event の定義
        "event_id": "business_conflict_received",
        # trigger
        "trigger": "api_response_409_with_subtype",
        # 条件
        "when": "pending_queue_or_optimistic_local_present",
        # actions（subtype 分岐、後述の subtypes テーブルで詳細化）
        "actions": [
            "dispatch_to_subtype_branch",
        ],
        # ステータス
        "status": "green",
    },
]

# 4 BusinessConflict subtype 定義（適合仕様 v1 subtype）
_SUBTYPES: list[dict[str, Any]] = [
    {
        # stale_write subtype の定義
        "subtype_id": "stale_write",
        # 判定条件
        "condition": "base_version_lt_st_version_and_field_diff_disjoint",
        # actions（rebase_clean の場合）
        "actions_rebase_clean": [
            "refetch_server_truth",
            "rebase_pending_op_on_new_version_field_level",
            "auto_resend_with_new_idempotency_key_chained_to_original",
        ],
        # actions（rebase_dirty の場合）
        "actions_rebase_dirty": [
            "refetch_server_truth",
            "rebase_pending_op_on_new_version_field_level",
            "present_3way_merge_ui",
            "hold_queue_until_user_decides",
        ],
        # ステータス
        "status": "green",
    },
    {
        # lost_update subtype の定義
        "subtype_id": "lost_update",
        # 判定条件
        "condition": "base_version_lt_st_version_and_field_diff_intersect",
        # actions
        "actions": [
            "refetch_server_truth",
            "present_3way_merge_ui",
            "hold_queue_until_user_decides",
        ],
        # ステータス
        "status": "green",
    },
    {
        # supersede subtype の定義
        "subtype_id": "supersede",
        # 判定条件
        "condition": "same_actor_subsequent_op_already_updated_same_aggregate",
        # actions
        "actions": [
            "delete_queue_entry",
            "notify_user_silent_toast",
        ],
        # ステータス
        "status": "green",
    },
    {
        # concurrent_edit subtype の定義
        "subtype_id": "concurrent_edit",
        # 判定条件
        "condition": "other_actor_presence_editing_and_self_send_attempt",
        # actions
        "actions": [
            "update_presence_indicator",
            "allow_user_to_continue_or_abort",
        ],
        # ステータス
        "status": "green",
    },
]


class ConflictTreeGenerator(BaseGenerator):
    """conflict_tree.lock.yaml 生成器。

    適合仕様 11 の 5 event × 4 subtype actions matrix を生成する。
    """

    # 出力ファイル名
    OUTPUT_NAME = "conflict_tree.lock.yaml"
    # 必須入力なし（内部定数から生成）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/tier3/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """入力は不要（内部定数から生成するため空 dict を返す）。"""
        return {}

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """5 event × 4 subtype の actions matrix を生成して artifact dict を返す。"""
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        return {
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_conflict_tree.py"
            ),
            # 適合仕様 SoT リファレンス
            "spec_ref": "docs/04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md",
            # 生成日時
            "generated_at": generated_at,
            # 5 event のカタログ
            "events": _EVENTS,
            # イベント数
            "total_events": len(_EVENTS),
            # 4 subtype のカタログ
            "subtypes": _SUBTYPES,
            # subtype 数
            "total_subtypes": len(_SUBTYPES),
        }
