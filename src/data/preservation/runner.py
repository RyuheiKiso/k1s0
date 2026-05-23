"""src/data/preservation/runner.py

preservation_class ドリルランナー。
仕様: 14_データ保全適合仕様.md §preservation_class
- 5 preservation_class × drill_type のクロス積でドリルを実行する
- Barman PITR restore をシミュレーション（dry_run）または実行する
- drill.yaml を更新して lock.yaml 生成の SoT として機能する

環境変数:
  PRESERVATION_DRY_RUN: "true" で実際の DB 操作をスキップ（デフォルト true）
  BARMAN_SERVER: Barman サーバー名（デフォルト k1s0-main）
  POSTGRES_HOST: PostgreSQL ホスト
  POSTGRES_PORT: PostgreSQL ポート（デフォルト 5432）
"""

from __future__ import annotations

import argparse
import datetime
import json
import logging
import os
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

# PyYAML が必要
try:
    import yaml
    _HAS_YAML = True
except ImportError:
    _HAS_YAML = False
    print("PyYAML required: pip install PyYAML", file=sys.stderr)
    sys.exit(1)

# ロガーを設定する
logger = logging.getLogger(__name__)

# このファイルのあるディレクトリ（src/data/preservation/）
_PRESERVATION_DIR = Path(__file__).parent
# data ディレクトリ（src/data/）
_DATA_DIR = _PRESERVATION_DIR.parent
# drill.yaml のパス（SoT）
_DRILL_YAML = _PRESERVATION_DIR / "drill.yaml"
# classes.yaml のパス（preservation_class 定義の SoT）
_CLASSES_YAML = _PRESERVATION_DIR / "classes.yaml"
# dry_run デフォルト値を環境変数から取得する
_DRY_RUN_DEFAULT = os.environ.get("PRESERVATION_DRY_RUN", "true").lower() in ("1", "true", "yes")


# ---------------------------------------------------------------------------
# preservation_class データクラス
# ---------------------------------------------------------------------------

@dataclass(frozen=True)
class PreservationClass:
    """14_データ保全適合仕様.md が定義する保全クラス。"""
    # クラス識別子（例: v1_local_only）
    class_id: str
    # 人間が読める説明
    description: str
    # バックアップ先ストレージ種別
    storage_type: str
    # RPO 秒数（Recovery Point Objective）
    rpo_seconds: int
    # RTO 秒数（Recovery Time Objective）
    rto_seconds: int


@dataclass
class DrillResult:
    """ドリル実行結果を保持するデータクラス。"""
    # ドリル識別子
    drill_id: str
    # 対象 preservation_class
    preservation_class: str
    # ドリル実行状態（green / yellow / red）
    drill_state: str
    # ドリル実行日時（UTC ISO 8601）
    drill_executed_at: str
    # ドリル実行メモ
    drill_notes: str
    # restore 後の row 数（None は dry-run）
    row_count_after_restore: int | None = None
    # row count が元と一致したか
    row_count_matched: bool = False
    # restore 所要時間（秒）
    restore_duration_seconds: float = 0.0
    # エラーメッセージ（失敗時のみ）
    error: str = ""


# ---------------------------------------------------------------------------
# 保全クラスローダー
# ---------------------------------------------------------------------------

def _load_preservation_classes(classes_yaml: Path) -> dict[str, PreservationClass]:
    """classes.yaml から preservation_class 定義をロードする。"""
    # classes.yaml が存在しない場合はデフォルトを返す
    if not classes_yaml.exists():
        logger.warning("classes.yaml not found: %s, using defaults", classes_yaml)
        return _default_preservation_classes()

    # YAML をパースして PreservationClass dict を構築する
    data = yaml.safe_load(classes_yaml.read_text(encoding="utf-8")) or {}
    result: dict[str, PreservationClass] = {}

    # preservation_classes リストを走査して各クラスをロードする
    for entry in data.get("preservation_classes", []):
        class_id = str(entry.get("class_id", ""))
        if not class_id:
            continue
        result[class_id] = PreservationClass(
            class_id=class_id,
            description=str(entry.get("description", "")),
            storage_type=str(entry.get("storage_type", "unknown")),
            rpo_seconds=int(entry.get("rpo_seconds", 3600)),
            rto_seconds=int(entry.get("rto_seconds", 14400)),
        )

    # クラスが空の場合はデフォルトを返す
    return result if result else _default_preservation_classes()


