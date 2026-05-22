"""tools/lock_yaml_generator/generate_coverage_oracle.py

trace_coverage.lock.yaml 生成器（coverage oracle）。

trace_ledger.lock.yaml から FR-ID 一覧を読み込み、src 配下の IMPL-ID マーカーと
照合して FR-ID → IMPL-ID の被覆率（fr_to_impl_ratio）を計算する。

IMPL-ID マーカー形式（src 各言語）:
  Rust / Go / C# / TS: // k1s0-impl: IMPL-<axis>-<NNNN> realizes=FR-<axis>-<NNN>
  Python (tools/):      # k1s0-impl: IMPL-<axis>-<NNNN> realizes=FR-<axis>-<NNN>

フェーズ別達成目標:
  R3 開始: fr_to_impl_ratio = 0.0（マーカー未付与 → yellow 解消は threshold=0.0 で green）
  R3 完了: fr_to_impl_ratio >= 0.95（src 主要モジュールへのマーカー注入完了）
"""

from __future__ import annotations

import datetime
import re
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:
    raise ImportError("PyYAML required: pip install PyYAML")

from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT, sha256_of_text

# IMPL-ID マーカーのパターン（Rust/Go/C#/TS コメント形式）
_IMPL_MARKER_PATTERN = re.compile(
    r"//\s*k1s0-impl:\s*(IMPL-[a-z][a-z0-9_]*-[0-9]{4})\s+realizes=(FR-[a-z][a-z0-9_]*-[0-9]{3})"
)
# Python コメント形式
_IMPL_MARKER_PATTERN_PY = re.compile(
    r"#\s*k1s0-impl:\s*(IMPL-[a-z][a-z0-9_]*-[0-9]{4})\s+realizes=(FR-[a-z][a-z0-9_]*-[0-9]{3})"
)

# src 走査対象拡張子と対応パターン
_SRC_EXTENSIONS = {
    ".rs":  _IMPL_MARKER_PATTERN,
    ".go":  _IMPL_MARKER_PATTERN,
    ".cs":  _IMPL_MARKER_PATTERN,
    ".ts":  _IMPL_MARKER_PATTERN,
    ".tsx": _IMPL_MARKER_PATTERN,
    ".py":  _IMPL_MARKER_PATTERN_PY,
}

# 走査除外ディレクトリ（build artifact / external deps）
_EXCLUDE_DIRS = {"target", "node_modules", ".git", "__pycache__", ".cache"}


def _scan_src_impl_ids() -> list[dict[str, str]]:
    """src/ および tools/ 配下の全コードファイルを走査して IMPL-ID マーカーを抽出する。

    tools/ を含む理由: infra/data/security/ops/formal 等で src コードが scaffold 段階の軸は、
    対応する lock.yaml generator（tools/lock_yaml_generator/*.py）が実質的な適合仕様の物理化を担うため、
    generator ファイルを IMPL-ID の対象とする。

    Returns:
        list of {"impl_id": ..., "fr_id": ..., "file": ..., "line": ...}
    """
    # 抽出した IMPL-ID エントリのリスト
    found: list[dict[str, str]] = []
    # src/ と tools/ の両方を走査する
    scan_dirs = [REPO_ROOT / "src", REPO_ROOT / "tools"]

    for scan_dir in scan_dirs:
        # ディレクトリが存在しない場合はスキップ
        if not scan_dir.exists():
            continue
        for ext, pattern in _SRC_EXTENSIONS.items():
            # 除外ディレクトリを回避しながら全ソースファイルを走査
            for file_path in scan_dir.rglob(f"*{ext}"):
                # 除外ディレクトリに属するファイルはスキップ
                if any(part in _EXCLUDE_DIRS for part in file_path.parts):
                    continue
                try:
                    text = file_path.read_text(encoding="utf-8", errors="ignore")
                except OSError:
                    continue
                # 各行でパターンマッチを実行
                for lineno, line in enumerate(text.splitlines(), 1):
                    m = pattern.search(line)
                    if m:
                        found.append({
                            "impl_id": m.group(1),
                            "fr_id":   m.group(2),
                            "file":    str(file_path.relative_to(REPO_ROOT)),
                            "line":    str(lineno),
                        })
    return found


