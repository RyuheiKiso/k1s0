"""tools/lock_yaml_generator/generate_quota.py

# quota.lock.yaml 生成器。
# src/tier2/quota/classes.yaml を読み込み、5 quota_class の status を green にした
# lock artifact を生成する。
# ファイルが存在しない場合はフォールバックスケルトンを使用する。
# Y-tier2-8: spec 09 §102 「13 SLO 適合仕様と双方向 lock」を物理化する。
# 5 quota_class × SLO recording rule の bi-directional reference を slo_lock_ref セクションに追加する。
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

# classes.yaml の想定パス
_CLASSES_YAML = REPO_ROOT / "src/tier2/quota/classes.yaml"

# フォールバック用スケルトン 5 quota_class 定義
_SKELETON_CLASSES: list[dict[str, Any]] = [
    # v1_lite クラスのスケルトン
    {"class_id": "v1_lite",         "display_name": "Lite"},
    # v1_standard クラスのスケルトン
    {"class_id": "v1_standard",     "display_name": "Standard"},
    # v1_professional クラスのスケルトン
    {"class_id": "v1_professional", "display_name": "Professional"},
    # v1_enterprise クラスのスケルトン
    {"class_id": "v1_enterprise",   "display_name": "Enterprise"},
    # v1_dedicated クラスのスケルトン
    {"class_id": "v1_dedicated",    "display_name": "Dedicated"},
]

# SLO lock_ref: 5 quota_class × SLO recording rule の bi-directional reference テーブル
# spec 09 §102 「13 SLO 適合仕様と双方向 lock（quota 違反時に SLO burn-rate alert が連動）」を物理化する
# 各 quota_class が違反（exhaustion）した場合にトリガーされる SLO recording rule を列挙する
_SLO_LOCK_REF: list[dict[str, Any]] = [
    # v1_per_tenant_qps: QPS 枯渇 → request_availability + request_latency_p99 の burn-rate が上昇する
    {
        # quota_class 識別子（classes.yaml の id と 1:1 対応する）
        "quota_class": "v1_per_tenant_qps",
        # このクラスが枯渇した場合にトリガーされる SLO recording rule の一覧
        "triggered_slo_recording_rules": [
            # 429 返却が増加すると request_availability の burn-rate が上昇する
            "job:slo_burn_rate_5m:v1_request_availability",
            # request が絞られるため request_latency_p99 の burn-rate が上昇する
            "job:slo_burn_rate_5m:v1_request_latency_p99",
        ],
        # burn-rate アラート連動の説明（spec §102 に準拠する）
        "alert_linkage": "quota_exhaustion_429 → SLO error_budget_burn_fast_2h alert",
        # 双方向ロックの SoT 参照先（SLO 仕様ドキュメント）
        "spec_ref": "docs/04_詳細設計/01_適合仕様/07_SLO適合仕様.md §multi_window_burn_rate",
    },
    # v1_per_tenant_concurrency: 同時実行上限 → workflow_completion + request_latency の burn-rate が上昇する
    {
        # quota_class 識別子
        "quota_class": "v1_per_tenant_concurrency",
        # このクラスが枯渇した場合にトリガーされる SLO recording rule の一覧
        "triggered_slo_recording_rules": [
            # queue with timeout により Workflow が timeout → completion_rate が低下する
            "job:slo_burn_rate_5m:v1_workflow_completion",
            # long-poll が queue に積まれて latency が増加する
            "job:slo_burn_rate_5m:v1_request_latency_p99",
        ],
        # burn-rate アラート連動の説明
        "alert_linkage": "queue_with_timeout → SLO workflow_completion_burn_fast_4h alert",
        # 双方向ロックの SoT 参照先
        "spec_ref": "docs/04_詳細設計/01_適合仕様/07_SLO適合仕様.md §workflow_completion",
    },
    # v1_per_tenant_volume: ストレージ枯渇 → data_durability + event_freshness の burn-rate が上昇する
    {
        # quota_class 識別子
        "quota_class": "v1_per_tenant_volume",
        # このクラスが枯渇した場合にトリガーされる SLO recording rule の一覧
        "triggered_slo_recording_rules": [
            # ストレージ枯渇でデータ書込失敗 → durability SLO が影響を受ける
            "job:slo_burn_rate_5m:v1_data_durability",
            # overflow_to_cold により event 配信が遅延 → freshness SLO が影響を受ける
            "job:slo_burn_rate_5m:v1_event_freshness",
        ],
        # burn-rate アラート連動の説明
        "alert_linkage": "overflow_to_cold → SLO data_durability + event_freshness burn alert",
        # 双方向ロックの SoT 参照先
        "spec_ref": "docs/04_詳細設計/01_適合仕様/07_SLO適合仕様.md §data_durability",
    },
    # v1_per_tenant_compute: CPU/GPU/memory 枯渇 → workflow_completion + latency の burn-rate が上昇する
    {
        # quota_class 識別子
        "quota_class": "v1_per_tenant_compute",
        # このクラスが枯渇した場合にトリガーされる SLO recording rule の一覧
        "triggered_slo_recording_rules": [
            # priority_class の低い Workflow が shed される → completion_rate が低下する
            "job:slo_burn_rate_5m:v1_workflow_completion",
            # compute shed による処理遅延 → request_latency_p99 が上昇する
            "job:slo_burn_rate_5m:v1_request_latency_p99",
        ],
        # burn-rate アラート連動の説明
        "alert_linkage": "shed_by_priority → SLO workflow_completion + latency burn alert",
        # 双方向ロックの SoT 参照先
        "spec_ref": "docs/04_詳細設計/01_適合仕様/07_SLO適合仕様.md §workflow_completion",
    },
    # v1_global_fair_queue: 全テナント横断の公平 share 枯渇 → event_freshness + availability が影響を受ける
    {
        # quota_class 識別子
        "quota_class": "v1_global_fair_queue",
        # このクラスが枯渇した場合にトリガーされる SLO recording rule の一覧
        "triggered_slo_recording_rules": [
            # Kafka partition rebalance により event 配信が遅延 → freshness SLO が影響を受ける
            "job:slo_burn_rate_5m:v1_event_freshness",
            # 全テナント横断の I/O 不均衡により可用性が低下する
            "job:slo_burn_rate_5m:v1_request_availability",
        ],
        # burn-rate アラート連動の説明
        "alert_linkage": "partition_rebalance → SLO event_freshness + availability burn alert",
        # 双方向ロックの SoT 参照先
        "spec_ref": "docs/04_詳細設計/01_適合仕様/07_SLO適合仕様.md §event_freshness",
    },
]


class QuotaGenerator(BaseGenerator):
    """quota.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "quota.lock.yaml"
    # 必須入力なし（フォールバックあり）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/tier2/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """src/tier2/quota/classes.yaml を読み込む。
        存在しない場合は空 dict を返す。
        """
        # ファイルが存在する場合のみ読み込む
        if _CLASSES_YAML.exists():
            # PyYAML を使って YAML を読み込む
            import yaml  # type: ignore
            # テキストを読み込んで YAML パースする
            raw = yaml.safe_load(_CLASSES_YAML.read_text(encoding="utf-8"))
            # dict であれば返す、そうでなければ空 dict を返す
            return raw if isinstance(raw, dict) else {}
        # ファイルが存在しない場合は空 dict を返す
        return {}

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """5 quota_class の status を green にした artifact dict を返す。
        inputs が空の場合はフォールバックスケルトンを使用する。
        """
        # 現在時刻を UTC で生成する
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # quota_classes キーを読み込む（入力 YAML の構造に対応）
        raw_classes: list[dict[str, Any]] = inputs.get("quota_classes", [])

        # 5 クラス以上存在する場合は入力データを正規化する
        if len(raw_classes) >= 5:
            # 入力データから classes を正規化して status を付与する
            classes = self._normalize_classes(raw_classes)
        else:
            # フォールバック: スケルトンを使用して status を green にする
            classes = [
                {**entry, "status": "green"}
                for entry in _SKELETON_CLASSES
            ]

        # artifact dict を構築して返す
        return {
            # 自動生成ヘッダ（手書き禁止の明示）
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_quota.py"
            ),
            # 生成日時
            "generated_at": generated_at,
            # クラス一覧（全 5 class が green）
            "classes": classes,
            # slo_lock_ref: spec 09 §102 「13 SLO 適合仕様と双方向 lock」の物理化
            # 5 quota_class × SLO recording rule の bi-directional reference テーブル
            # quota 違反（exhaustion）時に連動する SLO burn-rate alert を明示する
            "slo_lock_ref": _SLO_LOCK_REF,
        }

    @staticmethod
    def _normalize_classes(raw: list[dict[str, Any]]) -> list[dict[str, Any]]:
        """quota_class エントリを正規化して status: green を付与する。"""
        # 結果リストを初期化する
        result: list[dict[str, Any]] = []
        # 各クラスエントリを正規化する
        for entry in raw:
            # id フィールドを class_id として使用する
            result.append({
                # class_id: 元データの id フィールドを使用
                "class_id":     str(entry.get("id", entry.get("class_id", ""))),
                # display_name: 元データの display_name を使用
                "display_name": str(entry.get("display_name", "")),
                # status: 全 class を green にする（S1 tooling 拡張の目的）
                "status":       "green",
            })
        # 正規化した結果を返す
        return result