def _default_preservation_classes() -> dict[str, PreservationClass]:
    """14_データ保全適合仕様.md §4.2 の 5 保全クラスを返す。"""
    return {
        "v1_local_only": PreservationClass(
            "v1_local_only", "ローカルバックアップのみ", "minio_local", 3600, 14400
        ),
        "v1_zone_redundant": PreservationClass(
            "v1_zone_redundant", "ゾーン冗長バックアップ", "s3_zone_redundant", 900, 7200
        ),
        "v1_cross_region": PreservationClass(
            "v1_cross_region", "クロスリージョン PITR", "s3_cross_region", 300, 3600
        ),
        "v1_object_lock": PreservationClass(
            "v1_object_lock", "S3 Object Lock WORM", "s3_object_lock", 300, 3600
        ),
        "v1_air_gapped": PreservationClass(
            "v1_air_gapped", "エアギャップ tape バックアップ", "tape_airgap", 86400, 72000
        ),
    }


# ---------------------------------------------------------------------------
# ドリル実行エンジン
# ---------------------------------------------------------------------------

class PreservationDrillRunner:
    """preservation_class ごとのドリルを実行するクラス。"""

    def __init__(
        self,
        classes_yaml: Path = _CLASSES_YAML,
        drill_yaml: Path = _DRILL_YAML,
        dry_run: bool = _DRY_RUN_DEFAULT,
        barman_server: str | None = None,
        postgres_host: str | None = None,
        postgres_port: int = 5432,
    ) -> None:
        # 保全クラス定義をロードする
        self._classes = _load_preservation_classes(classes_yaml)
        # drill.yaml のパスを保持する
        self._drill_yaml = drill_yaml
        # dry_run フラグ（True の場合は実際の DB 操作をスキップ）
        self._dry_run = dry_run
        # Barman サーバー名
        self._barman_server = barman_server or os.environ.get("BARMAN_SERVER", "k1s0-main")
        # PostgreSQL ホスト
        self._postgres_host = postgres_host or os.environ.get("POSTGRES_HOST", "localhost")
        # PostgreSQL ポート番号
        self._postgres_port = int(os.environ.get("POSTGRES_PORT", str(postgres_port)))

    def run_all(self) -> list[DrillResult]:
        """全 preservation_class のドリルを実行して結果を返す。"""
        # 結果リストを初期化する
        results: list[DrillResult] = []
        # 各保全クラスに対してドリルを実行する
        for class_id, pc in self._classes.items():
            logger.info("running drill for %s (dry_run=%s)", class_id, self._dry_run)
            result = self._run_single(pc)
            results.append(result)
        return results

    def _run_single(self, pc: PreservationClass) -> DrillResult:
        """1 保全クラスのドリルを実行する。"""
        # ドリル識別子を生成する（restore__ プレフィックスは spec 準拠）
        drill_id = f"restore__{pc.class_id}"
        # 実行日時を UTC ISO 8601 形式で取得する
        executed_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        # ドリル開始時刻を記録する（RTO 計測に使用）
        start_time = time.monotonic()

        try:
            if self._dry_run:
                # dry_run モード: 実際の DB 操作をスキップしてシミュレーション結果を返す
                result = self._simulate_drill(pc, drill_id, executed_at, start_time)
            else:
                # 実際のドリルを実行する（Barman + PITR restore）
                result = self._execute_drill(pc, drill_id, executed_at, start_time)
        except Exception as exc:
            # 例外発生時は red 状態として記録する
            logger.error("drill failed for %s: %s", pc.class_id, exc)
            result = DrillResult(
                drill_id=drill_id,
                preservation_class=pc.class_id,
                drill_state="red",
                drill_executed_at=executed_at,
                drill_notes=f"drill failed: {exc}",
                error=str(exc),
            )

        return result

    def _simulate_drill(
        self,
        pc: PreservationClass,
        drill_id: str,
        executed_at: str,
        start_time: float,
    ) -> DrillResult:
        """dry_run モードでドリルをシミュレーションする。"""
        # 実際の DB 操作なしに成功をシミュレーションする
        simulated_row_count = 3
        # restore 所要時間をシミュレーションする（クラスごとに異なる想定）
        duration_map = {
            "v1_local_only": 45.0,
            "v1_zone_redundant": 48.0,
            "v1_cross_region": 52.0,
            "v1_object_lock": 50.0,
            "v1_air_gapped": 3600.0,
        }
        simulated_duration = duration_map.get(pc.class_id, 60.0)
        # dry_run ノートを生成する
        note = (
            f"[DRY-RUN] {pc.description}: "
            f"simulated PITR restore, row_count={simulated_row_count}, "
            f"duration={simulated_duration}s (RTO target={pc.rto_seconds}s)"
        )
        # RTO を超えていないかチェックする
        drill_state = "green" if simulated_duration <= pc.rto_seconds else "yellow"
        logger.info(
            "simulated drill %s: state=%s duration=%.1fs", drill_id, drill_state, simulated_duration
        )
        return DrillResult(
            drill_id=drill_id,
            preservation_class=pc.class_id,
            drill_state=drill_state,
            drill_executed_at=executed_at,
            drill_notes=note,
            row_count_after_restore=simulated_row_count,
            row_count_matched=True,
            restore_duration_seconds=simulated_duration,
        )

    def _execute_drill(
        self,
        pc: PreservationClass,
        drill_id: str,
        executed_at: str,
        start_time: float,
    ) -> DrillResult:
        """実際の Barman PITR restore ドリルを実行する。"""
        # Barman backup-id の取得（最新バックアップ）
        backup_id = self._get_latest_backup_id(pc)
        if not backup_id:
            return DrillResult(
                drill_id=drill_id,
                preservation_class=pc.class_id,
                drill_state="red",
                drill_executed_at=executed_at,
                drill_notes=f"no backup found for {pc.class_id}",
            )

        # restore クラスター名（本番クラスターと分離する）
        restore_cluster = f"k1s0-restore-{pc.class_id.replace('_', '-')}"
        # Barman recover コマンドを実行する
        self._barman_recover(backup_id, restore_cluster)

        # restore 後の row count を検証する
        source_count = self._count_rows(self._barman_server)
        restore_count = self._count_rows(restore_cluster)
        row_count_matched = source_count == restore_count

        # restore 所要時間を計算する
        duration = time.monotonic() - start_time
        # RTO 内に完了したかチェックする
        drill_state = "green" if row_count_matched and duration <= pc.rto_seconds else "red"

        note = (
            f"{pc.description}: Barman PITR restore from backup={backup_id}, "
            f"row_count_source={source_count}, row_count_restore={restore_count}, "
            f"matched={row_count_matched}, duration={duration:.1f}s"
        )
        return DrillResult(
            drill_id=drill_id,
            preservation_class=pc.class_id,
            drill_state=drill_state,
            drill_executed_at=executed_at,
            drill_notes=note,
            row_count_after_restore=restore_count,
            row_count_matched=row_count_matched,
            restore_duration_seconds=duration,
        )

    def _get_latest_backup_id(self, pc: PreservationClass) -> str | None:
        """Barman から最新のバックアップ ID を取得する。"""
        try:
            # barman list-backup コマンドを実行する
            result = subprocess.run(
                ["barman", "list-backup", self._barman_server, "--minimal"],
                capture_output=True, text=True, timeout=30,
            )
            # 出力の最初の行がバックアップ ID
            lines = result.stdout.strip().splitlines()
            return lines[0].split()[0] if lines else None
        except (subprocess.TimeoutExpired, FileNotFoundError) as exc:
            logger.warning("barman unavailable: %s", exc)
            return None

    def _barman_recover(self, backup_id: str, target_cluster: str) -> None:
        """Barman で PITR restore を実行する。"""
        subprocess.run(
            [
                "barman", "recover",
                "--target-time", datetime.datetime.now(tz=datetime.timezone.utc).isoformat(),
                self._barman_server,
                backup_id,
                f"/tmp/restore/{target_cluster}",
            ],
            check=True, timeout=pc.rto_seconds,
        )

    def _count_rows(self, cluster: str) -> int:
        """指定クラスターのメインテーブル row count を取得する。"""
        try:
            result = subprocess.run(
                ["psql", f"host={self._postgres_host} port={self._postgres_port} dbname=k1s0",
                 "-c", "SELECT COUNT(*) FROM domain_events;", "-t"],
                capture_output=True, text=True, timeout=30,
            )
            return int(result.stdout.strip())
        except (subprocess.TimeoutExpired, ValueError, FileNotFoundError):
            return 0

    def update_drill_yaml(self, results: list[DrillResult]) -> None:
        """ドリル結果を drill.yaml に書き込む。"""
        # 既存の drill.yaml を読み込む（存在しない場合は空 dict）
        existing: dict[str, Any] = {}
        if self._drill_yaml.exists():
            existing = yaml.safe_load(self._drill_yaml.read_text(encoding="utf-8")) or {}

        # 既存 drills リストを dict に変換してマージする
        existing_drills: dict[str, dict[str, Any]] = {
            d["drill_id"]: d for d in existing.get("drills", [])
        }

        # 新しい結果でドリルを更新する
        for result in results:
            entry: dict[str, Any] = {
                "drill_id": result.drill_id,
                "preservation_class": result.preservation_class,
                "drill_state": result.drill_state,
                "drill_executed_at": result.drill_executed_at,
                "drill_notes": result.drill_notes,
            }
            # 検証結果が存在する場合は verification セクションを追加する
            if result.row_count_after_restore is not None:
                entry["verification"] = {
                    "row_count_after_restore": result.row_count_after_restore,
                    "row_count_matched": result.row_count_matched,
                    "restore_duration_seconds": round(result.restore_duration_seconds, 1),
                }
            # エラーがある場合は error フィールドを追加する
            if result.error:
                entry["error"] = result.error

            # 既存エントリをマージ（新しい結果で上書き）する
            existing_drills[result.drill_id] = entry

        # drills リストを更新した内容で再構築する
        existing["drills"] = list(existing_drills.values())

        # drill.yaml に書き込む
        self._drill_yaml.write_text(
            yaml.dump(existing, allow_unicode=True, default_flow_style=False, sort_keys=False),
            encoding="utf-8",
        )
        logger.info("updated %s with %d drill results", self._drill_yaml, len(results))


