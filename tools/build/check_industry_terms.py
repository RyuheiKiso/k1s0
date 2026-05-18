#!/usr/bin/env python3
# 業界固有語チェックスクリプト
# forbidden_industry_terms.yaml の語彙が src/tier2/rust/src/, src/tier2/go/ 等に含まれていないかチェックする

"""tools/build/check_industry_terms.py

tier2 業界固有語チェックツール。
src/tier2/forbidden_industry_terms.yaml の語彙を src/tier2/ 配下のソースから検索し、
pack/manufacturing 以外の場所で使用されている場合に CI fail する。
"""

# 標準ライブラリのインポート
import sys
from pathlib import Path
import re

try:
    # PyYAML のインポート
    import yaml
except ImportError:
    # PyYAML がない場合はスキップ
    print("PyYAML not found, skipping industry terms check")
    sys.exit(0)

# リポジトリルートのパス
REPO_ROOT = Path(__file__).resolve().parent.parent.parent

# forbidden terms ファイルのパス
FORBIDDEN_TERMS_PATH = REPO_ROOT / "src/tier2/forbidden_industry_terms.yaml"

# チェック対象の拡張子
TARGET_EXTENSIONS = {".rs", ".go", ".cs", ".ts", ".py"}

# 除外パターン (pack/manufacturing および node_modules は除外する)
EXCLUDE_PATTERNS = ["pack/manufacturing", "pack/_stub_service", "node_modules", ".pnpm", "target/"]


def load_terms() -> list[str]:
    """forbidden_industry_terms.yaml から語彙リストを読み込む。"""
    if not FORBIDDEN_TERMS_PATH.exists():
        # ファイルが存在しない場合は空リストを返す
        return []
    data = yaml.safe_load(FORBIDDEN_TERMS_PATH.read_text(encoding="utf-8")) or {}
    return [entry.get("term", "") for entry in data.get("terms", []) if entry.get("term")]


def should_exclude(path: Path) -> bool:
    """除外パターンに一致するパスかどうかを返す。"""
    path_str = str(path)
    return any(pattern in path_str for pattern in EXCLUDE_PATTERNS)


def check_files(terms: list[str]) -> list[str]:
    """tier2 ソースファイルを検索して違反を返す。"""
    violations = []
    # tier2 ディレクトリ配下を再帰的に検索する
    tier2_dir = REPO_ROOT / "src/tier2"
    for path in tier2_dir.rglob("*"):
        # ファイルのみ対象
        if not path.is_file():
            continue
        # 対象拡張子のみ処理する
        if path.suffix not in TARGET_EXTENSIONS:
            continue
        # 除外パターンに一致するパスはスキップする
        if should_exclude(path):
            continue
        try:
            content = path.read_text(encoding="utf-8")
        except Exception:
            # 読み込みエラーはスキップする
            continue
        # 各語彙を検索する
        for term in terms:
            if term and re.search(r'\b' + re.escape(term) + r'\b', content):
                violations.append(f"{path}: 業界固有語 '{term}' の使用 (pack/manufacturing に隔離してください)")
    return violations


def main() -> int:
    """メイン関数: 違反があれば 1 を返す。"""
    terms = load_terms()
    if not terms:
        print("No forbidden terms defined, skipping check")
        return 0
    violations = check_files(terms)
    for v in violations:
        print(f"FAIL: {v}", file=sys.stderr)
    if violations:
        return 1
    print("check_industry_terms: OK (no violations)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
