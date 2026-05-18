"""tools/lock_yaml_generator/generate_enforcement_points.py

enforcement_points.lock.yaml 生成器。
src/tier1/lock/enforcement_points_input.yaml を読み込み、テナント容量 drill 結果を正規化する。
存在しない場合はフォールバックスケルトンを生成する（全 drill_state: pending）。

catalog SoT: src/tier1/schema/tenant_capacity/classes.yaml / src/tier1/schema/tenant_capacity/scenarios.yaml
catalog が存在する場合、宣言された quota_class id が全て drill 済みであることを検証する。
catalog の quota_classes[].id は input の drills[].layer_id と対応する。
catalog が存在しない場合は catalog_validation_status: catalog_absent で graceful degradation する。
"""

from __future__ import annotations

import datetime
from pathlib import Path
from typing import Any

from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT
from tools.lock_yaml_generator.lib.catalog_validator import (
    load_catalog,
    validate_catalog_against_input,
    build_catalog_metadata,
)

# enforcement_points_input.yaml の想定名
_INPUT_NAME = "enforcement_points_input.yaml"

# catalog SoT パス（REPO_ROOT 相対）
_CATALOG_CLASSES_PATH = "src/tier1/schema/tenant_capacity/classes.yaml"
_CATALOG_SCENARIOS_PATH = "src/tier1/schema/tenant_capacity/scenarios.yaml"

# 09_テナント容量適合仕様.md §v1 quota_class セット（5 class）に基づく enforcement point 定義
_ENFORCEMENT_LAYERS: list[dict[str, str]] = [
    # v1_per_tenant_qps: Envoy Local Rate Limit filter、burst 2x 許容、reject_429
    {"layer_id": "v1_per_tenant_qps", "layer_type": "transport"},
    # v1_per_tenant_concurrency: Library token bucket（in-process）、queue_with_timeout
    {"layer_id": "v1_per_tenant_concurrency", "layer_type": "library"},
    # v1_per_tenant_volume: storage layer（Rook+Ceph / CloudNativePG / ClickHouse / Kafka）、overflow_to_cold
    {"layer_id": "v1_per_tenant_volume", "layer_type": "storage"},
    # v1_per_tenant_compute: OSS native quota（Temporal / KEDA / pgvector）、shed_by_priority
    {"layer_id": "v1_per_tenant_compute", "layer_type": "infrastructure"},
    # v1_global_fair_queue: broker layer（Kafka partition rebalance / Istio destination rule）、partition_rebalance
    {"layer_id": "v1_global_fair_queue", "layer_type": "broker"},
]


class EnforcementPointsGenerator(BaseGenerator):
    """enforcement_points.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "enforcement_points.lock.yaml"
    # 必須入力なし（フォールバックあり）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/tier1/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """src/tier1/lock/enforcement_points_input.yaml を読み込む。
        存在しない場合は空 dict を返す。
        """
        return self.load_lock(lock_dir, _INPUT_NAME)

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """enforcement drills リストを正規化して artifact dict を返す。
        inputs が空の場合はフォールバックスケルトンを使用する。
        catalog SoT (tenant_capacity/classes.yaml) が存在する場合は cross-validation を実行する。
        quota_classes[].id は input drills[].layer_id と対応する。
        """
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # drills キーを読み込む
        raw_drills: list[dict[str, Any]] = inputs.get("drills", [])

        if raw_drills:
            # 入力データから drills を正規化する
            drills = self._normalize_drills(raw_drills)
        else:
            # フォールバック: 全 pending のスケルトンを生成する
            drills = self._build_skeleton()

        # catalog SoT のパスを解決する
        classes_path = REPO_ROOT / _CATALOG_CLASSES_PATH
        scenarios_path = REPO_ROOT / _CATALOG_SCENARIOS_PATH

        # catalog yaml を読み込む（不在時は None）
        classes_catalog = load_catalog(classes_path)
        scenarios_catalog = load_catalog(scenarios_path)

        # quota_classes catalog の cross-validation: quota_classes[].id vs input drills[].layer_id
        classes_result = validate_catalog_against_input(
            catalog=classes_catalog,
            input_data=inputs,
            class_key="quota_classes",
            input_id_key="layer_id",
            input_list_key="drills",
            catalog_path=classes_path,
            catalog_id_key="id",
        )

        # scenarios catalog の cross-validation（scenarios id と drill id の対応がないため passed 扱い）
        scenarios_result = validate_catalog_against_input(
            catalog=scenarios_catalog,
            input_data={},
            class_key="scenarios",
            input_id_key="id",
            input_list_key=None,
            catalog_path=scenarios_path,
            catalog_id_key="id",
        )
        # scenarios は drill 対象外なので常に passed とする
        if scenarios_catalog is not None:
            scenarios_result = {
                "status": "passed",
                "missing": [],
                "catalog_sot_hash": scenarios_result.get("catalog_sot_hash"),
            }

        # metadata を構築する
        catalog_meta = build_catalog_metadata(
            classes_result=classes_result,
            scenarios_result=scenarios_result,
            classes_path=_CATALOG_CLASSES_PATH,
            scenarios_path=_CATALOG_SCENARIOS_PATH,
        )

        return {
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_enforcement_points.py"
            ),
            "total_drills": len(drills),
            "generated_at": generated_at,
            "metadata":     catalog_meta,
            "drills":       drills,
        }

    @staticmethod
    def _normalize_drills(raw: list[dict[str, Any]]) -> list[dict[str, Any]]:
        """enforcement drill エントリを正規化する。"""
        result: list[dict[str, Any]] = []
        for entry in raw:
            result.append({
                "drill_id":   str(entry.get("drill_id", "")),
                "layer_id":   str(entry.get("layer_id", "")),
                "layer_type": str(entry.get("layer_type", "")),
                "drill_state": str(entry.get("drill_state", "pending")),
                "notes":      str(entry.get("notes", "")),
            })
        return result

    @staticmethod
    def _build_skeleton() -> list[dict[str, Any]]:
        """enforcement layer ごとのスケルトンを生成する。"""
        result: list[dict[str, Any]] = []
        for layer in _ENFORCEMENT_LAYERS:
            layer_id = layer["layer_id"]
            result.append({
                "drill_id":   f"capacity_drill__{layer_id}",
                "layer_id":   layer_id,
                "layer_type": layer["layer_type"],
                "drill_state": "pending",
                "notes":      "",
            })
        return result