class CoverageOracleGenerator(BaseGenerator):
    """trace_coverage.lock.yaml 生成器。

    docs 側の FR-ID（trace_ledger.lock.yaml）と src 側の IMPL-ID マーカーを照合し、
    FR-ID → IMPL-ID の被覆率を計算する。

    release_gate cell meta.trace_coverage_ratchet はこの lock を参照する。
    """

    # 出力ファイル名
    OUTPUT_NAME = "trace_coverage.lock.yaml"
    # 依存する入力 lock.yaml
    REQUIRED_INPUTS: list[str] = ["trace_ledger.lock.yaml"]
    # jsonschema 検証（スキーマは将来追加）
    SCHEMA_PATH: Path | None = None
    # デフォルト出力先
    DEFAULT_OUTPUT_DIR = "src/_meta/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """trace_ledger.lock.yaml を読み込む。"""
        # trace_ledger から FR-ID リストを取得する
        trace_ledger = self.load_lock(lock_dir, "trace_ledger.lock.yaml")
        return {"trace_ledger": trace_ledger, "lock_dir": lock_dir}

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """IMPL-ID 走査と FR-ID 照合を実行して coverage artifact を構築する。"""
        trace_ledger: dict[str, Any] = inputs.get("trace_ledger", {})

        # trace_ledger から FR-ID の集合を構築する
        fr_ids_from_ledger: set[str] = {
            edge.get("fr_id", "")
            for edge in (trace_ledger.get("edges") or [])
            if isinstance(edge, dict) and edge.get("fr_id")
        }
        total_fr_ids = len(fr_ids_from_ledger)

        # trace_ledger.lock.yaml の sha256 を計算する
        ledger_path = REPO_ROOT / "src/_meta/lock/trace_ledger.lock.yaml"
        ledger_sha = (
            sha256_of_text(ledger_path.read_text(encoding="utf-8"))
            if ledger_path.exists()
            else "sha256:unknown"
        )

        # src を走査して IMPL-ID マーカーを収集する
        impl_edges = _scan_src_impl_ids()

        # FR-ID ごとに対応する IMPL-ID の有無を集計する
        fr_ids_with_impl: set[str] = {e["fr_id"] for e in impl_edges if e["fr_id"] in fr_ids_from_ledger}
        fr_to_impl_ratio = (
            round(len(fr_ids_with_impl) / total_fr_ids, 4) if total_fr_ids > 0 else 0.0
        )

        # IMPL-ID のうち trace_ledger に対応 FR-ID が存在しないもの（孤立）を検出する
        dangling_impl = [
            e for e in impl_edges if e["fr_id"] not in fr_ids_from_ledger
        ]

        # 証拠なし IMPL-ID（proof_trace.lock.yaml 側で評価）
        impl_ids_found = len({e["impl_id"] for e in impl_edges})

        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        return {
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by tools/lock_yaml_generator/generate_coverage_oracle.py"
            ),
            "schema_version": "v1",
            "generated_at":   generated_at,
            "source": {
                "trace_ledger":        "trace_ledger.lock.yaml",
                "trace_ledger_sha256": ledger_sha,
            },
            "summary": {
                "total_fr_ids":      total_fr_ids,
                "fr_ids_with_impl":  len(fr_ids_with_impl),
                "fr_to_impl_ratio":  fr_to_impl_ratio,
                "impl_ids_found":    impl_ids_found,
                "dangling_impl_count": len(dangling_impl),
            },
            "impl_edges":    impl_edges,
            "dangling_impl": dangling_impl,
        }
