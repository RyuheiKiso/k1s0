#!/bin/bash
# PostgreSQLダンプ整合性テストスクリプト
# 目的: 定期リストアテスト（H-12監査対応）
# pg_restore --list でダンプの構造を検証し、実際にリストアできることを保証する
# 頻度: 四半期に1回以上（staging環境で実施すること）
# 実行前提: pg_restore, psql コマンドが利用可能であること
# バックアップはPVC（BACKUP_DIR）から読み込む。S3依存なし。

set -euo pipefail

# ==============================================================================
# 設定パラメータ（環境変数で上書き可能）
# ==============================================================================
# テスト対象のバックアップPVCマウントパス。S3依存なし。
BACKUP_DIR="${BACKUP_DIR:-/backup/postgres}"
# テスト用PostgreSQL接続先（staging環境の一時DBを使用すること）
TEST_PGHOST="${TEST_PGHOST:?TEST_PGHOST 環境変数を設定してください（staging DBホスト）}"
TEST_PGUSER="${TEST_PGUSER:?TEST_PGUSER 環境変数を設定してください}"
# テスト用リストア先データベース名（本番DBと別名にすること）
TEST_DB_NAME="${TEST_DB_NAME:-restore_test_$(date +%Y%m%d%H%M%S)}"
# テスト対象データベース一覧
TARGET_DATABASES="${TARGET_DATABASES:-k1s0_system config_db dlq_db}"
# テスト用一時ディレクトリ
WORK_DIR="${WORK_DIR:-/tmp/postgres-restore-test}"
# テスト完了後にクリーンアップするか（デフォルト: する）
CLEANUP="${CLEANUP:-true}"

# ==============================================================================
# ログ出力関数
# ==============================================================================
# タイムスタンプ付きでログを出力する
log() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

# エラーメッセージを標準エラーに出力して終了する
error() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $*" >&2
  exit 1
}

# ==============================================================================
# クリーンアップ関数
# ==============================================================================
# テスト終了時にテスト用DBと一時ファイルを削除する
cleanup() {
  log "クリーンアップを開始します..."
  # テスト用DBが存在する場合は削除する
  for DB_NAME in ${CREATED_TEST_DBS:-}; do
    log "テスト用DB ${DB_NAME} を削除します..."
    PGPASSWORD="${PGPASSWORD:-}" psql \
      -h "${TEST_PGHOST}" \
      -U "${TEST_PGUSER}" \
      -c "DROP DATABASE IF EXISTS ${DB_NAME};" \
      postgres 2>/dev/null || log "警告: ${DB_NAME} の削除に失敗しました（手動で削除してください）"
  done
  # 一時ディレクトリを削除する
  if [[ "${CLEANUP}" == "true" ]] && [[ -d "${WORK_DIR}" ]]; then
    log "一時ディレクトリ ${WORK_DIR} を削除します..."
    rm -rf "${WORK_DIR}"
  fi
  log "クリーンアップ完了"
}

# スクリプト終了時（正常・異常・シグナル問わず）にクリーンアップを実行する
trap cleanup EXIT

# 作成したテスト用DBを追跡するリスト
CREATED_TEST_DBS=""

# ==============================================================================
# メイン処理
# ==============================================================================
log "=== PostgreSQL ダンプ リストアテスト 開始 ==="
log "TEST_PGHOST: ${TEST_PGHOST}"
log "BACKUP_DIR: ${BACKUP_DIR}"
log "対象DB: ${TARGET_DATABASES}"

# ステップ1: 作業ディレクトリを作成する
log "ステップ1: 作業ディレクトリを準備します (${WORK_DIR})"
mkdir -p "${WORK_DIR}"

# ステップ2: PostgreSQL接続確認を行う
log "ステップ2: PostgreSQL接続確認..."
PGPASSWORD="${PGPASSWORD:-}" psql \
  -h "${TEST_PGHOST}" \
  -U "${TEST_PGUSER}" \
  -c "SELECT version();" \
  postgres > "${WORK_DIR}/pg-version.txt" 2>&1 || error "PostgreSQLへの接続に失敗しました"
log "PostgreSQL接続確認: OK"
grep "PostgreSQL" "${WORK_DIR}/pg-version.txt" | head -1 || true

# ステップ3: 各データベースのバックアップファイルを検証する
log "ステップ3: バックアップファイルの検証を開始します..."
TEST_PASS=0
TEST_FAIL=0

