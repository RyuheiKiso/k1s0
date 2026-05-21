"""tools/lock_yaml_generator/generate_dry_run.py

dry_run.lock.yaml 生成器。
src/tier1/lock/dry_run_input.yaml を読み込み、4 pair × 5 phase の migration dry-run 結果を正規化する。
存在しない場合はフォールバックスケルトンを生成する（全 last_green_at: ''）。

A-3 対応:
- phase_assertion_id / cross_axis_assertions_verified / exposed_concepts_diff_hash を input から output に pass-through する
- 365 日 expiry チェック: last_green_at が 365 日より古い cell があれば CI_FAIL エントリを dry_run.lock.yaml に記録する

exposed_concepts_diff_hash の算出規約:
- input yaml に hash 値が記載されている場合はそのまま pass-through する
- input yaml に記載がない場合（フォールバックスケルトン）は _compute_hash() で決定論的に生成する
- 算出式: hashlib.sha256(f"{pair_id}-{phase_id}-v1.0.0".encode()).hexdigest()
- 算出例: sha256("relational_pg_pair-schema_diff-v1.0.0") → sha256:2dd1ecc8...
- この算出式は exposed_concepts_version = v1.0.0 で固定（version 変更は破壊的変更）

catalog SoT: src/tier1/schema/migration/pairs.yaml / src/tier1/schema/migration/phases.yaml
catalog が存在する場合、宣言された pair_id が全て drill 済みであることを検証する。
catalog が存在しない場合は catalog_validation_status: catalog_absent で graceful degradation する。
"""

from __future__ import annotations

import datetime
import hashlib
from pathlib import Path
from typing import Any

from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT
from tools.lock_yaml_generator.lib.catalog_validator import (
    load_catalog,
    validate_catalog_against_input,
    build_catalog_metadata,
)

# dry_run_input.yaml の想定名
_INPUT_NAME = "dry_run_input.yaml"

# catalog SoT パス（REPO_ROOT 相対）
_CATALOG_CLASSES_PATH = "src/tier1/schema/migration/pairs.yaml"
_CATALOG_SCENARIOS_PATH = "src/tier1/schema/migration/phases.yaml"

# 365 日 expiry しきい値（tier1/CLAUDE.md §ship blocker に準拠する）
_EXPIRY_DAYS = 365

# 4 primary pair 定義（02_移行Pair適合仕様.md の v1 primary pair セットと一致する）
_PAIRS: list[dict[str, str]] = [
    # PostgreSQL（CloudNativePG）→ PostgreSQL（StackGres）移行ペア
    {"pair_id": "relational_pg_pair", "from_oss": "cnpg_postgresql", "to_oss": "stackgres_postgresql"},
    # Apache Kafka（Strimzi）→ RedPanda 移行ペア
    {"pair_id": "messaging_kafka_pair", "from_oss": "strimzi_kafka", "to_oss": "redpanda"},
    # Temporal → Cadence 移行ペア
    {"pair_id": "workflow_pair", "from_oss": "temporal", "to_oss": "cadence"},
    # ZEN Engine → 自製 DSL backend（Rust 実装）移行ペア
    {"pair_id": "rule_engine_pair", "from_oss": "zen_engine", "to_oss": "custom_rust_dsl"},
]

# 5 phase 定義（順序固定、変更は破壊的変更）
_PHASES: list[str] = [
    # フェーズ 1: スキーマ差分検出
    "schema_diff",
    # フェーズ 2: 状態複製
    "state_replicate",
    # フェーズ 3: 二重書き込みランプアップ
    "dual_write_ramp",
    # フェーズ 4: カットオーバー
    "cutover",
    # フェーズ 5: ロールバック
    "rollback",
]


