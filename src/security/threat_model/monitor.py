"""src/security/threat_model/monitor.py

threat_model severity 監視デーモン。
仕様: 15_脅威モデル適合仕様.md §threat_model_monitor
- catalog/cells.yaml を 24h cadence で評価する
- mitigation_status == "open" の cell を検出して alert する
- threat_model.lock.yaml の open_count / cells を更新する
- STRIDE × 4 次元の重大度マトリクスで severity を評価する

環境変数:
  THREAT_DRY_RUN: "true" で実際の書き込みをスキップ（デフォルト true）
  THREAT_MONITOR_INTERVAL_HOURS: 評価間隔（デフォルト 24）
"""

from __future__ import annotations

import argparse
import datetime
import logging
import os
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

try:
    import yaml
    _HAS_YAML = True
except ImportError:
    print("PyYAML required", file=sys.stderr)
    sys.exit(1)

# ロガーを設定する
logger = logging.getLogger(__name__)

# このファイルのあるディレクトリ（src/security/threat_model/）
_THREAT_MODEL_DIR = Path(__file__).parent
# catalog/cells.yaml のパス（脅威モデル cells の SoT）
_CELLS_YAML = _THREAT_MODEL_DIR / "catalog" / "cells.yaml"
# threat_model.lock.yaml の出力先
_LOCK_YAML = _THREAT_MODEL_DIR.parent / "lock" / "threat_model.lock.yaml"
# dry_run デフォルト値を環境変数から取得する
_DRY_RUN_DEFAULT = os.environ.get("THREAT_DRY_RUN", "true").lower() in ("1", "true", "yes")
# 評価間隔（秒）: デフォルト 24 時間
_MONITOR_INTERVAL_SECONDS = int(os.environ.get("THREAT_MONITOR_INTERVAL_HOURS", "24")) * 3600


# ---------------------------------------------------------------------------
# 重大度マトリクス
# ---------------------------------------------------------------------------

# STRIDE カテゴリ別のベース重大度スコア（高いほど危険）
_STRIDE_BASE_SEVERITY: dict[str, int] = {
    "spoofing": 8,
    "tampering": 9,
    "repudiation": 6,
    "information_disclosure": 7,
    "denial_of_service": 7,
    "elevation_of_privilege": 10,
}

# 攻撃主体別の係数（外部攻撃者が最も高い）
_ACTOR_MULTIPLIER: dict[str, float] = {
    "external_actor": 1.0,
    "internal_actor": 0.8,
    "supply_chain": 0.9,
    "automated_system": 0.7,
    "hardware_fault": 0.6,
}

# 攻撃対象別の重み（鍵材料への攻撃が最も高い）
_TARGET_WEIGHT: dict[str, float] = {
    "key_material": 1.2,
    "identity_provider": 1.1,
    "data_store": 1.0,
    "api_endpoint": 0.9,
    "network_transport": 0.8,
}

# severity スコア → ラベル変換
def _score_to_label(score: float) -> str:
    """スコアから severity ラベルを返す。"""
    if score >= 9.0:
        return "critical"
    if score >= 7.0:
        return "high"
    if score >= 5.0:
        return "medium"
    return "low"


# ---------------------------------------------------------------------------
# 脅威セルデータクラス
# ---------------------------------------------------------------------------

@dataclass
class ThreatCell:
    """脅威モデルマトリクスの 1 セル。"""
    # セル識別子（例: threat_001）
    cell_id: str
    # STRIDE 脅威クラス
    threat_class: str
    # 攻撃主体
    threat_dimension_1: str
    # 攻撃対象
    threat_dimension_2: str
    # 対抗手段カテゴリ
    threat_dimension_3: str
    # 対象軸
    threat_dimension_4: str
    # 緩和状態
    mitigation_status: str
    # 緩和理由
    mitigation_reason: str
    # 計算された severity スコア
    severity_score: float = field(default=0.0)
    # severity ラベル
    severity_label: str = field(default="low")

    def __post_init__(self) -> None:
        """severity スコアを STRIDE マトリクスから計算する。"""
        base = _STRIDE_BASE_SEVERITY.get(self.threat_class, 5)
        actor_mult = _ACTOR_MULTIPLIER.get(self.threat_dimension_1, 0.7)
        target_weight = _TARGET_WEIGHT.get(self.threat_dimension_2, 0.8)
        self.severity_score = base * actor_mult * target_weight
        self.severity_label = _score_to_label(self.severity_score)


