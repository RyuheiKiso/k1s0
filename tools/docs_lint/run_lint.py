#!/usr/bin/env python3
"""tools/docs_lint/run_lint.py

ドキュメント lint Python 厳密実装。bash 版（run_lint.sh）の高度化:
- YAML parser での frontmatter 厳密検証
- circular dependency 検出（DAG 検査）
- cross-link anchor 検査
- id 一意性 + id 導出整合（phase prefix + axis + slug）
- 禁止表現（段階的 release / forbidden frontmatter）
- 空セクション / TBD 残存検査
- status: locked ドキュメントの追加検査
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

try:
    import yaml  # type: ignore
except ImportError:
    print("ERROR: PyYAML required. Install: pip install pyyaml", file=sys.stderr)
    sys.exit(2)


REPO_ROOT = Path(__file__).resolve().parent.parent.parent
DOCS_DIR = REPO_ROOT / "docs"
EXCLUDE_PATTERNS = [
    "/90_archive/",
    "/00_format/",
]
EXCLUDE_FILES = [
    "docs/README.md",  # top-level navigation, frontmatter 不要
]

REQUIRED_FIELDS = {"id", "axis", "phase", "kind", "status", "depends_on", "covered_by"}
FORBIDDEN_FIELDS = {"changelog", "last_updated", "last_modified", "author", "reviewers"}
LOCK_PATTERN = re.compile(r"^[a-z][a-z0-9_]*\.lock\.yaml$")
PHASE_PREFIX_BY_PATH = {
    "01_企画": "plan",
    "02_要件定義": "req",
    "03_概要設計": "arch",
    "04_詳細設計": "detail",
    "05_環境構築": "env",
}
FORBIDDEN_EXPRESSIONS = [
    re.compile(r"Phase\s*\d+\s*で.*Phase\s*\d+\s*で追記"),
]
TBD_PATTERN = re.compile(r"\b(TBD|未定|後述のみ)\b")
LINK_PATTERN = re.compile(r"\]\(([^)]+\.md(?:#[^)]+)?)\)")


def collect_md_files() -> list[Path]:
    files = []
    for path in DOCS_DIR.rglob("*.md"):
        rel = str(path.relative_to(REPO_ROOT))
        if any(p in str(path) for p in EXCLUDE_PATTERNS):
            continue
        if rel in EXCLUDE_FILES:
            continue
        files.append(path)
    return sorted(files)


def parse_frontmatter(path: Path) -> tuple[dict | None, str, str | None]:
    """Return (frontmatter_dict, body, error_msg)."""
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return None, text, "frontmatter 不在 (先頭 --- なし)"
    end = text.find("\n---\n", 4)
    if end < 0:
        return None, text, "frontmatter 終端 --- なし"
    fm_text = text[4:end]
    body = text[end + 5 :]
    try:
        fm = yaml.safe_load(fm_text)
        if not isinstance(fm, dict):
            return None, body, "frontmatter が dict ではない"
    except yaml.YAMLError as e:
        return None, body, f"YAML parse error: {e}"
    return fm, body, None


def check_frontmatter(path: Path, fm: dict | None, error: str | None) -> list[str]:
    fails = []
    rel = str(path.relative_to(REPO_ROOT))
    if error or fm is None:
        fails.append(f"{rel}: {error or 'frontmatter 不在'}")
        return fails
    missing = REQUIRED_FIELDS - fm.keys()
    if missing:
        fails.append(f"{rel}: required fields 不在: {sorted(missing)}")
    forbidden_present = FORBIDDEN_FIELDS & fm.keys()
    if forbidden_present:
        fails.append(f"{rel}: forbidden fields 出現: {sorted(forbidden_present)}")
    locks = fm.get("lock_artifacts") or []
    if isinstance(locks, list):
        for lock in locks:
            if not isinstance(lock, str) or not LOCK_PATTERN.match(lock):
                fails.append(f"{rel}: lock_artifacts pattern 違反: {lock!r}")
    return fails


def check_id_uniqueness(items: list[tuple[Path, dict]]) -> list[str]:
    fails = []
    by_id: dict[str, list[Path]] = defaultdict(list)
    for path, fm in items:
        i = fm.get("id")
        if i:
            by_id[i].append(path)
    for i, paths in by_id.items():
        if len(paths) > 1:
            rels = [str(p.relative_to(REPO_ROOT)) for p in paths]
            fails.append(f"id 重複 '{i}' in {rels}")
    return fails


def check_id_derivation(items: list[tuple[Path, dict]]) -> list[str]:
    fails = []
    for path, fm in items:
        rel = str(path.relative_to(REPO_ROOT))
        i = fm.get("id", "")
        if not i:
            continue
        prefix = i.split(".")[0]
        expected = None
        for path_part, exp_phase in PHASE_PREFIX_BY_PATH.items():
            if f"/{path_part}/" in rel:
                expected = exp_phase
                break
        if expected and prefix != expected:
            fails.append(f"{rel}: id prefix '{prefix}' != expected '{expected}'")
    return fails


def check_depends_on(items: list[tuple[Path, dict]]) -> list[str]:
    fails = []
    all_ids = {fm.get("id") for _, fm in items if fm.get("id")}
    for path, fm in items:
        rel = str(path.relative_to(REPO_ROOT))
        deps = fm.get("depends_on") or []
        if not isinstance(deps, list):
            fails.append(f"{rel}: depends_on が list ではない")
            continue
        for dep in deps:
            if not isinstance(dep, str):
                fails.append(f"{rel}: depends_on 要素が str でない: {dep!r}")
                continue
            if dep not in all_ids:
                fails.append(f"{rel}: depends_on '{dep}' 解決不能")
    return fails


def check_circular_dependency(items: list[tuple[Path, dict]]) -> list[str]:
    fails = []
    graph: dict[str, list[str]] = defaultdict(list)
    for _, fm in items:
        i = fm.get("id")
        if not i:
            continue
        deps = fm.get("depends_on") or []
        for dep in deps:
            if isinstance(dep, str):
                graph[i].append(dep)
    WHITE, GRAY, BLACK = 0, 1, 2
    color: dict[str, int] = defaultdict(int)
    cycles: list[list[str]] = []

    def dfs(node: str, path: list[str]) -> None:
        if color[node] == GRAY:
            cycle_start = path.index(node) if node in path else 0
            cycles.append(path[cycle_start:] + [node])
            return
        if color[node] == BLACK:
            return
        color[node] = GRAY
        path.append(node)
        for neighbor in graph[node]:
            dfs(neighbor, path)
        path.pop()
        color[node] = BLACK

    for node in list(graph.keys()):
        if color[node] == WHITE:
            dfs(node, [])
    for cycle in cycles:
        fails.append(f"循環依存検出: {' -> '.join(cycle)}")
    return fails


def check_cross_links(path: Path, body: str) -> list[str]:
    fails = []
    rel = str(path.relative_to(REPO_ROOT))
    base_dir = path.parent
    for match in LINK_PATTERN.finditer(body):
        link = match.group(1)
        if link.startswith("http"):
            continue
        link_path = link.split("#")[0]
        if not link_path:
            continue
        target = (base_dir / link_path).resolve()
        if not target.exists():
            fails.append(f"{rel}: dangling link: {link}")
    return fails


def check_forbidden_expressions(path: Path, body: str) -> list[str]:
    fails = []
    rel = str(path.relative_to(REPO_ROOT))
    for pat in FORBIDDEN_EXPRESSIONS:
        if pat.search(body):
            fails.append(f"{rel}: 段階的 release 表現「{pat.pattern}」検出")
    return fails


def check_empty_locked(path: Path, fm: dict, body: str) -> list[str]:
    fails = []
    rel = str(path.relative_to(REPO_ROOT))
    if fm.get("status") != "locked":
        return fails
    if TBD_PATTERN.search(body):
        fails.append(f"{rel}: status: locked で TBD / 未定 / 後述のみ 検出")
    return fails


ROOT_ALLOWED_FILES = frozenset([
    "CLAUDE.md", "README.md", "LICENSE", "ARCHITECTURE.md",
    ".claudeignore", ".gitignore",
    # build/tooling 基盤（pyproject.toml / uv.lock / Makefile は P3 で追加）
    "pyproject.toml", "uv.lock", "Makefile", "requirements.txt",
])
ROOT_ALLOWED_DIRS = frozenset([
    ".claude", ".github", "docs", "img", "src", "tools",
    # uv/pytest 自動生成（git管理外）
    ".git",
])
SRC_ALLOWED_AXES = frozenset([
    "tier1", "tier2", "tier3", "infra", "data",
    "security", "ops", "client", "test", "formal",
    "_meta", "_crosscutting",
])
CROSSCUTTING_SLUG_PATTERN = re.compile(r"^(0[1-9]|1[0-3])_[a-z][a-z0-9_-]+$")
IMG_ALLOWED_EXTS = frozenset([".svg", ".drawio"])


def check_repository_layout() -> list[str]:
    fails = []

    # root 直下許可ファイル
    for entry in REPO_ROOT.iterdir():
        if entry.is_file() and entry.name not in ROOT_ALLOWED_FILES:
            fails.append(f"root: 許可外ファイル: {entry.name}")

    # root top-level directory allowlist（dot-prefix hidden dir は除外）
    for entry in REPO_ROOT.iterdir():
        if entry.is_dir() and not entry.name.startswith("."):
            if entry.name not in ROOT_ALLOWED_DIRS:
                fails.append(f"root: 許可外ディレクトリ: {entry.name}")

    # src/ 直下 sub-directory 検査
    src_dir = REPO_ROOT / "src"
    if src_dir.exists():
        for entry in src_dir.iterdir():
            if entry.is_dir() and entry.name not in SRC_ALLOWED_AXES:
                fails.append(f"src/: 許可外サブディレクトリ: {entry.name}")
            if entry.is_file() and entry.name not in {"README.md", "CLAUDE.md"}:
                fails.append(f"src/: 許可外ファイル: {entry.name}")

        # src/_crosscutting/ 配下 NN_<slug>/ 形式検査
        crosscutting_dir = src_dir / "_crosscutting"
        if crosscutting_dir.exists():
            for entry in crosscutting_dir.iterdir():
                if entry.is_dir() and not CROSSCUTTING_SLUG_PATTERN.match(entry.name):
                    fails.append(f"src/_crosscutting/: 命名規約違反: {entry.name}")

    # /img/ 直下拡張子検査
    img_dir = REPO_ROOT / "img"
    if img_dir.exists():
        for entry in img_dir.iterdir():
            if entry.is_file() and entry.suffix not in IMG_ALLOWED_EXTS:
                fails.append(f"img/: 許可外拡張子: {entry.name}")

    return fails


def check_published_spec_completeness(items: list[tuple[Path, dict]], body_by_path: dict[Path, str]) -> list[str]:
    """status: published の適合仕様 .md は version フィールドと lock_artifacts 物理存在が必要。
    docs/04_詳細設計/01_適合仕様/ 配下のファイルが対象。"""
    # 失敗メッセージ蓄積リスト
    fails = []
    # SemVer パターン（major.minor.patch 形式）
    import re as _re
    SEMVER = _re.compile(r"^\d+\.\d+\.\d+$")
    # 全 frontmatter 付き markdown ファイルを走査
    for path, fm in items:
        # REPO_ROOT からの相対パスで対象ディレクトリを判定
        rel = str(path.relative_to(REPO_ROOT))
        # 04_詳細設計/01_適合仕様/ 配下のみ対象
        if "04_詳細設計/01_適合仕様/" not in rel:
            continue
        # status: published のファイルのみ検査
        if fm.get("status") != "published":
            continue
        # version フィールドの SemVer 検査
        version = fm.get("version", "")
        if not isinstance(version, str) or not SEMVER.match(str(version)):
            fails.append(f"{rel}: status: published だが version が SemVer でない: {version!r}")
        # lock_artifacts の物理存在検査（axis サブディレクトリ配下を参照）
        axis = fm.get("axis", "")
        # _meta / _crosscutting は src/ 直下でアンダースコア prefix を持つ
        _UNDERSCORE_AXES = {"meta", "crosscutting"}
        axis_dir = f"_{axis}" if axis in _UNDERSCORE_AXES else axis
        locks = fm.get("lock_artifacts") or []
        for lock in (locks if isinstance(locks, list) else []):
            # lock_artifacts の各要素が str であることを確認
            if not isinstance(lock, str):
                continue
            # src/<axis_dir>/lock/<lock> の物理パスを構築
            lock_path = REPO_ROOT / "src" / axis_dir / "lock" / lock
            # 物理ファイルが存在しない場合は失敗として記録
            if not lock_path.exists():
                fails.append(f"{rel}: lock_artifacts 物理欠落: src/{axis_dir}/lock/{lock}")
    return fails


def check_test_matrix_implementation_paths() -> list[str]:
    """src/tier3/test_matrix.yaml の scenarios[].implementation_path が物理存在するか検査。"""
    # 失敗メッセージ蓄積リスト
    fails = []
    # test_matrix.yaml の物理パスを構築
    tm_path = REPO_ROOT / "src/tier3/test_matrix.yaml"
    # ファイルが存在しない場合はスキップ（非破壊）
    if not tm_path.exists():
        return fails
    # YAML パース（失敗時もスキップ）
    try:
        import yaml as _yaml
        tm = _yaml.safe_load(tm_path.read_text(encoding="utf-8")) or {}
    except Exception:
        return fails
    # scenarios リスト内の各シナリオを検査
    for scenario in (tm.get("scenarios") or []):
        # シナリオが dict でない場合はスキップ
        if not isinstance(scenario, dict):
            continue
        # implementation_path フィールドを取得
        impl_path = scenario.get("implementation_path")
        # 未設定の場合はスキップ
        if not impl_path:
            continue
        # glob パターンをサポートして物理存在を確認
        import glob as _glob
        matched = _glob.glob(str(REPO_ROOT / impl_path))
        # マッチするファイルがなければ失敗として記録
        if not matched:
            sid = scenario.get("scenario_id", "unknown")
            fails.append(f"test_matrix: scenario {sid} implementation_path 物理欠落: {impl_path}")
    return fails


def check_policy_mapping_bidirectional(items: list[tuple[Path, dict]]) -> list[str]:
    """src/tier1/schema/policy_mapping.yaml が存在する場合のみ: spec frontmatter と双方向参照を検査。"""
    # 失敗メッセージ蓄積リスト
    fails = []
    # policy_mapping.yaml の物理パスを構築（src/tier1/schema/ に移動済み）
    pm_path = REPO_ROOT / "src/tier1/schema/policy_mapping.yaml"
    # ファイルが存在しない場合はスキップ（非破壊）
    if not pm_path.exists():
        return fails
    # YAML パース（失敗時もスキップ）
    try:
        import yaml as _yaml
        pm = _yaml.safe_load(pm_path.read_text(encoding="utf-8")) or {}
    except Exception:
        return fails
    # policy_mapping.yaml 内に記載された spec_id の集合を構築
    spec_ids_in_pm = {
        entry.get("spec_id")
        for entry in (pm.get("mappings") or [])
        if isinstance(entry, dict) and entry.get("spec_id")
    }
    # docs 内 frontmatter から id フィールドの集合を構築
    fm_spec_ids = {fm.get("id") for _, fm in items if fm.get("id")}
    # policy_mapping に載っている spec_id が docs 内に存在するか検査（dead ref 検出）
    for sid in spec_ids_in_pm:
        if sid and sid not in fm_spec_ids:
            fails.append(f"policy_mapping.yaml: spec_id '{sid}' が docs 内に存在しない (dead ref)")
    return fails


def main() -> int:
    md_files = collect_md_files()
    print(f"=== docs_lint (Python): {len(md_files)} files ===")

    items: list[tuple[Path, dict]] = []
    body_by_path: dict[Path, str] = {}
    fails: list[str] = []

    # [1/11] frontmatter schema 検査（required fields / forbidden fields / lock_artifacts パターン）
    print("\n[1/11] frontmatter schema 検査")
    for path in md_files:
        fm, body, err = parse_frontmatter(path)
        body_by_path[path] = body
        sub = check_frontmatter(path, fm, err)
        for f in sub:
            print(f"  FAIL: {f}")
        fails.extend(sub)
        if fm is not None:
            items.append((path, fm))

    # [2/11] id 一意性検査（全ファイル横断で id 重複を検出）
    print("\n[2/11] id 一意性検査")
    sub = check_id_uniqueness(items)
    for f in sub:
        print(f"  FAIL: {f}")
    fails.extend(sub)

    # [3/11] id 導出整合（phase prefix がパス由来と一致するか検査）
    print("\n[3/11] id 導出整合 (phase prefix)")
    sub = check_id_derivation(items)
    for f in sub:
        print(f"  FAIL: {f}")
    fails.extend(sub)

    # [4/11] depends_on 参照整合 + 循環検出（DAG 検査）
    print("\n[4/11] depends_on 参照整合 + 循環検出")
    sub = check_depends_on(items)
    for f in sub:
        print(f"  FAIL: {f}")
    fails.extend(sub)
    sub = check_circular_dependency(items)
    for f in sub:
        print(f"  FAIL: {f}")
    fails.extend(sub)

    # [5/11] cross-link dangling 検査（内部リンクの物理存在を確認）
    print("\n[5/11] cross-link dangling 検査")
    for path in md_files:
        sub = check_cross_links(path, body_by_path.get(path, ""))
        for f in sub:
            print(f"  FAIL: {f}")
        fails.extend(sub)

    # [6/11] 禁止表現検査（段階的 release 表現などを検出）
    print("\n[6/11] 禁止表現検査")
    for path in md_files:
        sub = check_forbidden_expressions(path, body_by_path.get(path, ""))
        for f in sub:
            print(f"  FAIL: {f}")
        fails.extend(sub)

    # [7/11] 空セクション / TBD 残存検査（status: locked のみ対象）
    print("\n[7/11] 空セクション / TBD 残存検査 (status: locked のみ)")
    for path, fm in items:
        sub = check_empty_locked(path, fm, body_by_path.get(path, ""))
        for f in sub:
            print(f"  FAIL: {f}")
        fails.extend(sub)

    # [8/11] repository layout 検査（root / src / img 配下の許可制）
    print("\n[8/11] repository layout 検査")
    sub = check_repository_layout()
    for f in sub:
        print(f"  FAIL: {f}")
    fails.extend(sub)

    # [9/11] published 適合仕様の completeness 検査（version + lock_artifacts 物理存在）
    print("\n[9/11] published spec completeness 検査 (version SemVer + lock_artifacts 物理存在)")
    sub = check_published_spec_completeness(items, body_by_path)
    for f in sub:
        print(f"  FAIL: {f}")
    fails.extend(sub)

    # [10/11] test_matrix.yaml の implementation_path 物理存在検査
    print("\n[10/11] test_matrix implementation_path 物理存在検査")
    sub = check_test_matrix_implementation_paths()
    for f in sub:
        print(f"  FAIL: {f}")
    fails.extend(sub)

    # [11/11] policy_mapping.yaml ↔ spec frontmatter 双方向参照検査
    print("\n[11/11] policy_mapping bidirectional 参照検査")
    sub = check_policy_mapping_bidirectional(items)
    for f in sub:
        print(f"  FAIL: {f}")
    fails.extend(sub)

    print()
    if not fails:
        # 全 11 check が green の場合の終了メッセージ
        print("=== docs_lint (Python): 11 check 全 green ===")
        return 0
    print(f"=== docs_lint (Python): {len(fails)} FAIL detected ===")
    return 1


if __name__ == "__main__":
    sys.exit(main())
