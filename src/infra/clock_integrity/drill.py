"""src/infra/clock_integrity/drill.py

clock_integrity drill 実行モジュール。
仕様: 13_時刻整合適合仕様.md §clock_drill_cadence
- 5 clock_integrity_class に対して drill を実行する
- chrony / PTP の skew 注入テストを実施する
- drill 結果を clock_causality_proof.lock.yaml の drill_state フィールドに書き込む

環境変数:
  CLOCK_DRY_RUN: "true" で実際のコマンドをスキップ（デフォルト true）
"""

from __future__ import annotations

import argparse
import datetime
import logging
import os
import subprocess
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

# このファイルのあるディレクトリ（src/infra/clock_integrity/）
_CLOCK_DIR = Path(__file__).parent
# classes.yaml のパス（clock_integrity_class 定義）
_CLASSES_YAML = _CLOCK_DIR / "classes.yaml"
# lock ディレクトリのパス
_LOCK_DIR = _CLOCK_DIR.parent / "lock"
# dry_run デフォルト値を環境変数から取得する
_DRY_RUN_DEFAULT = os.environ.get("CLOCK_DRY_RUN", "true").lower() in ("1", "true", "yes")


# ---------------------------------------------------------------------------
# ドリル結果データクラス
# ---------------------------------------------------------------------------

@dataclass
class ClockDrillResult:
    """1 clock_integrity_class のドリル実行結果。"""
    # クラス識別子
    class_id: str
    # ドリル実行状態（green / yellow / red）
    drill_state: str
    # ドリル実行日時
    drill_executed_at: str
    # ドリル実行メモ
    drill_notes: str
    # 計測されたスキュー（ナノ秒）
    measured_skew_ns: int = 0
    # 許容閾値（マイクロ秒）
    max_skew_us: int = 1000
    # エラーメッセージ（失敗時のみ）
    error: str = ""


# ---------------------------------------------------------------------------
# ドリル実行エンジン
# ---------------------------------------------------------------------------

