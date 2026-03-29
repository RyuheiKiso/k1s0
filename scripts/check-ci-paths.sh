#!/usr/bin/env bash
# scripts/check-ci-paths.sh
# L-10 監査対応: ci.yaml の paths-ignore と個別 CI の paths の整合性を確認するスクリプト。
#
# 目的:
#   ci.yaml の paths-ignore に登録されたパスと、
#   個別 CI ファイル（*-ci.yaml）の paths: トリガーが一致しているかを確認する。
#   不一致があると CI の二重実行または検出漏れが発生する。
#
# 使用方法:
#   bash scripts/check-ci-paths.sh
#
# 終了コード:
#   0 - 整合性OK（問題なし）
#   1 - 不整合あり（差分を出力）

set -euo pipefail

# スクリプトのあるディレクトリからリポジトリルートを特定する
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CI_YAML="${REPO_ROOT}/.github/workflows/ci.yaml"
WORKFLOWS_DIR="${REPO_ROOT}/.github/workflows"

# 終了コードのトラッキング
EXIT_CODE=0

echo "==================================================================="
echo "CI paths-ignore 整合性チェック"
echo "対象ファイル: .github/workflows/ci.yaml"
echo "==================================================================="
echo ""

# -------------------------------------------------------------------
# Step 1: ci.yaml の paths-ignore を抽出する
# -------------------------------------------------------------------
echo "[Step 1] ci.yaml から paths-ignore を抽出..."

# paths-ignore セクションを抽出してパスを取得する
CI_IGNORED_PATHS=$(
  awk '/^    paths-ignore:/{found=1; next} found && /^      - /{gsub(/^      - |'"'"'| *$/, ""); print} found && /^  [a-z]/{exit}' "${CI_YAML}" \
  | grep -v "^#" \
  | grep -v "^$" \
  | sort
)

echo "ci.yaml の paths-ignore:"
echo "${CI_IGNORED_PATHS}" | sed 's/^/  /'
echo ""

# -------------------------------------------------------------------
# Step 2: 個別 CI ファイルを検出して paths トリガーを抽出する
# -------------------------------------------------------------------
echo "[Step 2] 個別 CI ファイルのパストリガーを抽出..."

# 個別CIファイルを検索（アンダースコアで始まる共通ワークフローと ci.yaml 自体は除外）
INDIVIDUAL_CI_FILES=$(find "${WORKFLOWS_DIR}" -name "*-ci.yaml" ! -name "_*" | sort)

if [ -z "${INDIVIDUAL_CI_FILES}" ]; then
  echo "  [WARN] 個別 CI ファイルが見つかりません"
fi

# 個別 CI のパスをすべて収集する
INDIVIDUAL_PATHS=""
declare -A FILE_TO_PATHS

for ci_file in ${INDIVIDUAL_CI_FILES}; do
  ci_name=$(basename "${ci_file}")
  # pull_request の paths: セクションを抽出する
  paths=$(
    awk '/^  pull_request:/{found=1} found && /^    paths:/{in_paths=1; next} in_paths && /^      - /{gsub(/^      - |'"'"'| *$/, ""); print} in_paths && /^    [a-z]/{exit} in_paths && /^  [a-z]/{exit}' "${ci_file}" \
    | grep -v "^#" \
    | grep -v "^$" \
    | sort
  )
  if [ -n "${paths}" ]; then
    FILE_TO_PATHS["${ci_name}"]="${paths}"
    INDIVIDUAL_PATHS="${INDIVIDUAL_PATHS}
${paths}"
  fi
done

INDIVIDUAL_PATHS=$(echo "${INDIVIDUAL_PATHS}" | grep -v "^$" | sort)

echo "個別 CI のパストリガー（全ファイル合計）:"
echo "${INDIVIDUAL_PATHS}" | sed 's/^/  /'
echo ""

# -------------------------------------------------------------------
# Step 3: パスを正規化して比較する
# -------------------------------------------------------------------
echo "[Step 3] パスの整合性を確認..."

