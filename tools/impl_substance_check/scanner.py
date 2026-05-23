"""tools/impl_substance_check/scanner.py

src/ を走査して軸別 LOC・readme_only 判定・facade LOC を集計する。
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:
    raise ImportError("PyYAML required")

# walk 中にスキップするディレクトリ名
_SKIP_DIRS: frozenset[str] = frozenset([
    "target", "node_modules", ".git", "__pycache__", "dist", "build",
    "vendor", ".cargo", ".cache", "lock", "samples", ".venv", "venv",
    "lake-packages", ".lake",
])

# 実装言語として認めるファイル拡張子
_LANG_EXTS: frozenset[str] = frozenset([
    ".rs", ".ts", ".tsx", ".js", ".jsx",
    ".go", ".py", ".cs", ".java", ".scala",
    ".lean", ".tla", ".dfy", ".c", ".h",
])

# 拡張子別の単行コメント prefix
_COMMENT_PREFIXES: dict[str, list[str]] = {
    ".rs": ["//"],
    ".ts": ["//"], ".tsx": ["//"], ".js": ["//"], ".jsx": ["//"],
    ".go": ["//"],
    ".py": ["#"],
    ".cs": ["//"],
    ".java": ["//"], ".scala": ["//"],
    ".lean": ["--"],
    ".tla": [r"\*"],
    ".dfy": ["//"],
    ".c": ["//"], ".h": ["//"],
}

# 軸 → src/ 内の相対ディレクトリ
_AXIS_DIRS: dict[str, str] = {
    "ops": "src/ops",
    "infra": "src/infra",
    "data": "src/data",
    "security": "src/security",
    "client": "src/client",
    "tier1": "src/tier1",
    "tier2": "src/tier2",
    "tier3": "src/tier3",
    "formal": "src/formal",
    "test": "src/test",
    "crosscutting": "src/_crosscutting",
    "meta": "src/_meta",
}

# README-only チェックを実施する実装軸（宣言的 YAML のみで逃げやすい軸）
_IMPL_CHECK_AXES: frozenset[str] = frozenset([
    "ops", "infra", "data", "security",
])

# client はサブディレクトリ単位でチェック
_CLIENT_IMPL_SUBDIRS: list[str] = [
    "browser_spa", "tauri", "full_native", "hlc_lib", "dotnet_fw", "thin_api",
]


def _count_loc(file_path: Path) -> int:
    """コメント行・空白行を除いた実質 LOC を返す（近似値）。"""
    ext = file_path.suffix.lower()
    if ext == ".c" and file_path.name.endswith(".bpf.c"):
        ext = ".c"

    comment_prefixes = _COMMENT_PREFIXES.get(ext, [])
    try:
        lines = file_path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return 0

    count = 0
    for line in lines:
        stripped = line.strip()
        if not stripped:
            continue
        if comment_prefixes and any(stripped.startswith(p) for p in comment_prefixes):
            continue
        count += 1
    return count


def _walk_lang_files(directory: Path):
    """ディレクトリ配下の言語ファイルを yield する（スキップディレクトリを除外）。"""
    for item in directory.iterdir():
        if item.is_dir():
            if item.name in _SKIP_DIRS:
                continue
            yield from _walk_lang_files(item)
        elif item.is_file() and item.suffix.lower() in _LANG_EXTS:
            yield item


def _has_lang_files(directory: Path) -> bool:
    """ディレクトリ配下に言語ファイルが 1 本以上あるか（スキップディレクトリ除外）。"""
    for _ in _walk_lang_files(directory):
        return True
    return False


def _axis_loc(repo_root: Path, rel_dir: str) -> int:
    """指定ディレクトリ配下の言語ファイル全件の LOC 合計（スキップディレクトリ除外）。"""
    d = repo_root / rel_dir
    if not d.is_dir():
        return 0
    total = 0
    for file in _walk_lang_files(d):
        total += _count_loc(file)
    return total


def _scan_readme_only(repo_root: Path) -> tuple[int, list[str]]:
    """readme_only なディレクトリ数とパスリストを返す。"""
    readme_only: list[str] = []

    # 実装軸ごとのトップレベルチェック
    for axis in _IMPL_CHECK_AXES:
        rel = _AXIS_DIRS[axis]
        d = repo_root / rel
        if d.is_dir() and not _has_lang_files(d):
            readme_only.append(rel)

    # client サブディレクトリ
    client_dir = repo_root / _AXIS_DIRS["client"]
    if client_dir.is_dir():
        for subdir_name in _CLIENT_IMPL_SUBDIRS:
            subdir = client_dir / subdir_name
            if subdir.is_dir() and not _has_lang_files(subdir):
                readme_only.append(f"src/client/{subdir_name}")

    return len(readme_only), sorted(readme_only)


def _load_facade_paths(facade_file: Path) -> tuple[list[str], int]:
    """facade_paths.yaml を読み込んで (paths, floor) を返す。"""
    if not facade_file.exists():
        return [], 200
    data = yaml.safe_load(facade_file.read_text(encoding="utf-8")) or {}
    paths = [str(p) for p in (data.get("paths") or [])]
    floor = int(data.get("floor", 200))
    return paths, floor


def _scan_facade_files(repo_root: Path, facade_file: Path | None) -> tuple[int, list[dict[str, Any]]]:
    """facade ファイルの LOC が floor を下回るものを検出する。"""
    if facade_file is None:
        return 0, []

    paths, floor = _load_facade_paths(facade_file)
    results: list[dict[str, Any]] = []
    below_floor = 0

    for rel_path in paths:
        abs_path = repo_root / rel_path
        if not abs_path.is_file():
            results.append({
                "path": rel_path,
                "loc": 0,
                "floor": floor,
                "meets_floor": False,
                "exists": False,
            })
            below_floor += 1
            continue
        loc = _count_loc(abs_path)
        meets = loc >= floor
        if not meets:
            below_floor += 1
        results.append({
            "path": rel_path,
            "loc": loc,
            "floor": floor,
            "meets_floor": meets,
        })

    return below_floor, results


def scan_all(repo_root: Path, facade_file: Path | None = None) -> dict[str, Any]:
    """全軸を走査して impl_substance.lock.yaml の内容を返す。"""
    summary: dict[str, int] = {}
    axes_detail: list[dict[str, Any]] = []

    for axis, rel_dir in _AXIS_DIRS.items():
        loc = _axis_loc(repo_root, rel_dir)
        summary[f"{axis}_loc"] = loc
        axes_detail.append({"axis": axis, "loc": loc, "dir": rel_dir})

    readme_only_count, readme_only_dirs = _scan_readme_only(repo_root)
    facade_below_floor, facade_files = _scan_facade_files(repo_root, facade_file)

    return {
        "summary": summary,
        "readme_only_count": readme_only_count,
        "readme_only_dirs": readme_only_dirs,
        "facade_below_floor_count": facade_below_floor,
        "facade_files": facade_files,
        "axes": axes_detail,
    }