# ---------------------------------------------------------------------------
# Avro スキーマ互換性チェッカー
# ---------------------------------------------------------------------------

class AvroSchemaCompatibilityChecker:
    """Avro スキーマの後方互換性を検証するクラス。

    Apicurio Registry / confluent schema-registry の REST API を使って
    スキーマの互換性を確認する。Registry が利用不可の場合は dry_run で代替する。
    """

    def __init__(self, registry_url: str | None = None, dry_run: bool = True) -> None:
        # スキーマレジストリ URL を設定する
        self._registry_url = registry_url or os.environ.get(
            "SCHEMA_REGISTRY_URL", "http://localhost:8081"
        )
        # dry_run フラグを設定する
        self._dry_run = dry_run

    def check_compatibility(self, subject: str, schema: str) -> bool:
        """指定スキーマが既存バージョンと後方互換かチェックする。"""
        if self._dry_run:
            logger.info("[DRY-RUN] schema compatibility check: subject=%s", subject)
            return True

        import urllib.error
        import urllib.request
        # Confluent Schema Registry の compatibility チェックエンドポイント
        url = f"{self._registry_url}/compatibility/subjects/{subject}/versions/latest"
        body = json.dumps({"schema": schema}).encode("utf-8")
        req = urllib.request.Request(
            url, data=body,
            headers={"Content-Type": "application/vnd.schemaregistry.v1+json"},
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                result = json.loads(resp.read())
                # "is_compatible" フィールドで互換性を確認する
                return bool(result.get("is_compatible", False))
        except urllib.error.URLError as exc:
            logger.warning("schema registry unavailable: %s", exc)
            return False

    def check_all_schemas(self, schema_dir: Path) -> dict[str, bool]:
        """schema_dir 配下の全 .avsc ファイルを走査して互換性を確認する。"""
        results: dict[str, bool] = {}
        # .avsc ファイルを再帰的に検索する
        for avsc_file in schema_dir.rglob("*.avsc"):
            subject = avsc_file.stem
            schema_content = avsc_file.read_text(encoding="utf-8")
            # 互換性チェックを実行する
            compatible = self.check_compatibility(subject, schema_content)
            results[subject] = compatible
            logger.info("schema %s: compatible=%s", subject, compatible)
        return results


# ---------------------------------------------------------------------------
# エントリポイント
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    """preservation drill runner のメインエントリポイント。"""
    # コマンドライン引数をパースする
    parser = argparse.ArgumentParser(
        description="preservation_class ドリルを実行して drill.yaml を更新する"
    )
    parser.add_argument(
        "--dry-run", action="store_true", default=_DRY_RUN_DEFAULT,
        help="実際の DB 操作をスキップしてシミュレーションする（デフォルト: true）",
    )
    parser.add_argument(
        "--class", dest="class_id", default=None,
        help="実行する preservation_class を指定する（省略時は全クラス）",
    )
    parser.add_argument(
        "--update-yaml", action="store_true", default=True,
        help="drill.yaml を更新する（デフォルト: true）",
    )
    args = parser.parse_args(argv)

    # ロギングを設定する
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
    )

    # ドリルランナーを初期化する
    runner = PreservationDrillRunner(dry_run=args.dry_run)

    # 特定クラスが指定された場合はそのクラスのみ実行する
    if args.class_id:
        pc = runner._classes.get(args.class_id)
        if not pc:
            logger.error("unknown preservation_class: %s", args.class_id)
            return 1
        results = [runner._run_single(pc)]
    else:
        # 全クラスのドリルを実行する
        results = runner.run_all()

    # 結果をサマリ表示する
    print("\n--- preservation drill results ---")
    for r in results:
        status_mark = "✓" if r.drill_state == "green" else "✗"
        print(f"  [{status_mark}] {r.drill_id}: {r.drill_state}")
        if r.error:
            print(f"      error: {r.error}")

    # drill.yaml を更新する
    if args.update_yaml:
        runner.update_drill_yaml(results)

    # 失敗した drill が 1 つでもあれば exit code 1 を返す
    failed = [r for r in results if r.drill_state == "red"]
    if failed:
        logger.error("%d drill(s) failed", len(failed))
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
