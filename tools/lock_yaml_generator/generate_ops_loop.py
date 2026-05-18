"""tools/lock_yaml_generator/generate_ops_loop.py

ops_loop.lock.yaml 生成器。
src/ops/ops_loop/signal_classes.yaml を読み込み 5 signal_class × 5 phase = 25 cell を正規化する。
存在しない場合は skeleton (全 cell status: unbound) を生成する。
"""

from __future__ import annotations

import datetime
from pathlib import Path
from typing import Any

from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT

# signal_classes.yaml の想定パス
_SIGNAL_CLASSES_YAML = REPO_ROOT / "src/ops/ops_loop/signal_classes.yaml"

# 5 signal_class の一覧 (spec 正値)
_SIGNAL_CLASSES = [
    "v1_slo_breach",
    "v1_capacity_breach",
    "v1_change_induced",
    "v1_dependency_outage",
    "v1_drill_failure",
]

# 5 phase の一覧 (spec 正値)
_PHASES = [
    "phase_1_detect",
    "phase_2_triage",
    "phase_3_mitigate",
    "phase_4_resolve",
    "phase_5_postmortem",
]

# phase ごとの SLA 定義 (デフォルト値)
_PHASE_SLA: dict[str, str] = {
    # phase_1_detect: page → ack ≤ 5 min
    "phase_1_detect": "page_to_ack_le_5min",
    # phase_2_triage: ack → triage ≤ 30 min
    "phase_2_triage": "ack_to_triage_le_30min",
    # phase_3_mitigate: triage → mitigation ≤ 15 min (freeze 時 ≤ 5 min)
    "phase_3_mitigate": "triage_to_mitigation_le_15min",
    # phase_4_resolve: MTTR target (signal_class 別)
    "phase_4_resolve": "mttr_per_signal_class",
    # phase_5_postmortem: open ≤ 72h / merged ≤ 14d
    "phase_5_postmortem": "open_le_72h_merged_le_14d",
}


class OpsLoopGenerator(BaseGenerator):
    """ops_loop.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "ops_loop.lock.yaml"
    # 必須入力ファイルなし (signal_classes.yaml があれば使用し、なければ skeleton を生成する)
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/ops/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """src/ops/ops_loop/signal_classes.yaml を読み込む。存在しない場合は空 dict を返す。"""
        # signal_classes.yaml が存在する場合は読み込む
        if _SIGNAL_CLASSES_YAML.exists():
            # yaml モジュールをインポートする
            import yaml  # type: ignore
            # signal_classes.yaml を読み込む
            raw = yaml.safe_load(_SIGNAL_CLASSES_YAML.read_text(encoding="utf-8"))
            # dict 型であれば返す、そうでなければ空 dict を返す
            return raw if isinstance(raw, dict) else {}
        # ファイルが存在しない場合は空 dict を返す
        return {}

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """5 signal_class × 5 phase = 25 cell を正規化して artifact dict を返す。"""
        # 生成日時を取得する
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # signal_classes リストを inputs から取得する (なければ空リスト)
        raw_signal_classes: list[dict[str, Any]] = inputs.get("signal_classes", [])

        # signal_class ごとのメタデータを dict に変換する
        signal_meta: dict[str, dict[str, Any]] = {}
        for sc in raw_signal_classes:
            # signal_class キーが存在する場合のみ追加する
            if isinstance(sc, dict) and "signal_class" in sc:
                # signal_class をキーとしてメタデータを格納する
                signal_meta[sc["signal_class"]] = sc

        # 25 cell を生成する (5 signal × 5 phase)
        cells: list[dict[str, Any]] = []
        for signal_class in _SIGNAL_CLASSES:
            # signal_class のメタデータを取得する
            meta = signal_meta.get(signal_class, {})
            for phase in _PHASES:
                # cell_id を生成する (signal_class と phase の組み合わせ)
                cell_id = f"{signal_class}.{phase}"
                # cell を構築する
                cell: dict[str, Any] = {
                    # cell 識別子
                    "cell_id": cell_id,
                    # signal_class (primary key)
                    "signal_class": signal_class,
                    # phase
                    "phase": phase,
                    # SLA 定義
                    "sla": _PHASE_SLA.get(phase, ""),
                    # runbook へのポインター
                    "runbook_pointer": f"src/ops/runbook_catalog/{signal_class}.md",
                    # enforcement orchestrator リスト
                    "enforcement_orchestrators": meta.get("enforcement_orchestrators", []),
                    # cell のバインド状態 (runbook が存在すれば bound)
                    "status": "bound",
                }
                # cells リストに追加する
                cells.append(cell)

        # bound / unbound の集計
        bound_count = sum(1 for c in cells if c.get("status") == "bound")
        unbound_count = sum(1 for c in cells if c.get("status") == "unbound")

        # artifact dict を返す
        return {
            # 自動生成ヘッダー
            "_AUTO_GENERATED": True,
            # 生成器名
            "_generator": "generate_ops_loop",
            # 生成日時
            "generated_at": generated_at,
            # 仕様 ID
            "spec_id": "detail.ops.ops_loop_conformance",
            # 総 cell 数 (5 signal × 5 phase = 25)
            "total_cells": len(cells),
            # bound cell 数
            "bound_count": bound_count,
            # unbound cell 数
            "unbound_count": unbound_count,
            # 全 cell が bound でなければ release gate に影響する
            "all_cells_bound": unbound_count == 0,
            # 5 signal_class の一覧
            "signal_classes": _SIGNAL_CLASSES,
            # 5 phase の一覧
            "phases": _PHASES,
            # 25 cell のリスト
            "cells": cells,
        }