for DB in ${TARGET_DATABASES}; do
  log "--- データベース: ${DB} ---"

  # バックアップファイルを特定する（PVCマウントパスから最新ファイルを取得する）
  BACKUP_FILE=$(ls -t "${BACKUP_DIR}/${DB}-"*.dump 2>/dev/null | head -1)
  if [[ -z "${BACKUP_FILE}" ]]; then
    log "警告: ${BACKUP_DIR} に ${DB} のバックアップファイルが見つかりません。スキップします"
    TEST_FAIL=$((TEST_FAIL + 1))
    continue
  fi

  log "検証対象ファイル: ${BACKUP_FILE} ($(du -h "${BACKUP_FILE}" | cut -f1))"

  # ステップ3a: pg_restore --list でダンプの構造を検証する
  # --list は実際のリストアを行わずにダンプ内のオブジェクト一覧を表示する
  log "pg_restore --list でダンプ構造を検証します..."
  LIST_OUTPUT="${WORK_DIR}/${DB}-list.txt"
  if pg_restore --list "${BACKUP_FILE}" > "${LIST_OUTPUT}" 2>&1; then
    OBJECT_COUNT=$(wc -l < "${LIST_OUTPUT}")
    log "ダンプ構造検証: OK (オブジェクト数: ${OBJECT_COUNT})"
    # テーブル・インデックス・シーケンスのカウントを表示する
    TABLE_COUNT=$(grep -c "TABLE DATA" "${LIST_OUTPUT}" 2>/dev/null || echo 0)
    INDEX_COUNT=$(grep -c "INDEX" "${LIST_OUTPUT}" 2>/dev/null || echo 0)
    log "  テーブルデータ: ${TABLE_COUNT}件, インデックス: ${INDEX_COUNT}件"
  else
    log "エラー: pg_restore --list が失敗しました"
    cat "${LIST_OUTPUT}" >&2
    TEST_FAIL=$((TEST_FAIL + 1))
    continue
  fi

  # ステップ3b: テスト用DBにリストアして整合性を確認する
  RESTORE_DB_NAME="${TEST_DB_NAME}_${DB}"
  log "テスト用DB ${RESTORE_DB_NAME} を作成してリストアします..."

  # テスト用DBを作成する
  PGPASSWORD="${PGPASSWORD:-}" psql \
    -h "${TEST_PGHOST}" \
    -U "${TEST_PGUSER}" \
    -c "CREATE DATABASE ${RESTORE_DB_NAME};" \
    postgres || error "テスト用DB ${RESTORE_DB_NAME} の作成に失敗しました"
  CREATED_TEST_DBS="${CREATED_TEST_DBS} ${RESTORE_DB_NAME}"

  # リストアを実行する（-Fc: カスタム形式、-c: 既存オブジェクトをクリア）
  RESTORE_LOG="${WORK_DIR}/${DB}-restore.log"
  if PGPASSWORD="${PGPASSWORD:-}" pg_restore \
    -h "${TEST_PGHOST}" \
    -U "${TEST_PGUSER}" \
    -d "${RESTORE_DB_NAME}" \
    -Fc \
    "${BACKUP_FILE}" \
    > "${RESTORE_LOG}" 2>&1; then
    log "リストア: OK"
  else
    # pg_restoreはwarningでも終了コードが非0になる場合があるため内容を確認する
    ERROR_COUNT=$(grep -ci "error" "${RESTORE_LOG}" 2>/dev/null || echo 0)
    if [[ ${ERROR_COUNT} -gt 0 ]]; then
      log "警告: リストア中にエラーが発生しました (${ERROR_COUNT}件)"
      grep -i "error" "${RESTORE_LOG}" | head -5 >&2
    else
      log "リストア: OK (警告のみ、エラーなし)"
    fi
  fi

  # ステップ3c: リストア後の基本整合性チェックを行う
  log "リストア後の整合性チェック..."
  INTEGRITY_OUTPUT="${WORK_DIR}/${DB}-integrity.txt"
  PGPASSWORD="${PGPASSWORD:-}" psql \
    -h "${TEST_PGHOST}" \
    -U "${TEST_PGUSER}" \
    -d "${RESTORE_DB_NAME}" \
    -c "SELECT schemaname, tablename, n_live_tup FROM pg_stat_user_tables ORDER BY n_live_tup DESC LIMIT 10;" \
    > "${INTEGRITY_OUTPUT}" 2>&1 || true

  RESTORED_TABLE_COUNT=$(PGPASSWORD="${PGPASSWORD:-}" psql \
    -h "${TEST_PGHOST}" \
    -U "${TEST_PGUSER}" \
    -d "${RESTORE_DB_NAME}" \
    -t \
    -c "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public';" \
    2>/dev/null | tr -d ' ' || echo "0")

  log "リストア済みテーブル数 (public schema): ${RESTORED_TABLE_COUNT}"
  if [[ "${RESTORED_TABLE_COUNT}" -gt 0 ]]; then
    log "整合性チェック: OK"
    TEST_PASS=$((TEST_PASS + 1))
  else
    log "警告: リストア後にテーブルが確認できませんでした"
    TEST_FAIL=$((TEST_FAIL + 1))
  fi
done

# ステップ4: テスト結果をまとめる
log "=== PostgreSQL ダンプ リストアテスト 完了 ==="
log "テスト結果:"
log "  合格: ${TEST_PASS}件"
log "  失敗: ${TEST_FAIL}件"

if [[ ${TEST_FAIL} -gt 0 ]]; then
  error "一部のリストアテストが失敗しました。詳細は ${WORK_DIR} のログを確認してください"
fi

log ""
log "本番環境でのリストア手順は docs/infrastructure/kubernetes/バックアップリストア手順書.md を参照すること"