# ---------------------------------------------------------------------------
# 脅威セルローダー
# ---------------------------------------------------------------------------

def _load_cells(cells_yaml: Path) -> list[ThreatCell]:
    """catalog/cells.yaml から全脅威セルをロードする。"""
    # cells.yaml が存在しない場合は空リストを返す
    if not cells_yaml.exists():
        logger.warning("cells.yaml not found: %s", cells_yaml)
        return []

    # YAML をパースする
    data = yaml.safe_load(cells_yaml.read_text(encoding="utf-8")) or {}
    cells: list[ThreatCell] = []

    # threat_model_cells リストを走査して各セルをロードする
    for entry in data.get("threat_model_cells", []):
        cell_id = str(entry.get("cell_id", ""))
        if not cell_id:
            continue
        cell = ThreatCell(
            cell_id=cell_id,
            threat_class=str(entry.get("threat_class", "")),
            threat_dimension_1=str(entry.get("threat_dimension_1", "")),
            threat_dimension_2=str(entry.get("threat_dimension_2", "")),
            threat_dimension_3=str(entry.get("threat_dimension_3", "")),
            threat_dimension_4=str(entry.get("threat_dimension_4", "")),
            mitigation_status=str(entry.get("mitigation_status", "open")),
            mitigation_reason=str(entry.get("mitigation_reason", "")),
        )
        cells.append(cell)

    return cells


# ---------------------------------------------------------------------------
# 脅威モデル評価エンジン
# ---------------------------------------------------------------------------

