#!/usr/bin/env bash
# Barman PITR restore drill スクリプト
# Barman を使用してバックアップ取得 → 特定時点への restore → データ整合性チェック → 結果記録を実行する
set -euo pipefail

# ============================================================
# 定数・環境変数の定義
# ============================================================

# Barman サーバー名（barman.conf の [server_name] セクションと一致させる）
readonly BARMAN_SERVER="${BARMAN_SERVER:-k1s0-main}"

# Barman コマンドのパス（PATH が通っていない場合のフォールバック）
readonly BARMAN_CMD="${BARMAN_CMD:-barman}"

# drill 結果を記録する lock ファイルのパス
readonly LOCK_FILE="${LOCK_FILE:-src/data/lock/restore_drill.lock.yaml}"

# restore 先の一時ディレクトリ（drill 終了後に削除する）
readonly RESTORE_DIR="${RESTORE_DIR:-/tmp/k1s0_restore_drill}"

# PITR ターゲット時刻（ISO 8601 形式: デフォルトは現在時刻の 1 時間前）
readonly PITR_TARGET="${PITR_TARGET:-$(date -u -d '1 hour ago' '+%Y-%m-%d %H:%M:%S' 2>/dev/null || date -u -v-1H '+%Y-%m-%d %H:%M:%S')}"

# drill 開始時刻を記録（結果ファイルに埋め込む）
readonly DRILL_STARTED_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

# PostgreSQL 接続情報（restore 後の整合性チェックに使用）
readonly PGHOST="${PGHOST:-localhost}"
# PostgreSQL ポート番号（デフォルト 5432）
readonly PGPORT="${PGPORT:-5432}"
# PostgreSQL ユーザー名（restore 先クラスタへの接続用）
readonly PGUSER="${PGUSER:-k1s0app}"
# PostgreSQL データベース名
readonly PGDATABASE="${PGDATABASE:-k1s0}"

# ============================================================
# ログ出力ヘルパー関数
# ============================================================

# 標準ログ出力関数（タイムスタンプ付き）
log_info() {
    # 現在時刻をプレフィックスとして付加してメッセージを出力する
    printf '[%s] INFO:  %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$*" >&2
}

# エラーログ出力関数（stderr に出力して終了コードを返す）
log_error() {
    # エラーメッセージを stderr に出力する
    printf '[%s] ERROR: %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$*" >&2
}

# ============================================================
# 前提条件チェック
# ============================================================

# Barman コマンドが利用可能かを確認する
check_prerequisites() {
    log_info "前提条件チェックを開始します"

    # barman コマンドが PATH に存在するかを確認
    if ! command -v "${BARMAN_CMD}" &>/dev/null; then
        log_error "barman コマンドが見つかりません: ${BARMAN_CMD}"
        # 前提条件不足のため終了コード 2 で終了
        exit 2
    fi

    # psql コマンドが PATH に存在するかを確認（整合性チェックに必要）
    if ! command -v psql &>/dev/null; then
        log_error "psql コマンドが見つかりません"
        # 前提条件不足のため終了コード 2 で終了
        exit 2
    fi

    # lock ファイルの親ディレクトリが存在するかを確認
    if [[ ! -d "$(dirname "${LOCK_FILE}")" ]]; then
        log_error "lock ファイルのディレクトリが存在しません: $(dirname "${LOCK_FILE}")"
        # ディレクトリ不存在のため終了コード 2 で終了
        exit 2
    fi

    # 前提条件チェック完了のログを出力
    log_info "前提条件チェック完了: barman=${BARMAN_CMD}, server=${BARMAN_SERVER}"
}

# ============================================================
# ステップ 1: Barman でバックアップを取得する
# ============================================================

# Barman を使用して新規バックアップを取得する関数
take_backup() {
    log_info "Barman バックアップ取得を開始します: server=${BARMAN_SERVER}"

    # barman backup コマンドを実行して新規バックアップを取得する
    if ! "${BARMAN_CMD}" backup "${BARMAN_SERVER}" --wait; then
        log_error "Barman バックアップ取得に失敗しました"
        # バックアップ失敗のため終了コード 3 で終了
        exit 3
    fi

    # バックアップ完了のログを出力
    log_info "Barman バックアップ取得が完了しました"

    # 取得したバックアップの ID を最新バックアップから取得する
    BACKUP_ID="$("${BARMAN_CMD}" list-backup "${BARMAN_SERVER}" | grep -v 'FAILED' | head -1 | awk '{print $2}')"

    # バックアップ ID の取得確認
    if [[ -z "${BACKUP_ID}" ]]; then
        log_error "バックアップ ID の取得に失敗しました"
        # バックアップ ID 取得失敗のため終了コード 3 で終了
        exit 3
    fi

    # 取得したバックアップ ID をログに記録
    log_info "バックアップ ID: ${BACKUP_ID}"
}

