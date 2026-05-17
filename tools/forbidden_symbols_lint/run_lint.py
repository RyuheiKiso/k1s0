"""tools/forbidden_symbols_lint/run_lint.py

forbidden_export_symbols.lock.yaml を読み込み、tier3 配下の全ソース（ts/tsx/rs/go/cs）を
regex scan して banned symbol が含まれていたら報告し exit 1 する。

polyglot 対応: TypeScript / Rust / Go / C# を同一の pattern で検査する。

Usage:
    uv run python -m tools.forbidden_symbols_lint.run_lint [--dry-run]
    uv run python -m tools.forbidden_symbols_lint.run_lint [options]

Options:
    --dry-run           違反を報告するが exit 1 しない
    --allow-list PATH   改行区切りの symbol_id を除外する
    --tier3-root PATH   tier3 ディレクトリ（デフォルト: src/tier3）
    --lock-yaml PATH    lock.yaml パス（デフォルト: src/tier3/lock/forbidden_export_symbols.lock.yaml）
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:
    print("ERROR: pyyaml is not installed. Run: uv add pyyaml", file=sys.stderr)
    sys.exit(2)

# 除外するディレクトリ名（スキャン対象外）
_EXCLUDED_DIRS = frozenset({
    "node_modules", "target", "dist", ".git", "__pycache__",
    ".venv", ".pytest_cache", "generated", ".sqlx",
})

# スキャン対象拡張子
_SCANNED_EXTENSIONS = frozenset({".ts", ".tsx", ".rs", ".go", ".cs"})


def _load_lock_yaml(lock_path: Path) -> list[dict[str, Any]]:
    with lock_path.open(encoding="utf-8") as f:
        data = yaml.safe_load(f)
    if not isinstance(data, dict):
        print(f"ERROR: {lock_path} is not a valid YAML mapping", file=sys.stderr)
        sys.exit(2)
    return data.get("symbols", [])


def _collect_source_files(tier3_root: Path) -> list[Path]:
    result: list[Path] = []
    for path in tier3_root.rglob("*"):
        if any(part in _EXCLUDED_DIRS for part in path.parts):
            continue
        if path.is_file() and path.suffix in _SCANNED_EXTENSIONS:
            result.append(path)
    return sorted(result)


def _scan_file(
    path: Path,
    patterns: list[tuple[str, re.Pattern[str], str, str]],
) -> list[tuple[str, int, str, str, str]]:
    """ファイルを scan して違反を返す。

    Returns:
        List of (path_str, line_no, matched_text, symbol_id, reason)
    """
    violations: list[tuple[str, int, str, str, str]] = []
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return violations

    for lineno, line in enumerate(text.splitlines(), start=1):
        for symbol_id, pattern, _, reason in patterns:
            m = pattern.search(line)
            if m:
                violations.append((str(path), lineno, m.group(), symbol_id, reason))
    return violations


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Scan tier3 sources for banned export symbols"
    )
    parser.add_argument("--dry-run", action="store_true", help="Report but do not exit 1")
    parser.add_argument("--allow-list", metavar="PATH", help="File with symbol_id to ignore (one per line)")
    parser.add_argument(
        "--tier3-root",
        metavar="PATH",
        default="src/tier3",
        help="tier3 root directory (default: src/tier3)",
    )
    parser.add_argument(
        "--lock-yaml",
        metavar="PATH",
        default="src/tier3/lock/forbidden_export_symbols.lock.yaml",
        help="forbidden_export_symbols.lock.yaml path",
    )
    args = parser.parse_args(argv)

    lock_path = Path(args.lock_yaml)
    tier3_root = Path(args.tier3_root)

    if not lock_path.exists():
        print(f"ERROR: lock yaml not found: {lock_path}", file=sys.stderr)
        return 2
    if not tier3_root.exists():
        print(f"ERROR: tier3 root not found: {tier3_root}", file=sys.stderr)
        return 2

    allow_list: set[str] = set()
    if args.allow_list:
        allow_path = Path(args.allow_list)
        if allow_path.exists():
            allow_list = {line.strip() for line in allow_path.read_text().splitlines() if line.strip()}

    symbols = _load_lock_yaml(lock_path)

    # banned symbol のみ取り出し
    patterns: list[tuple[str, re.Pattern[str], str, str]] = []
    for sym in symbols:
        symbol_id = str(sym.get("symbol_id", ""))
        if sym.get("status") != "banned":
            continue
        if symbol_id in allow_list:
            continue
        raw_pattern = str(sym.get("pattern", ""))
        reason = str(sym.get("reason", ""))
        scope = str(sym.get("scope", ""))
        if not raw_pattern:
            continue
        try:
            compiled = re.compile(raw_pattern)
        except re.error as e:
            print(f"WARNING: invalid pattern for {symbol_id}: {e}", file=sys.stderr)
            continue
        patterns.append((symbol_id, compiled, scope, reason))

    if not patterns:
        print("INFO: no banned symbols to scan", file=sys.stdout)
        return 0

    source_files = _collect_source_files(tier3_root)
    all_violations: list[tuple[str, int, str, str, str]] = []

    for path in source_files:
        violations = _scan_file(path, patterns)
        all_violations.extend(violations)

    if not all_violations:
        print(
            f"OK: scanned {len(source_files)} files, 0 violations [{len(patterns)} banned patterns]",
            file=sys.stdout,
        )
        return 0

    for file_path, lineno, matched, symbol_id, reason in all_violations:
        print(f"{file_path}:{lineno}: BANNED[{symbol_id}] '{matched}' — {reason}", file=sys.stderr)

    print(
        f"\nFAIL: {len(all_violations)} violation(s) in {len(source_files)} files",
        file=sys.stderr,
    )

    return 0 if args.dry_run else 1


if __name__ == "__main__":
    sys.exit(main())
