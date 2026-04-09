#!/usr/bin/env bash
# ライブラリ言語パリティ検証スクリプト
# modules.yaml の library_parity セクションに基づき、
# 各カテゴリで宣言された全言語の実装ディレクトリが存在するか検証する
# 戻り値: 問題なし=0, 実装漏れ検出=1
set -euo pipefail

# Windows/CI 環境での Python UTF-8 出力を保証する（MED-004 監査対応）。
# PYTHONUTF8=1 は Python 3.7+ でUTF-8モードを強制し、日本語を含む出力の文字化けを防ぐ。
export PYTHONUTF8=1
export PYTHONIOENCODING=utf-8

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

# 言語別ディレクトリ名オーバーライドマッピング
# modules.yaml の library_parity 宣言名（抽象名）と実際のディレクトリ名が異なる場合に使用する。
# server カテゴリの Building Blocks ライブラリは言語ごとに命名規則が異なる:
#   - Rust: "bb-" prefix を付加（例: binding -> bb-binding）
#   - Go: 一部ハイフン区切りの別名（例: secretstore -> secret-store）
# core カテゴリの bb-ai-client は Dart でもハイフン形式のディレクトリ名を維持している。
LANG_DIR_OVERRIDES: dict[tuple[str, str], str] = {
    # server カテゴリ: Rust は "bb-" prefix が付く
    ('binding', 'rust'):    'bb-binding',
    ('pubsub', 'rust'):     'bb-pubsub',
    ('secretstore', 'rust'): 'bb-secretstore',
    ('statestore', 'rust'): 'bb-statestore',
    # server カテゴリ: Go の secretstore は "secret-store" というディレクトリ名
    ('secretstore', 'go'):  'secret-store',
    # core カテゴリ: Dart の bb-ai-client はハイフン形式のディレクトリ名を使用（snake_case 変換対象外）
    ('bb-ai-client', 'dart'): 'bb-ai-client',
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
    例: "dlq-client" -> "dlq_client"
    ただし LANG_DIR_OVERRIDES で明示指定されているものはオーバーライドが優先される。
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
            # LANG_DIR_OVERRIDES に明示的なマッピングがある場合はそれを優先する。
            # 次に Dart の snake_case 変換を適用し、それ以外は宣言名をそのまま使用する。
            if (lib, lang) in LANG_DIR_OVERRIDES:
                dir_name = LANG_DIR_OVERRIDES[(lib, lang)]
            elif lang == 'dart':
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