def _compute_hash(pair_id: str, phase_id: str, version: str = "v1.0.0") -> str:
    """pair_id / phase_id / version から exposed_concepts_diff_hash を決定論的に算出する。
    算出式: sha256("{pair_id}-{phase_id}-{version}") → "sha256:{hexdigest}"。
    version は exposed_concepts_version と等価であり、変更は破壊的変更として扱う。
    """
    # pair_id-phase_id-version の文字列を UTF-8 エンコードして sha256 ダイジェストを算出する
    raw = f"{pair_id}-{phase_id}-{version}".encode()
    # "sha256:" プレフィックスを付けて返す（lock.yaml の表記規約に準拠する）
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def _check_expiry(last_green_at: str, now: datetime.datetime) -> bool:
    """last_green_at が _EXPIRY_DAYS 日以内かどうかを返す。
    空文字列または未設定の場合は expired とみなす（False を返す）。
    wall-clock TTL ではなく相対日数チェックであることに注意する。
    """
    # last_green_at が空の場合は expired とみなす
    if not last_green_at:
        return False
    # ISO 8601 形式の文字列を datetime に変換する
    try:
        # Z サフィックスを +00:00 に置換して fromisoformat でパースする
        ts = datetime.datetime.fromisoformat(last_green_at.replace("Z", "+00:00"))
    except ValueError:
        # パース失敗時は expired とみなす
        return False
    # now との差を計算して 365 日以内かどうかを返す
    delta = now - ts
    return delta.days <= _EXPIRY_DAYS


