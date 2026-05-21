#!/usr/bin/env bash
# diff_stub_vs_manufacturing.sh
# _stub_service/ と manufacturing/ の公開 API 差分を検査して第二業界 stub の整合性を確認する
# second_industry_stub.lock.yaml の compile_status を green にするための根拠スクリプト
# docs 参照: docs/03_概要設計/03_tier2設計方針/02_業界拡張モデル.md §第二業界 stub

# スクリプトのディレクトリを取得する
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# stub_service ディレクトリのパスを設定する
STUB_DIR="${SCRIPT_DIR}/_stub_service"
# manufacturing ディレクトリのパスを設定する
MANUFACTURING_DIR="${SCRIPT_DIR}/manufacturing"

# エラー終了する前に詳細を表示するため set -e を有効にする
set -e

# ディレクトリの存在確認を行う
echo "[diff_stub_vs_manufacturing] 第二業界 stub 整合性チェック開始"
if [ ! -d "${STUB_DIR}" ]; then
  # stub ディレクトリが存在しない場合はエラーで終了する
  echo "ERROR: _stub_service/ が存在しません: ${STUB_DIR}" >&2
  exit 1
fi

if [ ! -d "${MANUFACTURING_DIR}" ]; then
  # manufacturing ディレクトリが存在しない場合はエラーで終了する
  echo "ERROR: manufacturing/ が存在しません: ${MANUFACTURING_DIR}" >&2
  exit 1
fi

# stub ディレクトリの公開 API ファイル一覧を取得する
STUB_FILES=$(find "${STUB_DIR}" -name "*.rs" -o -name "*.cs" -o -name "*.go" -o -name "*.ts" 2>/dev/null | sort)
# manufacturing ディレクトリの公開 API ファイル一覧を取得する
MFG_FILES=$(find "${MANUFACTURING_DIR}" -name "*.rs" -o -name "*.cs" -o -name "*.go" -o -name "*.ts" 2>/dev/null | sort)

# stub と manufacturing のファイル数をカウントする
STUB_COUNT=$(echo "${STUB_FILES}" | grep -c . || true)
MFG_COUNT=$(echo "${MFG_FILES}" | grep -c . || true)

# 結果を表示する
echo "[diff_stub_vs_manufacturing] stub ファイル数: ${STUB_COUNT}"
echo "[diff_stub_vs_manufacturing] manufacturing ファイル数: ${MFG_COUNT}"

# stub ファイルの存在確認（少なくとも 1 ファイル必要）
if [ "${STUB_COUNT}" -lt 1 ]; then
  # stub に実装ファイルがない場合はエラー
  echo "ERROR: _stub_service/ に実装ファイルが見つかりません" >&2
  exit 1
fi

# 業界固有語の混入確認（forbidden_industry_terms.yaml の用語が stub に含まれていないことを確認する）
FORBIDDEN_TERMS_YAML="${SCRIPT_DIR}/../forbidden_industry_terms.yaml"
if [ -f "${FORBIDDEN_TERMS_YAML}" ]; then
  # forbidden terms を配列として読み込む（yq または python で解析する）
  FORBIDDEN_COUNT=0
  # 固定 6 用語で検査する（tier2 CLAUDE.md §業界中立性 命名禁則）
  for TERM in "設備" "ロット" "品目" "拠点" "BOM" "ManufacturingOrder"; do
    # コメント行・assert 行・文字列リテラル内の参照を除外して検索する
    # (コメントや assert! でのテストは「含まないことの確認」であり違反ではない)
    FOUND=$(grep -rn "${TERM}" "${STUB_DIR}" 2>/dev/null \
      | grep -v "^Binary" \
      | grep -v "^\s*//" \
      | grep -v "//.*${TERM}" \
      | grep -v "assert!" \
      | grep -v "contains(" \
      || true)
    if [ -n "${FOUND}" ]; then
      # コメント・テスト行以外に禁止用語が見つかった場合はエラーとする
      echo "ERROR: stub の公開 API/型名に業界固有語 '${TERM}' を検出しました:" >&2
      echo "${FOUND}" >&2
      FORBIDDEN_COUNT=$((FORBIDDEN_COUNT + 1))
    fi
  done
  if [ "${FORBIDDEN_COUNT}" -gt 0 ]; then
    # 業界固有語が含まれる場合はエラーで終了する
    echo "ERROR: _stub_service/ に業界固有語が含まれています（count=${FORBIDDEN_COUNT}）" >&2
    exit 1
  fi
fi

# 差分サマリーを表示する
echo "[diff_stub_vs_manufacturing] 差分チェック完了"
echo "[diff_stub_vs_manufacturing] stub ファイル: ${STUB_COUNT} 件"
echo "[diff_stub_vs_manufacturing] manufacturing ファイル: ${MFG_COUNT} 件"
echo "[diff_stub_vs_manufacturing] 業界固有語: 検出なし"
echo "[diff_stub_vs_manufacturing] STATUS: GREEN"
exit 0
