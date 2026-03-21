#!/usr/bin/env bash
# ライブラリ言語パリティ検証スクリプト
# modules.yaml の library_parity セクションに基づき、
# 各カテゴリで宣言された全言語の実装ディレクトリが存在するか検証する
# 戻り値: 問題なし=0, 実装漏れ検出=1
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MODULES_YAML="$REPO_ROOT/modules.yaml"

# python3 と PyYAML が利用可能か確認する
if ! command -v python3 &>/dev/null; then
  echo "ERROR: python3 が見つかりません。python3 をインストールしてください。" >&2
  exit 1
fi

python3 - "$MODULES_YAML" "$REPO_ROOT" <<'PYEOF'
# modules.yaml の library_parity セクションを解析し、
# 各言語のライブラリディレクトリ存在を検証する Python スクリプト
import sys
import os

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML がインストールされていません。pip install pyyaml を実行してください。", file=sys.stderr)
    sys.exit(1)

modules_yaml_path = sys.argv[1]
repo_root = sys.argv[2]

# 言語ごとのライブラリ配置ベースパス
LANG_PATHS = {
    'rust': 'regions/system/library/rust',
    'go': 'regions/system/library/go',
    'ts': 'regions/system/library/typescript',
    'dart': 'regions/system/library/dart',
}

# カテゴリごとに実装が必要な言語の一覧
CATEGORY_LANGS = {
    'core': ['rust', 'go', 'ts', 'dart'],
    'server': ['rust', 'go'],
    'client': ['ts', 'dart'],
}

with open(modules_yaml_path) as f:
    data = yaml.safe_load(f)

parity = data.get('library_parity', {})
issues = []
checked = 0

def to_dart_dir_name(lib_name: str) -> str:
    """
    Dart パッケージのディレクトリ名は snake_case を使用する（Dart/Flutter 命名規約）。
    modules.yaml では kebab-case で宣言されているため、ハイフンをアンダースコアに変換する。
    例: "dlq-client" -> "dlq_client", "bb-ai-client" -> "bb_ai_client"
    """
    return lib_name.replace('-', '_')

for category, libs in parity.items():
    # lang_specific カテゴリは言語固有のため対象外
    if category == 'lang_specific':
        continue
    if not libs:
        continue

    expected_langs = CATEGORY_LANGS.get(category, [])
    for lib in libs:
        for lang in expected_langs:
            # Dart は snake_case、他言語（Go/Rust/TypeScript）は kebab-case のディレクトリ名を使用する。
            # modules.yaml の宣言名（kebab-case）を Dart 用に変換してパスを構築する。
            if lang == 'dart':
                dir_name = to_dart_dir_name(lib)
            else:
                dir_name = lib
            lib_dir = os.path.join(repo_root, LANG_PATHS[lang], dir_name)
            if not os.path.isdir(lib_dir):
                issues.append(
                    f"MISSING [{category}] {lib} — {lang}: {LANG_PATHS[lang]}/{dir_name}/"
                )
            checked += 1

if issues:
    print(f"\nパリティチェック失敗: {len(issues)} 件の実装漏れを検出（合計 {checked} 件を確認）\n")
    for issue in issues:
        print(f"  {issue}")
    sys.exit(1)
else:
    print(f"OK: 全 {checked} 件のライブラリ言語パリティを確認しました")
PYEOF