# ci.yaml の paths-ignore から /** サフィックスを除去して比較可能な形にする
normalize_path() {
  echo "$1" | sed 's/\*\*$//' | sed 's/\/$//'
}

# 個別 CI のパスから /** を除去する
NORMALIZED_INDIVIDUAL=$(echo "${INDIVIDUAL_PATHS}" | while read -r path; do normalize_path "$path"; done | sort | uniq)
NORMALIZED_IGNORED=$(echo "${CI_IGNORED_PATHS}" | while read -r path; do normalize_path "$path"; done | sort | uniq)

# -------------------------------------------------------------------
# Step 4: 「個別CIにあるが ci.yaml の paths-ignore にない」を検出する
# -------------------------------------------------------------------
echo "==================================================================="
echo "[チェック A] 個別CIのパスが ci.yaml の paths-ignore に未登録"
echo "(検出漏れ: 個別 CI がトリガーされてもメインCIも実行されてしまう)"
echo "==================================================================="

MISSING_IN_CI=""
while IFS= read -r indiv_path; do
  if ! echo "${NORMALIZED_IGNORED}" | grep -qF "${indiv_path}"; then
    MISSING_IN_CI="${MISSING_IN_CI}\n  MISSING: ${indiv_path}/** (ci.yaml paths-ignore に未登録)"
  fi
done <<< "${NORMALIZED_INDIVIDUAL}"

if [ -n "${MISSING_IN_CI}" ]; then
  echo -e "${MISSING_IN_CI}"
  echo ""
  echo "  [ACTION] ci.yaml の paths-ignore に上記パスを追加してください"
  EXIT_CODE=1
else
  echo "  OK: 全ての個別CIパスが ci.yaml の paths-ignore に登録されています"
fi
echo ""

# -------------------------------------------------------------------
# Step 5: 「ci.yaml の paths-ignore にあるが個別CIに対応なし」を検出する
# -------------------------------------------------------------------
echo "==================================================================="
echo "[チェック B] ci.yaml の paths-ignore に登録されているが個別 CI がない"
echo "(孤立エントリ: 個別 CI が削除されたにもかかわらず paths-ignore が残っている)"
echo "==================================================================="

ORPHAN_IN_CI=""
while IFS= read -r ignored_path; do
  if ! echo "${NORMALIZED_INDIVIDUAL}" | grep -qF "${ignored_path}"; then
    ORPHAN_IN_CI="${ORPHAN_IN_CI}\n  ORPHAN: ${ignored_path}/** (対応する個別 CI が見つからない)"
  fi
done <<< "${NORMALIZED_IGNORED}"

if [ -n "${ORPHAN_IN_CI}" ]; then
  echo -e "${ORPHAN_IN_CI}"
  echo ""
  echo "  [ACTION] ci.yaml の paths-ignore から上記のエントリを削除するか、個別 CI を追加してください"
  EXIT_CODE=1
else
  echo "  OK: ci.yaml の全 paths-ignore エントリに対応する個別 CI が存在します"
fi
echo ""

# -------------------------------------------------------------------
# Step 6: 各個別 CI ファイルのパス詳細を表示する
# -------------------------------------------------------------------
echo "==================================================================="
echo "[詳細] 個別 CI ファイルとカバーパスの対応"
echo "==================================================================="
for ci_file in ${INDIVIDUAL_CI_FILES}; do
  ci_name=$(basename "${ci_file}")
  if [ -n "${FILE_TO_PATHS[${ci_name}]+set}" ]; then
    echo "  ${ci_name}:"
    echo "${FILE_TO_PATHS[${ci_name}]}" | sed 's/^/    - /'
  else
    echo "  ${ci_name}: (paths: セクションなし)"
  fi
done
echo ""

# -------------------------------------------------------------------
# 結果まとめ
# -------------------------------------------------------------------
echo "==================================================================="
if [ "${EXIT_CODE}" -eq 0 ]; then
  echo "結果: OK - paths-ignore と個別 CI の整合性に問題はありません"
else
  echo "結果: NG - 不整合が検出されました。上記の ACTION を確認してください"
fi
echo "==================================================================="

exit "${EXIT_CODE}"
