# k1s0-proof: PROOF-cross_bff-prog-001 -> IMPL-cross_bff-0001
"""tools/lock_yaml_generator/generate_capabilities.py

capabilities.lock.yaml 生成器。
src/tier1/lock/capabilities_input.yaml を読み込む。

01_Bidi適合仕様.md §adapter↔class supports 対応 に基づき、
5 conformance_class × 8 adapter の全 40 cell を構築する。
supports 行列に含まれない cell は status: not_applicable となる（手書き禁止）。

catalog SoT: src/tier1/schema/bidi/classes.yaml / src/tier1/schema/bidi/scenarios.yaml
catalog が存在する場合、宣言された conformance_class id が全て drill 済みであることを検証する。
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
    _extract_catalog_ids,
    compute_sot_hash,
    STATUS_PASSED,
    STATUS_MISSING_DRILLS,
    STATUS_CATALOG_ABSENT,
)

_INPUT_NAME = "capabilities_input.yaml"

# catalog SoT パス（REPO_ROOT 相対）
_CATALOG_CLASSES_PATH = "src/tier1/schema/bidi/classes.yaml"
_CATALOG_SCENARIOS_PATH = "src/tier1/schema/bidi/scenarios.yaml"

# 01_Bidi適合仕様.md §v1 conformance_class セット（5 class）
_CONFORMANCE_CLASSES: list[str] = [
    "v1_interactive",
    "v1_alert",
    "v1_event_feed",
    "v1_live_snapshot",
    "v1_bulk_upload",
]

# 01_Bidi適合仕様.md §capabilities.lock.yaml adapter 定義（8 adapter）
_ADAPTERS: list[str] = [
    "grpc_native",
    "connect_bidi",
    "web_transport",
    "sse_paired",
    "paired_post_sse",
    "long_poll",
    "webhook",
    "messaging_bridge",
]

# 01_Bidi適合仕様.md §adapter↔class supports 対応 — adapter → supports 集合
_SUPPORTS: dict[str, set[str]] = {
    "grpc_native":      {"v1_interactive", "v1_alert", "v1_event_feed", "v1_live_snapshot", "v1_bulk_upload"},
    "connect_bidi":     {"v1_interactive", "v1_alert", "v1_event_feed", "v1_live_snapshot", "v1_bulk_upload"},
    "web_transport":    {"v1_interactive", "v1_alert", "v1_event_feed", "v1_live_snapshot", "v1_bulk_upload"},
    "sse_paired":       {"v1_alert", "v1_event_feed", "v1_live_snapshot"},
    "paired_post_sse":  {"v1_interactive", "v1_alert", "v1_event_feed", "v1_live_snapshot"},
    "long_poll":        {"v1_event_feed"},
    "webhook":          {"v1_alert", "v1_event_feed", "v1_live_snapshot"},
    "messaging_bridge": {"v1_event_feed", "v1_live_snapshot", "v1_bulk_upload"},
}


class CapabilitiesGenerator(BaseGenerator):
    """capabilities.lock.yaml 生成器。"""

    OUTPUT_NAME = "capabilities.lock.yaml"
    REQUIRED_INPUTS: list[str] = []
    SCHEMA_PATH: Path | None = None
    DEFAULT_OUTPUT_DIR = "src/tier1/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """src/tier1/lock/capabilities_input.yaml を読み込む。
        存在しない場合は空 dict を返す。
        """
        return self.load_lock(lock_dir, _INPUT_NAME)

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """conformance_classes × adapters の cell matrix を構築する。

        applicable cell の status は inputs の status_map から読む。
        not_applicable cell は _SUPPORTS 行列から自動決定（input 禁止）。
        catalog SoT (bidi/classes.yaml / bidi/scenarios.yaml) が存在する場合は
        cross-validation を実行して metadata に結果を埋め込む。
        """
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        raw_cells: list[dict[str, Any]] = inputs.get("cells", [])
        status_map: dict[str, str] = {
            str(c.get("cell_id", "")): str(c.get("status", "pending"))
            for c in raw_cells
        }
        # assertion_run_result_id を cell_id → 値のマップに変換する
        # 各 cell に対して scenario assertion runner の実行結果 id を記録する（CI 整合 2 の物理化）
        assertion_run_result_map: dict[str, str] = {
            str(c.get("cell_id", "")): str(c.get("assertion_run_result_id", "pending"))
            for c in raw_cells
        }

        cells = self._build_cells(status_map, assertion_run_result_map)
        applicable = [c for c in cells if c["status"] != "not_applicable"]

        # catalog SoT のパスを解決する
        classes_path = REPO_ROOT / _CATALOG_CLASSES_PATH
        scenarios_path = REPO_ROOT / _CATALOG_SCENARIOS_PATH

        # catalog yaml を読み込む（不在時は None）
        classes_catalog = load_catalog(classes_path)
        scenarios_catalog = load_catalog(scenarios_path)

        # classes catalog と input の cross-validation を実行する
        # bidi/classes.yaml は conformance_classes[] キーに id を持つ
        # input は cells[] の cell_id キーに "{class}__{adapter}" 形式の id を持つ
        # conformance class id のみで突合するため、class id を cell_id prefix から抽出する
        drilled_class_ids: set[str] = set()
        for cell in raw_cells:
            cell_id = str(cell.get("cell_id", ""))
            # cell_id = "{conformance_class}__{adapter}" 形式
            if "__" in cell_id:
                drilled_class_ids.add(cell_id.split("__")[0])

        # catalog が存在する場合は直接突合する
        if classes_catalog is not None:
            declared = _extract_catalog_ids(classes_catalog, "conformance_classes", "id")
            missing = [cid for cid in declared if cid not in drilled_class_ids]
            try:
                sot_hash = compute_sot_hash(classes_path)
            except ValueError:
                sot_hash = None
            classes_result: dict[str, Any] = {
                "status": STATUS_PASSED if not missing else STATUS_MISSING_DRILLS,
                "missing": missing,
                "catalog_sot_hash": sot_hash,
            }
        else:
            # catalog が存在しない場合は graceful degradation
            classes_result = {
                "status": STATUS_CATALOG_ABSENT,
                "missing": [],
                "catalog_sot_hash": None,
            }

        # scenarios catalog は bidi scenarios（test scenario 定義）であり
        # input の cell_id とは直接対応しないため、catalog hash の記録のみ行う
        if scenarios_catalog is not None:
            try:
                scenarios_hash = compute_sot_hash(scenarios_path)
            except ValueError:
                scenarios_hash = None
            scenarios_result: dict[str, Any] = {
                "status": STATUS_PASSED,
                "missing": [],
                "catalog_sot_hash": scenarios_hash,
            }
        else:
            scenarios_result = {
                "status": STATUS_CATALOG_ABSENT,
                "missing": [],
                "catalog_sot_hash": None,
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
                " tools/lock_yaml_generator/generate_capabilities.py"
            ),
            "conformance_classes": len(_CONFORMANCE_CLASSES),
            "adapters":            len(_ADAPTERS),
            "total_cells":         len(cells),
            "applicable_cells":    len(applicable),
            "generated_at":        generated_at,
            "metadata":            catalog_meta,
            "cells":               cells,
        }

    @classmethod
    def _build_cells(
        cls,
        status_map: dict[str, str],
        assertion_run_result_map: dict[str, str] | None = None,
    ) -> list[dict[str, Any]]:
        """spec の supports 行列から全 40 cell を構築する。

        applicable cell は status_map に status があればそれを使用、無ければ pending。
        not_applicable cell は _SUPPORTS から自動決定。
        assertion_run_result_map が渡された場合は assertion_run_result_id を cell に追加する。
        """
        # assertion_run_result_map が None の場合は空辞書で初期化する
        if assertion_run_result_map is None:
            assertion_run_result_map = {}
        result: list[dict[str, Any]] = []
        for cc in _CONFORMANCE_CLASSES:
            for adapter in _ADAPTERS:
                cell_id = f"{cc}__{adapter}"
                if cc in _SUPPORTS.get(adapter, set()):
                    # applicable cell: status_map から status を取得し、無ければ pending とする
                    status = status_map.get(cell_id, "pending")
                    # assertion_run_result_id を assertion_run_result_map から取得する
                    # 値が無い場合は "pending" とする（assertion runner 未実行を示す）
                    assertion_run_result_id = assertion_run_result_map.get(cell_id, "pending")
                    result.append({
                        "cell_id":                 cell_id,
                        "conformance_class":       cc,
                        "adapter":                 adapter,
                        "status":                  status,
                        "assertion_run_result_id": assertion_run_result_id,
                    })
                else:
                    # not_applicable cell: _SUPPORTS から自動決定（assertion_run_result_id は n/a）
                    result.append({
                        "cell_id":                 cell_id,
                        "conformance_class":       cc,
                        "adapter":                 adapter,
                        "status":                  "not_applicable",
                        "assertion_run_result_id": "n/a",
                    })
        return result