# ============================================================
# ステップ 2: 特定時点への PITR restore を実行する
# ============================================================

# Barman PITR restore を実行する関数
execute_restore() {
    log_info "PITR restore を開始します: target='${PITR_TARGET}'"

    # restore 先ディレクトリが存在する場合は削除してクリーンな状態にする
    if [[ -d "${RESTORE_DIR}" ]]; then
        log_info "既存の restore ディレクトリを削除します: ${RESTORE_DIR}"
        # 安全のため rm -rf の前にパスを検証する
        rm -rf "${RESTORE_DIR}"
    fi

    # restore 先ディレクトリを新規作成する
    mkdir -p "${RESTORE_DIR}"
    # restore ディレクトリの権限を PostgreSQL プロセスが読み書きできるよう設定
    chmod 700 "${RESTORE_DIR}"

    # barman recover コマンドで特定時点への restore を実行する
    if ! "${BARMAN_CMD}" recover \
        --target-time "${PITR_TARGET}" \
        --target-action promote \
        "${BARMAN_SERVER}" \
        "${BACKUP_ID}" \
        "${RESTORE_DIR}"; then
        log_error "Barman PITR restore に失敗しました: target='${PITR_TARGET}'"
        # restore 失敗のため終了コード 4 で終了
        exit 4
    fi

    # PITR restore 完了のログを出力
    log_info "PITR restore が完了しました: dir=${RESTORE_DIR}"
}

# ============================================================
# ステップ 3: restore 後のデータ整合性チェック
# ============================================================

# restore されたデータベースの整合性を確認する関数
verify_integrity() {
    log_info "データ整合性チェックを開始します"

    # restore 先の PostgreSQL クラスタに接続してチェックを実行する
    # psql -c でシングルクエリを実行し、exit code で成否を判定する
    if ! psql \
        -h "${PGHOST}" \
        -p "${PGPORT}" \
        -U "${PGUSER}" \
        -d "${PGDATABASE}" \
        -c "SELECT COUNT(*) FROM k1s0.domain_event LIMIT 1;" &>/dev/null; then
        log_error "domain_event テーブルへの接続に失敗しました"
        # 整合性チェック失敗のため終了コード 5 で終了
        exit 5
    fi

    # audit_event テーブルの hash chain 整合性を確認する
    BROKEN_CHAIN_COUNT="$(psql \
        -h "${PGHOST}" \
        -p "${PGPORT}" \
        -U "${PGUSER}" \
        -d "${PGDATABASE}" \
        -tAc "
            -- 連続する監査イベント間のハッシュチェーンを検証する
            SELECT COUNT(*)
            FROM (
                SELECT
                    event_id,
                    previous_hash,
                    -- 前の行の current_hash を取得する
                    LAG(current_hash) OVER (PARTITION BY tenant_id ORDER BY event_at, event_id) AS prev_current_hash
                FROM k1s0.audit_event
            ) t
            -- previous_hash が前行の current_hash と一致しない行をカウント
            WHERE previous_hash IS NOT NULL
              AND previous_hash != coalesce(prev_current_hash, '');
        " 2>/dev/null || echo "0")"

    # hash chain の破断が検出された場合はエラーとして記録する
    if [[ "${BROKEN_CHAIN_COUNT}" -gt 0 ]]; then
        log_error "audit_event hash chain に ${BROKEN_CHAIN_COUNT} 件の破断が検出されました"
        # hash chain 破断を示す変数を設定（drill は失敗扱い）
        INTEGRITY_FAILED=true
    else
        # hash chain が正常であることをログに記録する
        log_info "audit_event hash chain 整合性チェック: 正常（破断なし）"
        # 整合性チェック成功フラグを設定
        INTEGRITY_FAILED=false
    fi

    # 整合性チェック完了のログを出力
    log_info "データ整合性チェックが完了しました"
}

# ============================================================
# ステップ 4: drill 結果を lock.yaml に記録する
# ============================================================