class ThreatModelMonitor:
    """threat_model cells を評価して open_count と severity matrix を追跡するクラス。"""

    def __init__(
        self,
        cells_yaml: Path = _CELLS_YAML,
        lock_yaml: Path = _LOCK_YAML,
        dry_run: bool = _DRY_RUN_DEFAULT,
    ) -> None:
        # 脅威セル定義ファイルのパスを保持する
        self._cells_yaml = cells_yaml
        # lock.yaml の出力先を保持する
        self._lock_yaml = lock_yaml
        # dry_run フラグを設定する
        self._dry_run = dry_run

    def evaluate(self) -> dict[str, Any]:
        """全脅威セルを評価して集計結果を返す。"""
        # cells.yaml から全セルをロードする
        cells = _load_cells(self._cells_yaml)
        logger.info("loaded %d threat cells from %s", len(cells), self._cells_yaml)

        # mitigation_status ごとに集計する
        status_counts: dict[str, int] = {}
        for cell in cells:
            status_counts[cell.mitigation_status] = status_counts.get(cell.mitigation_status, 0) + 1

        # open セルを severity 別に分類する
        open_cells = [c for c in cells if c.mitigation_status == "open"]
        open_by_severity: dict[str, list[str]] = {"critical": [], "high": [], "medium": [], "low": []}
        for cell in open_cells:
            open_by_severity[cell.severity_label].append(cell.cell_id)

        # open_count を集計する
        open_count = len(open_cells)
        # critical / high の open 件数を計算する
        critical_open = len(open_by_severity["critical"])
        high_open = len(open_by_severity["high"])

        # 評価結果 dict を構築する
        evaluation = {
            "evaluated_at": datetime.datetime.now(tz=datetime.timezone.utc).strftime(
                "%Y-%m-%dT%H:%M:%SZ"
            ),
            "total_cells": len(cells),
            "status_counts": status_counts,
            "open_count": open_count,
            "open_by_severity": {k: len(v) for k, v in open_by_severity.items()},
            "open_cell_ids_critical": open_by_severity["critical"],
            "open_cell_ids_high": open_by_severity["high"],
        }

        # critical / high の open セルがある場合は警告ログを出力する
        if critical_open > 0:
            logger.error(
                "THREAT MONITOR: %d critical-severity open cells detected! cells: %s",
                critical_open, open_by_severity["critical"][:5],
            )
        if high_open > 0:
            logger.warning(
                "THREAT MONITOR: %d high-severity open cells detected",
                high_open,
            )
        if open_count == 0:
            logger.info("THREAT MONITOR: all cells mitigated (open_count=0)")

        return evaluation

    def update_lock_yaml(self, evaluation: dict[str, Any]) -> None:
        """threat_model.lock.yaml を最新の評価結果で更新する。"""
        if self._dry_run:
            logger.info("[DRY-RUN] would update %s", self._lock_yaml)
            return

        # 既存の lock.yaml を読み込む
        existing: dict[str, Any] = {}
        if self._lock_yaml.exists():
            existing = yaml.safe_load(self._lock_yaml.read_text(encoding="utf-8")) or {}

        # open_count と evaluated_at を更新する
        existing["open_count"] = evaluation["open_count"]
        existing["last_evaluated_at"] = evaluation["evaluated_at"]
        existing["open_by_severity"] = evaluation["open_by_severity"]

        # lock.yaml に書き込む
        self._lock_yaml.parent.mkdir(parents=True, exist_ok=True)
        self._lock_yaml.write_text(
            yaml.dump(existing, allow_unicode=True, default_flow_style=False, sort_keys=False),
            encoding="utf-8",
        )
        logger.info("updated %s: open_count=%d", self._lock_yaml, evaluation["open_count"])

    def run_once(self) -> dict[str, Any]:
        """1 回の評価サイクルを実行して結果を返す。"""
        evaluation = self.evaluate()
        self.update_lock_yaml(evaluation)
        return evaluation

    def run_daemon(self, interval_seconds: int = _MONITOR_INTERVAL_SECONDS) -> None:
        """interval_seconds ごとに評価を繰り返すデーモンモードで実行する。"""
        logger.info(
            "threat_model monitor started (interval=%dh, dry_run=%s)",
            interval_seconds // 3600, self._dry_run,
        )
        while True:
            try:
                # 評価を実行する
                evaluation = self.run_once()
                logger.info(
                    "evaluation complete: total=%d open=%d",
                    evaluation["total_cells"], evaluation["open_count"],
                )
            except Exception as exc:
                # 例外が発生しても停止せず次の評価サイクルを待つ
                logger.error("evaluation failed: %s", exc)
            # 次の評価サイクルまで待機する
            time.sleep(interval_seconds)


# ---------------------------------------------------------------------------
# エントリポイント
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    """threat_model monitor のメインエントリポイント。"""
    parser = argparse.ArgumentParser(
        description="threat_model cells を評価して open_count を追跡する"
    )
    parser.add_argument(
        "--dry-run", action="store_true", default=_DRY_RUN_DEFAULT,
        help="lock.yaml への書き込みをスキップする",
    )
    parser.add_argument(
        "--daemon", action="store_true", default=False,
        help="デーモンモードで繰り返し評価する",
    )
    parser.add_argument(
        "--interval-hours", type=int, default=24,
        help="デーモンモードの評価間隔（時間）",
    )
    args = parser.parse_args(argv)

    # ロギングを設定する
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
    )

    # モニターを初期化する
    monitor = ThreatModelMonitor(dry_run=args.dry_run)

    if args.daemon:
        # デーモンモードで実行する
        monitor.run_daemon(interval_seconds=args.interval_hours * 3600)
    else:
        # 1 回だけ評価して終了する
        evaluation = monitor.run_once()
        print(f"\n--- threat_model evaluation ---")
        print(f"total_cells: {evaluation['total_cells']}")
        print(f"open_count: {evaluation['open_count']}")
        for severity, count in evaluation["open_by_severity"].items():
            if count > 0:
                print(f"  {severity}: {count}")

        # critical / high の open セルが存在する場合は exit code 1 を返す
        if evaluation["open_by_severity"].get("critical", 0) > 0:
            return 1
        if evaluation["open_by_severity"].get("high", 0) > 0:
            return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
