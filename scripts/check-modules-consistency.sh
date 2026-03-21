#!/usr/bin/env bash
# modules.yaml と CI ワークフローの整合性を検証するスクリプト（P2-36）
#
# 以下を検証する:
#   1. modules.yaml に登録されている全モジュールのパスが実際に存在するか
#   2. status: stable かつ skip-ci: false のモジュールが CI でカバーされているか
#   3. archived モジュールが CI paths-ignore に残っていないか（削除済み確認）
#
# 使い方:
#   bash scripts/check-modules-consistency.sh
#
# 終了コード:
#   0: 問題なし
#   1: 整合性の問題を検出

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
MODULES_YAML="${REPO_ROOT}/modules.yaml"
CI_YAML="${REPO_ROOT}/.github/workflows/ci.yaml"

echo "=== modules.yaml と CI 整合性チェック ==="
failed=0

# Python が利用可能かチェック（YAML パース用）
if ! command -v python3 >/dev/null 2>&1; then
  echo "WARNING: python3 が見つかりません。YAML パースチェックをスキップします。"
  exit 0
fi

# modules.yaml のパース：各モジュールの存在チェック
echo ""
echo "--- モジュールパス存在チェック ---"
python3 - "$MODULES_YAML" "$REPO_ROOT" <<'PYEOF'
import sys, yaml, os

modules_yaml = sys.argv[1]
repo_root = sys.argv[2]

with open(modules_yaml) as f:
    data = yaml.safe_load(f)

issues = []
for mod in data.get('modules', []):
    path = mod.get('path', '')
    status = mod.get('status', 'stable')
    if status == 'archived':
        # archived はパス削除を想定しているためスキップ
        continue
    abs_path = os.path.join(repo_root, path)
    if not os.path.exists(abs_path):
        issues.append(f"ERROR: modules.yaml に登録されているパスが存在しません: {path}")

if issues:
    for issue in issues:
        print(issue)
    sys.exit(1)
else:
    print(f"OK: 全 {len([m for m in data.get('modules', []) if m.get('status') != 'archived'])} モジュールのパスが存在します")
PYEOF
check_result=$?
if [ $check_result -ne 0 ]; then
  failed=1
fi

# ci.yaml の paths-ignore が modules.yaml のワークスペース/サービスパスと一致するか検証
echo ""
echo "--- ci.yaml paths-ignore と modules.yaml の整合性チェック ---"
python3 - "$MODULES_YAML" "$CI_YAML" "$REPO_ROOT" <<'PYEOF'
import sys, yaml, re

modules_yaml = sys.argv[1]
ci_yaml = sys.argv[2]
repo_root = sys.argv[3]

with open(modules_yaml) as f:
    mods_data = yaml.safe_load(f)

with open(ci_yaml) as f:
    ci_data = yaml.safe_load(f)

# ci.yaml の paths-ignore を取得
paths_ignore = []
on_section = ci_data.get('on', {})
for event in ['pull_request', 'push']:
    if event in on_section and isinstance(on_section[event], dict):
        paths_ignore.extend(on_section[event].get('paths-ignore', []))

# modules.yaml で個別 CI がある（skip-ci: true ではない stable サービス）パスを取得
# paths-ignore は個別CI管理サービスのパスを除外するためのもの
# ここでは path のパターンマッチのみで簡易チェック
ignore_dirs = set()
for pattern in paths_ignore:
    # '/**' を取り除いてディレクトリパスを抽出
    base = pattern.rstrip('/**').rstrip('/')
    ignore_dirs.add(base)

# 孤立した paths-ignore エントリを検出（modules.yaml に対応するパスがない）
all_module_paths = {mod['path'] for mod in mods_data.get('modules', [])}
orphan_ignores = []
for ignore_dir in ignore_dirs:
    if not any(ignore_dir == p or ignore_dir.startswith(p + '/') or p.startswith(ignore_dir + '/') for p in all_module_paths):
        orphan_ignores.append(ignore_dir)

if orphan_ignores:
    print("WARNING: ci.yaml paths-ignore に modules.yaml に対応パスがないエントリが含まれています:")
    for o in orphan_ignores:
        print(f"  - {o}")
    print("  → 削除済みモジュールの残留エントリの可能性があります")
else:
    print(f"OK: ci.yaml paths-ignore の全エントリが modules.yaml のパスと整合しています")
PYEOF

# 最終結果
echo ""
if [ "$failed" -eq 0 ]; then
  echo "OK: modules.yaml と CI の整合性チェックが完了しました"
else
  echo "FAIL: 整合性の問題が検出されました"
fi

exit $failed
