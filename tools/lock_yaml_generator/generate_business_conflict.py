"""tools/lock_yaml_generator/generate_business_conflict.py

# business_conflict.lock.yaml 生成器。
# src/tier2/business_conflict/subtypes.yaml を読み込み、
# BusinessConflict 4 サブタイプの宣言状態を lock artifact として生成する。
# tier3 の conflict_tree.lock.yaml とのクロスリファレンス検証を含む。
"""

# 標準ライブラリのインポート
from __future__ import annotations

# datetime モジュール: generated_at タイムスタンプ生成に使用
import datetime
# Path: ファイルパス操作に使用
from pathlib import Path
# Any: 型ヒントに使用
from typing import Any

# BaseGenerator と REPO_ROOT をインポートする
from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT

# subtypes.yaml の想定パス（SoT）
_SUBTYPES_YAML = REPO_ROOT / "src/tier2/business_conflict/subtypes.yaml"

# tier3 conflict_tree.lock.yaml のパス（クロスリファレンス検証用）
_CONFLICT_TREE_LOCK = REPO_ROOT / "src/tier3/lock/conflict_tree.lock.yaml"

# spec_ref 文字列（10_テナント分離適合仕様.md §business_conflict_subtypes）
_SPEC_REF = (
    "docs/04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md"
    " §business_conflict_subtypes"
)

# tier3 クロスリファレンスのパス文字列
_TIER3_CROSS_REF = "src/tier3/lock/conflict_tree.lock.yaml"


class BusinessConflictGenerator(BaseGenerator):
    """business_conflict.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "business_conflict.lock.yaml"
    # 必須入力なし（SoT は別パスから直読み）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ（tier2/lock）
    DEFAULT_OUTPUT_DIR = "src/tier2/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """subtypes.yaml と conflict_tree.lock.yaml を読み込む。
        いずれかが存在しない場合は空 dict を格納して返す。
        """
        # PyYAML をインポートする
        import yaml  # type: ignore

        # subtypes.yaml の読み込み結果を格納する dict
        result: dict[str, Any] = {}

        # subtypes.yaml が存在する場合は読み込む
        if _SUBTYPES_YAML.exists():
            # テキストを読み込んで YAML パースする
            raw_subtypes = yaml.safe_load(
                _SUBTYPES_YAML.read_text(encoding="utf-8")
            )
            # dict であれば subtypes キーに格納する
            result["subtypes_data"] = raw_subtypes if isinstance(raw_subtypes, dict) else {}
        else:
            # ファイルが存在しない場合は空 dict を格納する
            result["subtypes_data"] = {}

        # conflict_tree.lock.yaml が存在する場合は読み込む
        if _CONFLICT_TREE_LOCK.exists():
            # テキストを読み込んで YAML パースする
            raw_conflict = yaml.safe_load(
                _CONFLICT_TREE_LOCK.read_text(encoding="utf-8")
            )
            # dict であれば conflict_tree_data キーに格納する
            result["conflict_tree_data"] = (
                raw_conflict if isinstance(raw_conflict, dict) else {}
            )
        else:
            # ファイルが存在しない場合は空 dict を格納する
            result["conflict_tree_data"] = {}

        # 読み込み結果を返す
        return result

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """BusinessConflict サブタイプの宣言状態を表す artifact dict を返す。
        inputs が空の場合はサブタイプ一覧を空にして生成する。
        tier3 conflict_tree.lock.yaml の events に tier3_event が全件存在するか検証する。
        """
        # 現在時刻を UTC で生成する
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # subtypes_data を取り出す
        subtypes_data = inputs.get("subtypes_data", {})
        # conflict_tree_data を取り出す
        conflict_tree_data = inputs.get("conflict_tree_data", {})

        # subtypes.yaml から subtypes リストを取り出す
        raw_subtypes: list[dict[str, Any]] = subtypes_data.get("subtypes", [])

        # conflict_tree.lock.yaml の events リストから event_id の集合を構築する
        conflict_events: list[dict[str, Any]] = conflict_tree_data.get("events", [])
        # event_id を set に格納してクロスリファレンス検証に使う
        known_event_ids: set[str] = {
            str(ev.get("event_id", ""))
            for ev in conflict_events
            if ev.get("event_id")
        }

        # 各 subtype エントリを整形する
        subtypes: list[dict[str, Any]] = []
        # クロスリファレンスで欠損している event_id を記録するリスト
        missing_events: list[str] = []

        # raw_subtypes の各エントリを処理する
        for raw in raw_subtypes:
            # subtype_id を取り出す
            subtype_id = str(raw.get("subtype_id", ""))
            # detector_rule を取り出す
            detector_rule = str(raw.get("detector_rule", ""))
            # resolution_strategy を取り出す
            resolution_strategy = str(raw.get("resolution_strategy", ""))
            # audit_envelope_field を取り出す
            audit_envelope_field = str(raw.get("audit_envelope_field", ""))
            # tier3_event を取り出す
            tier3_event = str(raw.get("tier3_event", ""))

            # 整形済みエントリをリストに追加する
            subtypes.append(
                {
                    # サブタイプ識別子
                    "subtype_id": subtype_id,
                    # 検出ルール
                    "detector_rule": detector_rule,
                    # 解決戦略
                    "resolution_strategy": resolution_strategy,
                    # audit_envelope フィールド名
                    "audit_envelope_field": audit_envelope_field,
                    # 対応する tier3 event_id
                    "tier3_event": tier3_event,
                }
            )

            # known_event_ids に tier3_event が存在しない場合は missing_events に記録する
            if tier3_event and tier3_event not in known_event_ids:
                # 欠損イベントを記録する（generator は raise しない）
                missing_events.append(tier3_event)

        # cross_reference_check.status を決定する（欠損がなければ green、あれば red）
        cross_ref_status = "green" if not missing_events else "red"

        # artifact dict を構築して返す
        return {
            # 自動生成ヘッダ（手書き禁止の明示）
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_business_conflict.py"
            ),
            # 参照仕様
            "spec_ref": _SPEC_REF,
            # 生成日時
            "generated_at": generated_at,
            # tier3 クロスリファレンスのパス
            "tier3_cross_reference": _TIER3_CROSS_REF,
            # サブタイプ総数
            "total_subtypes": len(subtypes),
            # サブタイプ一覧
            "subtypes": subtypes,
            # クロスリファレンス検証結果
            "cross_reference_check": {
                # 検証ステータス（green または red）
                "status": cross_ref_status,
                # 欠損イベントのリスト（green の場合は空）
                "missing_events": missing_events,
            },
        }
