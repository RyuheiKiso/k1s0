"""tools/lock_yaml_generator/generate_build_evidence.py

build_evidence.lock.yaml 生成器。
src/_meta/build_evidence/evidence_entries.yaml を読み込み
build evidence エントリを正規化して lock.yaml を生成する。
evidence() DSL 関数が参照する build artifact。
"""

from __future__ import annotations

import datetime
from pathlib import Path
from typing import Any

from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT

# evidence_entries.yaml の想定パス
_EVIDENCE_ENTRIES_YAML = REPO_ROOT / "src/_meta/build_evidence/evidence_entries.yaml"

# 有効な evidence_kind 値の一覧 (dsl.py の _VALID_EVIDENCE_KINDS と同期すること)
_VALID_EVIDENCE_KINDS: frozenset[str] = frozenset([
    "cargo_build_pass",
    "pnpm_test_pass",
    "buf_lint_pass",
    "buf_lint_observability_pii_pass",
    "buf_generate_drift_zero",
    "testcontainers_e2e_pass",
    "kind_cluster_drill_pass",
    "pgtap_rls_force_pass",
    "playwright_8_scenario_pass",
    "openbao_transit_sign_verify_pass",
    "axe_core_zero_violation",
    "cargo_public_api_drift_zero",
    "cargo_deny_pass",
    "cargo_test_migration_pair_pass",
    "eslint_boundaries_pass",
    "bfl_oidc_e2e_pass",
    "slo_burn_rate_test_pass",
    "quota_enforcement_e2e_pass",
    "atomic_triple_write_4lang_pass",
    "cross_tenant_e2e_4lang_pass",
    "vitest_reducer_4subtype_pass",
    "sdk_dist_5class_e2e_pass",
    "cosign_verify_pass",
])


class BuildEvidenceGenerator(BaseGenerator):
    """build_evidence.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "build_evidence.lock.yaml"
    # 必須入力ファイルなし (evidence_entries.yaml があれば使用する)
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/_meta/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """src/_meta/build_evidence/evidence_entries.yaml を読み込む。"""
        # evidence_entries.yaml が存在する場合は読み込む
        if _EVIDENCE_ENTRIES_YAML.exists():
            # yaml モジュールをインポートする
            import yaml  # type: ignore
            # evidence_entries.yaml を読み込む
            raw = yaml.safe_load(_EVIDENCE_ENTRIES_YAML.read_text(encoding="utf-8"))
            # dict 型であれば返す、そうでなければ空 dict を返す
            return raw if isinstance(raw, dict) else {}
        # ファイルが存在しない場合は空 dict を返す
        return {}

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """evidence エントリを正規化して artifact dict を返す。"""
        # 生成日時を取得する
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # evidence_entries リストを inputs から取得する
        raw_entries: list[dict[str, Any]] = inputs.get("evidence_entries", [])

        # 各エントリを正規化する
        entries: list[dict[str, Any]] = []
        missing_count = 0
        verified_count = 0

        for entry in raw_entries:
            # cell_id が存在する場合のみ処理する
            if not isinstance(entry, dict) or "cell_id" not in entry:
                continue

            # build_evidence_id が設定されているか確認する
            build_evidence_id: str = str(entry.get("build_evidence_id", ""))
            has_evidence = bool(build_evidence_id)

            # evidence_kind を build_evidence_id から抽出する (先頭のアンダースコア区切り部分)
            evidence_kind = ""
            if has_evidence:
                # build_evidence_id から有効な kind を抽出する
                for kind in sorted(_VALID_EVIDENCE_KINDS, key=len, reverse=True):
                    if kind in build_evidence_id:
                        evidence_kind = kind
                        break

            # エントリを構築する
            normalized: dict[str, Any] = {
                # セル識別子
                "cell_id": entry["cell_id"],
                # build evidence ID (CI run id 形式)
                "build_evidence_id": build_evidence_id,
                # 抽出した evidence kind
                "evidence_kind": evidence_kind,
                # evidence が設定されているかどうか
                "has_evidence": has_evidence,
                # 証拠収集日時
                "collected_at": entry.get("collected_at", ""),
                # 備考
                "notes": entry.get("notes", ""),
            }
            entries.append(normalized)

            # カウントを更新する
            if has_evidence:
                verified_count += 1
            else:
                missing_count += 1

        # artifact dict を返す
        return {
            # 自動生成ヘッダー
            "_AUTO_GENERATED": True,
            # 生成器名
            "_generator": "generate_build_evidence",
            # 生成日時
            "generated_at": generated_at,
            # 仕様 ID
            "spec_id": "detail.release_gate.build_evidence",
            # 総エントリ数
            "total_entries": len(entries),
            # evidence 設定済みエントリ数
            "verified_count": verified_count,
            # evidence 未設定エントリ数 (yellow 扱い)
            "missing_count": missing_count,
            # 全エントリが evidence 設定済みかどうか
            "all_evidenced": missing_count == 0,
            # evidence エントリリスト
            "evidence_entries": entries,
        }