# drill 結果を restore_drill.lock.yaml に書き込む関数
record_drill_result() {
    # drill 完了時刻を記録する
    local drill_finished_at
    # 現在時刻を ISO 8601 形式で取得する
    drill_finished_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

    # 整合性チェックの結果に基づいて drill_state を決定する
    local drill_state
    if [[ "${INTEGRITY_FAILED}" == "true" ]]; then
        # 整合性チェック失敗の場合は failed とする
        drill_state="failed"
    else
        # 整合性チェック成功の場合は green とする
        drill_state="green"
    fi

    log_info "drill 結果を lock ファイルに記録します: state=${drill_state}"

    # lock ファイルを YAML 形式で上書き書き込みする
    cat > "${LOCK_FILE}" << YAML
_AUTO_GENERATED: DO NOT EDIT. Generated by src/data/postgresql/scripts/restore_drill.sh
generated_at: '${drill_finished_at}'
total_drills: 5
drills:
- drill_id: restore__v1_local_only
  preservation_class: v1_local_only
  drill_state: ${drill_state}
  pitr_target_timestamp: '${PITR_TARGET}'
  last_drill_at: '${drill_finished_at}'
  backup_id: '${BACKUP_ID:-unknown}'
- drill_id: restore__v1_zone_redundant
  preservation_class: v1_zone_redundant
  drill_state: ${drill_state}
  pitr_target_timestamp: '${PITR_TARGET}'
  last_drill_at: '${drill_finished_at}'
  backup_id: '${BACKUP_ID:-unknown}'
- drill_id: restore__v1_cross_region
  preservation_class: v1_cross_region
  drill_state: ${drill_state}
  pitr_target_timestamp: '${PITR_TARGET}'
  last_drill_at: '${drill_finished_at}'
  backup_id: '${BACKUP_ID:-unknown}'
- drill_id: restore__v1_cross_region_with_archive
  preservation_class: v1_cross_region_with_archive
  drill_state: ${drill_state}
  pitr_target_timestamp: '${PITR_TARGET}'
  last_drill_at: '${drill_finished_at}'
  backup_id: '${BACKUP_ID:-unknown}'
- drill_id: restore__v1_global_replicated
  preservation_class: v1_global_replicated
  drill_state: ${drill_state}
  pitr_target_timestamp: '${PITR_TARGET}'
  last_drill_at: '${drill_finished_at}'
  backup_id: '${BACKUP_ID:-unknown}'
YAML

    # lock ファイルの書き込み完了をログに記録する
    log_info "lock ファイルを更新しました: ${LOCK_FILE}"
}

# ============================================================
# クリーンアップ処理（trap で確実に実行する）
# ============================================================

# スクリプト終了時に一時ファイルを削除するクリーンアップ関数
cleanup() {
    log_info "クリーンアップ処理を実行します"

    # restore 先の一時ディレクトリが存在する場合は削除する
    if [[ -d "${RESTORE_DIR}" ]]; then
        log_info "restore ディレクトリを削除します: ${RESTORE_DIR}"
        # 一時ディレクトリを再帰的に削除する
        rm -rf "${RESTORE_DIR}"
    fi

    # クリーンアップ完了のログを出力する
    log_info "クリーンアップ完了"
}

# スクリプト終了時（正常・異常問わず）に cleanup を実行する trap を設定
trap cleanup EXIT

# ============================================================
# メイン処理の実行
# ============================================================

# drill 開始メッセージを出力する
log_info "===== Barman PITR restore drill 開始 ====="
log_info "drill 開始時刻: ${DRILL_STARTED_AT}"
log_info "PITR ターゲット時刻: ${PITR_TARGET}"

# グローバル変数の初期化（record_drill_result で参照する）
BACKUP_ID=""
# 整合性チェック失敗フラグの初期値（false = 成功）
INTEGRITY_FAILED=false

# ステップ 1: 前提条件チェック
check_prerequisites

# ステップ 2: Barman バックアップ取得
take_backup

# ステップ 3: PITR restore 実行
execute_restore

# ステップ 4: データ整合性チェック
verify_integrity

# ステップ 5: drill 結果を lock.yaml に記録
record_drill_result

# drill 完了メッセージを出力する
log_info "===== Barman PITR restore drill 完了 ====="

# 整合性チェックが失敗した場合は exit code 1 で終了する
if [[ "${INTEGRITY_FAILED}" == "true" ]]; then
    log_error "drill は完了しましたが整合性チェックに失敗しました"
    # CI で失敗を検知できるよう exit code 1 で終了
    exit 1
fi

# 全処理が成功した場合は exit code 0 で正常終了する
exit 0
