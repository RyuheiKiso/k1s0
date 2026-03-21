#!/usr/bin/env bash
# standalone Rust サーバー（regions/system ワークスペース外）のパスを出力するスクリプト
# modules.yaml の modules セクションから動的に取得することで
# justfile の手動リストとの二重管理を解消する
# 出力: 1行1パス形式（bash の mapfile コマンドで配列に取り込み可能）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MODULES_YAML="$REPO_ROOT/modules.yaml"

# python3 と PyYAML が利用可能か確認する
if ! command -v python3 &>/dev/null; then
  echo "ERROR: python3 が見つかりません。" >&2
  exit 1
fi

python3 - "$MODULES_YAML" <<'PYEOF'
# modules.yaml から standalone Rust サーバーのパスを抽出する Python スクリプト
# standalone: regions/system ワークスペース外のサーバー（business/service tier）
import sys

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML がインストールされていません。pip install pyyaml を実行してください。", file=sys.stderr)
    sys.exit(1)

modules_yaml_path = sys.argv[1]

with open(modules_yaml_path) as f:
    data = yaml.safe_load(f)

modules = data.get('modules', [])

for module in modules:
    path = module.get('path', '')
    lang = module.get('lang', '')
    module_type = module.get('type', '')
    workspace = module.get('workspace', '')
    status = module.get('status', 'active')

    # 条件: Rust サーバー かつ regions/system ワークスペース外 かつ アーカイブ済みでない
    if (lang == 'rust'
            and module_type == 'server'
            and workspace != 'regions/system'
            and status != 'archived'):
        print(path)
PYEOF