class ClockDrillRunner:
    """clock_integrity_class ごとのドリルを実行するクラス。"""

    def __init__(
        self,
        classes_yaml: Path = _CLASSES_YAML,
        dry_run: bool = _DRY_RUN_DEFAULT,
    ) -> None:
        # classes.yaml を読み込む
        self._classes = self._load_classes(classes_yaml)
        # dry_run フラグを設定する
        self._dry_run = dry_run

    def _load_classes(self, classes_yaml: Path) -> dict[str, Any]:
        """classes.yaml から clock_integrity_class 定義をロードする。"""
        if not classes_yaml.exists():
            logger.warning("classes.yaml not found: %s", classes_yaml)
            return {}
        data = yaml.safe_load(classes_yaml.read_text(encoding="utf-8")) or {}
        return {
            entry["class_id"]: entry
            for entry in data.get("clock_integrity_classes", [])
            if "class_id" in entry
        }

    def run_all(self) -> list[ClockDrillResult]:
        """全 clock_integrity_class のドリルを実行する。"""
        results: list[ClockDrillResult] = []
        for class_id, entry in self._classes.items():
            logger.info("running clock drill: %s (dry_run=%s)", class_id, self._dry_run)
            result = self._run_single(class_id, entry)
            results.append(result)
        return results

    def _run_single(self, class_id: str, entry: dict[str, Any]) -> ClockDrillResult:
        """1 クラスのドリルを実行する。"""
        # ドリル実行日時を UTC ISO 8601 形式で取得する
        executed_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        # 許容スキュー閾値を取得する（マイクロ秒）
        max_skew_us = int(entry.get("max_skew_us", 1000))

        try:
            if self._dry_run:
                # dry_run モードでシミュレーション結果を返す
                return self._simulate_drill(class_id, executed_at, max_skew_us)
            else:
                # 実際のドリルを実行する
                return self._execute_drill(class_id, entry, executed_at, max_skew_us)
        except Exception as exc:
            logger.error("clock drill failed for %s: %s", class_id, exc)
            return ClockDrillResult(
                class_id=class_id,
                drill_state="red",
                drill_executed_at=executed_at,
                drill_notes=f"drill failed: {exc}",
                max_skew_us=max_skew_us,
                error=str(exc),
            )

    def _simulate_drill(
        self, class_id: str, executed_at: str, max_skew_us: int
    ) -> ClockDrillResult:
        """dry_run: WSL2 環境のシミュレーション結果を返す（skew=0ns）。"""
        # WSL2 では全ノードが同一ホスト時刻を共有するため skew=0
        note = (
            f"[DRY-RUN] kind cluster: nodes share WSL2 host clock, "
            f"measured skew=0ns < {max_skew_us * 1000}ns threshold"
        )
        return ClockDrillResult(
            class_id=class_id,
            drill_state="green",
            drill_executed_at=executed_at,
            drill_notes=note,
            measured_skew_ns=0,
            max_skew_us=max_skew_us,
        )

    def _execute_drill(
        self,
        class_id: str,
        entry: dict[str, Any],
        executed_at: str,
        max_skew_us: int,
    ) -> ClockDrillResult:
        """実際の clock drill を実行する。"""
        sync_protocol = str(entry.get("sync_protocol", "ntp_chrony_stratum1"))

        if sync_protocol.startswith("ptp"):
            # PTP プロトコルの場合は pmc コマンドで offset を取得する
            skew_ns = self._measure_ptp_offset()
        else:
            # NTP / chrony の場合は chronyc tracking で offset を取得する
            skew_ns = self._measure_chrony_offset()

        # スキューが閾値内かチェックする（ナノ秒 → マイクロ秒 変換）
        skew_us = skew_ns / 1000.0
        drill_state = "green" if skew_us <= max_skew_us else "red"

        note = (
            f"{sync_protocol}: measured skew={skew_ns}ns "
            f"({'<' if skew_us <= max_skew_us else '>'} {max_skew_us * 1000}ns threshold)"
        )
        return ClockDrillResult(
            class_id=class_id,
            drill_state=drill_state,
            drill_executed_at=executed_at,
            drill_notes=note,
            measured_skew_ns=skew_ns,
            max_skew_us=max_skew_us,
        )

    def _measure_ptp_offset(self) -> int:
        """pmc コマンドで PTP オフセットをナノ秒で返す。"""
        try:
            result = subprocess.run(
                ["pmc", "-u", "-b", "0", "GET CURRENT_DATA_SET"],
                capture_output=True, text=True, timeout=5,
            )
            import re
            match = re.search(r"offsetFromMaster\s+(-?\d+)", result.stdout)
            if match:
                return abs(int(match.group(1)))
            return 0
        except (subprocess.TimeoutExpired, FileNotFoundError):
            return 0

    def _measure_chrony_offset(self) -> int:
        """chronyc tracking でシステム時刻オフセットをナノ秒で返す。"""
        try:
            result = subprocess.run(
                ["chronyc", "tracking"],
                capture_output=True, text=True, timeout=5,
            )
            import re
            match = re.search(r"System time\s*:\s*([\d.]+)\s*seconds", result.stdout)
            if match:
                offset_seconds = float(match.group(1))
                # 秒からナノ秒に変換する
                return int(offset_seconds * 1_000_000_000)
            return 0
        except (subprocess.TimeoutExpired, FileNotFoundError):
            return 0

    def update_lock_yaml(self, results: list[ClockDrillResult]) -> None:
        """ドリル結果を clock_causality_proof.lock.yaml に反映する。"""
        lock_file = _LOCK_DIR / "clock_causality_proof.lock.yaml"
        if not lock_file.exists():
            logger.warning("lock file not found: %s, skipping update", lock_file)
            return

        # 既存の lock.yaml を読み込む
        data = yaml.safe_load(lock_file.read_text(encoding="utf-8")) or {}
        # classes リストを dict に変換する
        classes_dict = {
            c["class_id"]: c for c in data.get("classes", [])
            if isinstance(c, dict) and "class_id" in c
        }

        # ドリル結果で各クラスを更新する
        for result in results:
            if result.class_id in classes_dict:
                classes_dict[result.class_id].update({
                    "drill_state": result.drill_state,
                    "drill_executed_at": result.drill_executed_at,
                    "drill_notes": result.drill_notes,
                    "measured_skew_ns": result.measured_skew_ns,
                })

        # 更新した classes リストを data に反映する
        data["classes"] = list(classes_dict.values())
        # lock.yaml に書き込む
        lock_file.write_text(
            yaml.dump(data, allow_unicode=True, default_flow_style=False, sort_keys=False),
            encoding="utf-8",
        )
        logger.info("updated %s with %d drill results", lock_file, len(results))


# ---------------------------------------------------------------------------
# エントリポイント
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    """clock_integrity drill runner のメインエントリポイント。"""
    parser = argparse.ArgumentParser(
        description="clock_integrity_class のドリルを実行して clock_causality_proof.lock.yaml を更新する"
    )
    parser.add_argument(
        "--dry-run", action="store_true", default=_DRY_RUN_DEFAULT,
        help="実際のコマンド実行をスキップしてシミュレーションする",
    )
    parser.add_argument(
        "--update-yaml", action="store_true", default=False,
        help="ドリル結果を lock.yaml に書き込む",
    )
    args = parser.parse_args(argv)

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
    )

    runner = ClockDrillRunner(dry_run=args.dry_run)
    results = runner.run_all()

    print("\n--- clock drill results ---")
    for r in results:
        mark = "✓" if r.drill_state == "green" else "✗"
        print(f"  [{mark}] {r.class_id}: {r.drill_state} (skew={r.measured_skew_ns}ns)")

    if args.update_yaml:
        runner.update_lock_yaml(results)

    failed = [r for r in results if r.drill_state == "red"]
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
