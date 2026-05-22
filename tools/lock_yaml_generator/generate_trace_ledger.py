# k1s0-proof: PROOF-meta-prog-008 -> IMPL-meta-0008
# k1s0-impl: IMPL-meta-0008 realizes=FR-meta-008
"""tools/lock_yaml_generator/generate_trace_ledger.py

trace_ledger.lock.yaml 生成器。
docs 側の適合仕様（01_適合仕様/ + 03_クロスカッティング適合仕様/ 等）の
frontmatter trace.fr_ids を走査し FR-ID ↔ spec_id の対応関係 (edges) を記録する。

フェーズ別の対象ディレクトリ:
  R0: 骨格生成（全ディレクトリを走査・fr_ids は空なので ratio=0.0 でスタート）
  R1: 01_適合仕様/ の 21 件に trace.fr_ids を追加 → ratio >= 0.5
  R2: 残全ディレクトリ（02/03/04/05）に trace.fr_ids を追加 → ratio = 1.0
  R3: src AST annotation 追加（別ツール）
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

# FR-ID パターン（axis 部は文字列 enum）
_FR_ID_PATTERN: re.Pattern[str] = re.compile(r"^FR-[a-z][a-z0-9_]*-[0-9]{3}$")

# 走査対象 docs ディレクトリ（R0 から全ディレクトリを走査してベースラインを確立）
_SPEC_DIRS: list[str] = [
    "docs/04_詳細設計/01_適合仕様",
    "docs/04_詳細設計/02_強制機構",
    "docs/04_詳細設計/03_クロスカッティング適合仕様",
    "docs/04_詳細設計/04_運用UI開発者体験",
    "docs/04_詳細設計/05_lock_yaml体系",
]


def _parse_frontmatter(path: Path) -> dict[str, Any] | None:
    """Markdown ファイルから YAML frontmatter を解析して dict を返す。"""
    # frontmatter は --- で始まる必要がある
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return None
    # 終端 --- を検索する
    end = text.find("\n---\n", 4)
    if end < 0:
        return None
    # frontmatter テキストを抽出して YAML パース
    fm_text = text[4:end]
    try:
        fm = yaml.safe_load(fm_text)
        return fm if isinstance(fm, dict) else None
    except yaml.YAMLError:
        return None


class TraceLedgerGenerator(BaseGenerator):
    """trace_ledger.lock.yaml 生成器。

    docs 適合仕様の frontmatter trace.fr_ids と spec_id を対応付けた edges リストを生成し、
    FR-ID 被覆率（fr_to_spec_ratio）を summary に記録する。
    """

    # 出力ファイル名
    OUTPUT_NAME = "trace_ledger.lock.yaml"
    # 依存する入力 lock.yaml
    REQUIRED_INPUTS: list[str] = ["axis_registry.lock.yaml"]
    # jsonschema 検証用スキーマパス
    SCHEMA_PATH: Path | None = (
        REPO_ROOT / "tools/lock_yaml_generator/schemas/trace_ledger.schema.yaml"
    )
    # デフォルト出力先
    DEFAULT_OUTPUT_DIR = "src/_meta/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """axis_registry.lock.yaml を読み込む。"""
        # lock_dir から axis_registry を読み込む
        axis_registry = self.load_lock(lock_dir, "axis_registry.lock.yaml")
        return {"axis_registry": axis_registry, "lock_dir": lock_dir}

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """docs 適合仕様を走査して trace_ledger artifact を構築する。"""
        # axis_registry から有効な axis_name 集合を構築
        axis_registry: dict[str, Any] = inputs.get("axis_registry", {})
        valid_axes: set[str] = {
            ax.get("axis_name", "")
            for ax in (axis_registry.get("axes") or [])
            if isinstance(ax, dict) and ax.get("axis_name")
        }

        # axis_registry.lock.yaml の sha256 を計算（整合確認用）
        axis_registry_path = REPO_ROOT / "src/_meta/lock/axis_registry.lock.yaml"
        if axis_registry_path.exists():
            axis_registry_sha = sha256_of_text(axis_registry_path.read_text(encoding="utf-8"))
        else:
            axis_registry_sha = "sha256:unknown"

        # docs 適合仕様を走査して edges と被覆率を計算
        edges: list[dict[str, Any]] = []
        total_specs = 0
        specs_with_fr_ids: set[str] = set()

        for spec_dir_rel in _SPEC_DIRS:
            spec_dir = REPO_ROOT / spec_dir_rel
            # ディレクトリが存在しない場合はスキップ
            if not spec_dir.exists():
                continue
            # 各 .md ファイルを走査（README.md は index 扱いでスキップ）
            for md_path in sorted(spec_dir.glob("*.md")):
                if md_path.name == "README.md":
                    continue
                # frontmatter を解析
                fm = _parse_frontmatter(md_path)
                if fm is None:
                    continue
                # spec_id（docs frontmatter の id フィールド）を取得
                spec_id = fm.get("id")
                if not spec_id:
                    continue

                total_specs += 1
                # REPO_ROOT からの相対パス（spec_path フィールド）
                rel_path = str(md_path.relative_to(REPO_ROOT))

                # trace.fr_ids フィールドを取得（存在しない場合は空リスト）
                trace = fm.get("trace") or {}
                fr_ids = trace.get("fr_ids") or [] if isinstance(trace, dict) else []
                if not isinstance(fr_ids, list):
                    fr_ids = []

                # 各 FR-ID に対して edge を生成
                for fr_id in fr_ids:
                    if not isinstance(fr_id, str):
                        continue
                    # FR-ID パターン検証
                    if not _FR_ID_PATTERN.match(fr_id):
                        continue
                    # FR-ID の axis 部を抽出して axis_registry と整合確認
                    parts = fr_id.split("-")
                    if len(parts) >= 3 and valid_axes:
                        fr_axis = parts[1]
                        # axis が axis_registry に存在する場合は docs_lint、しない場合は axis_mismatch
                        verifier = "docs_lint" if fr_axis in valid_axes else "axis_mismatch"
                    else:
                        verifier = "docs_lint"

                    # edge エントリを追加
                    edges.append({
                        "fr_id":     fr_id,
                        "spec_id":   spec_id,
                        "spec_path": rel_path,
                        "kind":      "fr_to_spec",
                        "verifier":  verifier,
                    })
                    # fr_ids を持つ spec として記録
                    specs_with_fr_ids.add(spec_id)

        # FR-ID 被覆率を計算（0 除算を防ぐ）
        fr_to_spec_ratio = (
            round(len(specs_with_fr_ids) / total_specs, 4) if total_specs > 0 else 0.0
        )

        # 生成タイムスタンプ
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        return {
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by tools/lock_yaml_generator/generate_trace_ledger.py"
            ),
            "schema_version":    "v1",
            "generated_at":      generated_at,
            "axis_registry_ref": {
                "lock":   "axis_registry.lock.yaml",
                "sha256": axis_registry_sha,
            },
            "summary": {
                "total_specs":       total_specs,
                "specs_with_fr_ids": len(specs_with_fr_ids),
                "fr_to_spec_ratio":  fr_to_spec_ratio,
            },
            "edges": edges,
        }