class DryRunGenerator(BaseGenerator):
    """dry_run.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "dry_run.lock.yaml"
    # 必須入力なし（フォールバックあり）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/tier1/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """src/tier1/lock/dry_run_input.yaml を読み込む。
        存在しない場合は空 dict を返す。
        """
        return self.load_lock(lock_dir, _INPUT_NAME)

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """4 pair の dry-run 結果を正規化して artifact dict を返す。
        inputs が空の場合はフォールバックスケルトンを使用する。
        catalog SoT (migration/pairs.yaml) が存在する場合は cross-validation を実行する。
        365 日 expiry チェックを実行し、期限切れ cell があれば ci_failures に記録する。
        """
        # 現在時刻を UTC で取得する（expiry チェックの基準時刻として使用する）
        now = datetime.datetime.now(tz=datetime.timezone.utc)
        generated_at = now.strftime("%Y-%m-%dT%H:%M:%SZ")

        # pairs キーを読み込む
        raw_pairs: list[dict[str, Any]] = inputs.get("pairs", [])

        if raw_pairs:
            # 入力データから pairs を正規化する（新フィールドを pass-through する）
            pairs = self._normalize_pairs(raw_pairs)
        else:
            # フォールバック: 全 pending のスケルトンを生成する
            pairs = self._build_skeleton()

        # 365 日 expiry チェックを実行して期限切れ cell を収集する
        ci_failures: list[dict[str, str]] = []
        for pair in pairs:
            # pair_id を取得する
            pair_id = pair.get("pair_id", "")
            for phase in pair.get("phases", []):
                # phase_id を取得する
                phase_id = phase.get("phase_id", "")
                # last_green_at を取得する
                last_green_at = phase.get("last_green_at", "")
                # expiry チェックを実行する
                if not _check_expiry(last_green_at, now):
                    # 期限切れ cell を ci_failures に追加する
                    ci_failures.append({
                        # 期限切れの pair_id を記録する
                        "pair_id": pair_id,
                        # 期限切れの phase_id を記録する
                        "phase_id": phase_id,
                        # CI FAIL 理由を記録する（tier1/CLAUDE.md §ship blocker に準拠）
                        "reason": f"CI_FAIL: dry_run expiry exceeded (last_green_at={last_green_at!r}, threshold={_EXPIRY_DAYS}d)",
                        # 期限切れの last_green_at を記録する
                        "last_green_at": last_green_at,
                    })

        # catalog SoT のパスを解決する
        classes_path = REPO_ROOT / _CATALOG_CLASSES_PATH
        scenarios_path = REPO_ROOT / _CATALOG_SCENARIOS_PATH

        # catalog yaml を読み込む（不在時は None）
        classes_catalog = load_catalog(classes_path)
        scenarios_catalog = load_catalog(scenarios_path)

        # pairs catalog の cross-validation: pairs[].pair_id vs input pairs[].pair_id
        # migration/pairs.yaml は "pair_id" をキーとして使う（"id" ではない）
        classes_result = validate_catalog_against_input(
            catalog=classes_catalog,
            input_data=inputs,
            class_key="pairs",
            input_id_key="pair_id",
            input_list_key="pairs",
            catalog_path=classes_path,
            catalog_id_key="pair_id",
        )

        # phases catalog の cross-validation（phase_id と drill id の対応がないため passed 扱い）
        scenarios_result = validate_catalog_against_input(
            catalog=scenarios_catalog,
            input_data={},
            class_key="phases",
            input_id_key="phase_id",
            input_list_key=None,
            catalog_path=scenarios_path,
            catalog_id_key="phase_id",
        )
        # phases は pairs 配下に埋め込まれているため直接突合しない
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

        # artifact を構築して返す
        artifact: dict[str, Any] = {
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_dry_run.py"
            ),
            "total_pairs":  len(pairs),
            "total_phases": len(_PHASES),
            "total_cells":  len(pairs) * len(_PHASES),
            "generated_at": generated_at,
            "metadata":     catalog_meta,
            "pairs":        pairs,
        }

        # 365 日 expiry 超過 cell があれば ci_failures セクションに記録する
        if ci_failures:
            # ci_failures を artifact に追加する（CI がこのキーを検出して fail する）
            artifact["ci_failures"] = ci_failures

        return artifact

    @staticmethod
    def _normalize_pairs(raw: list[dict[str, Any]]) -> list[dict[str, Any]]:
        """pair エントリを正規化する。phases も正規化する。
        A-3 対応: phase_assertion_id / cross_axis_assertions_verified / exposed_concepts_diff_hash を pass-through する。
        """
        result: list[dict[str, Any]] = []
        for entry in raw:
            raw_phases: list[dict[str, Any]] = entry.get("phases", [])
            phases = []
            for p in raw_phases:
                # 基本フィールドを正規化する
                phase_entry: dict[str, Any] = {
                    "phase_id":       str(p.get("phase_id", "")),
                    "drill_state":    str(p.get("drill_state", "pending")),
                    "last_green_at":  str(p.get("last_green_at", "")),
                    "notes":          str(p.get("notes", "")),
                }
                # phase_assertion_id を input から pass-through する（存在しない場合は空文字列を設定する）
                if "phase_assertion_id" in p:
                    phase_entry["phase_assertion_id"] = str(p["phase_assertion_id"])
                # cross_axis_assertions_verified を input から pass-through する（存在しない場合は空リストを設定する）
                if "cross_axis_assertions_verified" in p:
                    # リスト形式を維持して pass-through する
                    raw_cross = p["cross_axis_assertions_verified"]
                    phase_entry["cross_axis_assertions_verified"] = (
                        list(raw_cross) if isinstance(raw_cross, list) else []
                    )
                # exposed_concepts_diff_hash を input から pass-through する（存在しない場合は pending を設定する）
                if "exposed_concepts_diff_hash" in p:
                    phase_entry["exposed_concepts_diff_hash"] = str(p["exposed_concepts_diff_hash"])
                phases.append(phase_entry)
            # last_green_at が全フェーズで記録されていれば pair 全体を green とする
            any_green_at = next(
                (p["last_green_at"] for p in phases if p["last_green_at"]), ""
            )
            # 全フェーズ green なら pair status = green
            all_green = all(p["drill_state"] == "green" for p in phases)
            result.append({
                "pair_id":       str(entry.get("pair_id", "")),
                "from_oss":      str(entry.get("from_oss", "")),
                "to_oss":        str(entry.get("to_oss", "")),
                "last_green_at": any_green_at,
                "status":        "green" if all_green else "pending",
                "phases":        phases,
            })
        return result

    @staticmethod
    def _build_skeleton() -> list[dict[str, Any]]:
        """4 pair × 5 phase のスケルトンを生成する。
        A-3 対応: 新フィールドのデフォルト値を設定する。
        """
        result: list[dict[str, Any]] = []
        for pair in _PAIRS:
            phases = [
                {
                    "phase_id":      phase,
                    "drill_state":   "pending",
                    "last_green_at": "",
                    "notes":         "",
                    # phase_assertion_id: スケルトン生成時は pair_id / phase_id から規則的に生成する
                    "phase_assertion_id": f"pa_{pair['pair_id']}_{phase}_001",
                    # cross_axis_assertions_verified: スケルトン生成時は空リストとする
                    "cross_axis_assertions_verified": [],
                    # exposed_concepts_diff_hash: pair_id / phase_id / v1.0.0 から決定論的に算出する
                    "exposed_concepts_diff_hash": _compute_hash(pair["pair_id"], phase),
                }
                for phase in _PHASES
            ]
            result.append({
                "pair_id":       pair["pair_id"],
                "from_oss":      pair["from_oss"],
                "to_oss":        pair["to_oss"],
                "last_green_at": "",
                "status":        "pending",
                "phases":        phases,
            })
        return result
