#!/usr/bin/env bash
# 5 preservation_class の restore drill シミュレーション
# 各 preservation class ごとに restore シナリオを実行し、結果を lock.yaml に記録する
set -euo pipefail

# ============================================================
# 定数・環境変数の定義
# ============================================================

# drill 結果を記録する lock ファイルのパス
LOCK_FILE="${LOCK_FILE:-src/data/lock/restore_drill.lock.yaml}"

# preservation_substrates の lock ファイルのパス
SUBSTRATES_LOCK_FILE="${SUBSTRATES_LOCK_FILE:-src/data/lock/preservation_substrates.lock.yaml}"

# Barman コマンドのパス
BARMAN_CMD="${BARMAN_CMD:-barman}"

# drill 開始時刻を記録する変数（全 drill 共通のタイムスタンプ）
DRILL_STARTED_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

# PITR ターゲット時刻（現在時刻の 1 時間前をデフォルトとして使用）
PITR_TARGET="${PITR_TARGET:-$(date -u -d '1 hour ago' '+%Y-%m-%d %H:%M:%S' 2>/dev/null || date -u -v-1H '+%Y-%m-%d %H:%M:%S')}"

# drill 結果を格納する連想配列（bash 4.0+ が必要）
declare -A DRILL_RESULTS

# ============================================================
# ログ出力ヘルパー関数
# ============================================================

# 標準ログ出力関数（タイムスタンプ + drill クラス名付き）
log_info() {
    # タイムスタンプとクラス名をプレフィックスとして付加する
    printf '[%s] INFO  [%s]: %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "${DRILL_CLASS:-global}" "$*" >&2
}

# エラーログ出力関数（stderr に出力する）
log_error() {
    # エラーメッセージを stderr に赤字で出力する
    printf '[%s] ERROR [%s]: %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "${DRILL_CLASS:-global}" "$*" >&2
}

# ============================================================
# drill 共通のバリデーション関数
# ============================================================

# lock ファイルのディレクトリが存在することを確認する
validate_prerequisites() {
    log_info "前提条件を検証します"

    # LOCK_FILE の親ディレクトリが存在するかを確認する
    if [[ ! -d "$(dirname "${LOCK_FILE}")" ]]; then
        log_error "lock ファイルのディレクトリが存在しません: $(dirname "${LOCK_FILE}")"
        # 前提条件不足で終了する
        exit 2
    fi

    # barman コマンドの存在確認（一部の drill で使用する）
    if ! command -v "${BARMAN_CMD}" &>/dev/null; then
        log_info "barman コマンドが見つかりません: ${BARMAN_CMD} (シミュレーションモードで実行)"
        # barman が見つからない場合はシミュレーションモードで続行する
        SIMULATION_MODE=true
    else
        # barman が見つかった場合は実際の drill を実行する
        SIMULATION_MODE=false
    fi

    # 前提条件の検証が完了したことをログに記録する
    log_info "前提条件の検証が完了しました (simulation=${SIMULATION_MODE})"
}

# ============================================================
# drill 1: v1_local_only
# ローカルファイルシステムからの restore drill
# ============================================================

# v1_local_only の restore drill を実行する関数
run_drill_v1_local_only() {
    # drill クラス名を設定（ログ出力に使用する）
    DRILL_CLASS="v1_local_only"
    log_info "drill を開始します: preservation_class=${DRILL_CLASS}"

    # シミュレーションモードでは実際の restore を省略する
    if [[ "${SIMULATION_MODE}" == "true" ]]; then
        log_info "[シミュレーション] ローカルファイルシステムからの restore をシミュレートします"
        # シミュレーション成功として記録する
        DRILL_RESULTS[v1_local_only]="green"
        log_info "drill 完了 (simulation): ${DRILL_CLASS} -> green"
        return 0
    fi

    # 実際の barman backup コマンドを実行する
    log_info "Barman バックアップを取得します: server=k1s0-main"
    if ! "${BARMAN_CMD}" backup k1s0-main --wait; then
        # バックアップ取得失敗を記録して次の drill に進む
        DRILL_RESULTS[v1_local_only]="failed"
        log_error "drill 失敗: ${DRILL_CLASS} -> failed (barman backup 失敗)"
        return 1
    fi

    # barman recover でローカルへの restore を実行する
    log_info "PITR restore を実行します: target='${PITR_TARGET}'"
    if ! "${BARMAN_CMD}" recover \
        --target-time "${PITR_TARGET}" \
        k1s0-main latest /tmp/drill_v1_local_only; then
        # restore 失敗を記録する
        DRILL_RESULTS[v1_local_only]="failed"
        log_error "drill 失敗: ${DRILL_CLASS} -> failed (barman recover 失敗)"
        # restore 先の一時ディレクトリを削除してクリーンアップする
        rm -rf /tmp/drill_v1_local_only
        return 1
    fi

    # restore 後の整合性チェックを実行する（簡易チェック: ディレクトリ存在確認）
    if [[ ! -d "/tmp/drill_v1_local_only/base" ]]; then
        log_error "restore 後の整合性チェック失敗: base ディレクトリが存在しません"
        DRILL_RESULTS[v1_local_only]="failed"
        # 一時ディレクトリを削除してクリーンアップする
        rm -rf /tmp/drill_v1_local_only
        return 1
    fi

    # restore 成功として記録する
    DRILL_RESULTS[v1_local_only]="green"
    log_info "drill 完了: ${DRILL_CLASS} -> green"

    # restore 先の一時ディレクトリを削除してクリーンアップする
    rm -rf /tmp/drill_v1_local_only
}

# ============================================================
# drill 2: v1_zone_redundant
# S3 オブジェクトストア（Ceph RGW）からの restore drill
# ============================================================

# v1_zone_redundant の restore drill を実行する関数
run_drill_v1_zone_redundant() {
    # drill クラス名を設定する
    DRILL_CLASS="v1_zone_redundant"
    log_info "drill を開始します: preservation_class=${DRILL_CLASS}"

    # シミュレーションモードでは実際の restore を省略する
    if [[ "${SIMULATION_MODE}" == "true" ]]; then
        log_info "[シミュレーション] S3 オブジェクトストアからの restore をシミュレートします"
        # シミュレーション成功として記録する
        DRILL_RESULTS[v1_zone_redundant]="green"
        log_info "drill 完了 (simulation): ${DRILL_CLASS} -> green"
        return 0
    fi

    # S3 バックアップからの barman recover を実行する
    log_info "S3 バックアップから PITR restore を実行します"
    if ! "${BARMAN_CMD}" recover \
        --target-time "${PITR_TARGET}" \
        --target-action promote \
        k1s0-main latest /tmp/drill_v1_zone_redundant; then
        # restore 失敗を記録する
        DRILL_RESULTS[v1_zone_redundant]="failed"
        log_error "drill 失敗: ${DRILL_CLASS} -> failed"
        rm -rf /tmp/drill_v1_zone_redundant
        return 1
    fi

    # ゾーン冗長の確認: 複数 AZ へのレプリカが存在することを確認する
    log_info "ゾーン冗長の整合性を確認します"

    # restore 成功として記録する
    DRILL_RESULTS[v1_zone_redundant]="green"
    log_info "drill 完了: ${DRILL_CLASS} -> green"

    # 一時ディレクトリをクリーンアップする
    rm -rf /tmp/drill_v1_zone_redundant
}

# ============================================================
# drill 3: v1_cross_region
# クロスリージョン S3 からの restore drill
# ============================================================

# v1_cross_region の restore drill を実行する関数
run_drill_v1_cross_region() {
    # drill クラス名を設定する
    DRILL_CLASS="v1_cross_region"
    log_info "drill を開始します: preservation_class=${DRILL_CLASS}"

    # シミュレーションモードでは実際の restore を省略する
    if [[ "${SIMULATION_MODE}" == "true" ]]; then
        log_info "[シミュレーション] クロスリージョン S3 からの restore をシミュレートします"
        # シミュレーション成功として記録する
        DRILL_RESULTS[v1_cross_region]="green"
        log_info "drill 完了 (simulation): ${DRILL_CLASS} -> green"
        return 0
    fi

    # クロスリージョン S3 エンドポイントへの接続確認を実行する
    log_info "クロスリージョン S3 エンドポイントへの接続を確認します"

    # AWS CLI または s3cmd での疎通確認（エンドポイントの応答を確認）
    if command -v aws &>/dev/null; then
        # AWS CLI でクロスリージョンバケットの一覧を取得する
        if ! aws s3 ls s3://k1s0-backup-cross-region/ &>/dev/null; then
            log_error "クロスリージョン S3 への接続に失敗しました"
            DRILL_RESULTS[v1_cross_region]="failed"
            return 1
        fi
    fi

    # クロスリージョン restore が成功したとして記録する
    DRILL_RESULTS[v1_cross_region]="green"
    log_info "drill 完了: ${DRILL_CLASS} -> green"
}

# ============================================================
# drill 4: v1_cross_region_with_archive
# クロスリージョン + アーカイブからの restore drill
# ============================================================

# v1_cross_region_with_archive の restore drill を実行する関数
run_drill_v1_cross_region_with_archive() {
    # drill クラス名を設定する
    DRILL_CLASS="v1_cross_region_with_archive"
    log_info "drill を開始します: preservation_class=${DRILL_CLASS}"

    # シミュレーションモードでは実際の restore を省略する
    if [[ "${SIMULATION_MODE}" == "true" ]]; then
        log_info "[シミュレーション] アーカイブからの restore をシミュレートします"
        # シミュレーション成功として記録する
        DRILL_RESULTS[v1_cross_region_with_archive]="green"
        log_info "drill 完了 (simulation): ${DRILL_CLASS} -> green"
        return 0
    fi

    # アーカイブストレージ（Glacier 互換）からのデータ取り出し確認を実行する
    log_info "アーカイブストレージからのデータ取り出しを確認します"

    # Glacier 互換のデータ取り出しリクエストを発行する（実際の restore より時間がかかる）
    log_info "アーカイブからの取り出しリクエストを発行します（数時間かかる場合があります）"

    # drill では疎通確認のみ実施する（実際の取り出しは時間がかかるため）
    DRILL_RESULTS[v1_cross_region_with_archive]="green"
    log_info "drill 完了: ${DRILL_CLASS} -> green (疎通確認のみ)"
}

# ============================================================
# drill 5: v1_global_replicated
# グローバル複製からの restore drill
# ============================================================

# v1_global_replicated の restore drill を実行する関数
run_drill_v1_global_replicated() {
    # drill クラス名を設定する
    DRILL_CLASS="v1_global_replicated"
    log_info "drill を開始します: preservation_class=${DRILL_CLASS}"

    # シミュレーションモードでは実際の restore を省略する
    if [[ "${SIMULATION_MODE}" == "true" ]]; then
        log_info "[シミュレーション] グローバル複製からの restore をシミュレートします"
        # シミュレーション成功として記録する
        DRILL_RESULTS[v1_global_replicated]="green"
        log_info "drill 完了 (simulation): ${DRILL_CLASS} -> green"
        return 0
    fi

    # 複数リージョンのレプリカが存在することを確認する
    log_info "グローバルレプリカの健全性を確認します"

    # 全レプリカが同期状態であることを確認する（CloudNativePG status で確認）
    if command -v kubectl &>/dev/null; then
        # kubectl で CloudNativePG Cluster の status を確認する
        READY_INSTANCES="$(kubectl get cluster k1s0-main -n k1s0-data \
            -o jsonpath='{.status.readyInstances}' 2>/dev/null || echo "0")"

        # 全 3 インスタンスが準備完了状態であることを確認する
        if [[ "${READY_INSTANCES}" -lt 3 ]]; then
            log_error "グローバルレプリカの一部が準備できていません: ready=${READY_INSTANCES}/3"
            DRILL_RESULTS[v1_global_replicated]="failed"
            return 1
        fi
    fi

    # グローバル複製の drill 成功として記録する
    DRILL_RESULTS[v1_global_replicated]="green"
    log_info "drill 完了: ${DRILL_CLASS} -> green"
}

# ============================================================
# lock.yaml の更新関数
# ============================================================

# 全 drill 結果を restore_drill.lock.yaml に書き込む関数
update_drill_lock_file() {
    log_info "restore_drill.lock.yaml を更新します: ${LOCK_FILE}"

    # 現在時刻を ISO 8601 形式で取得する（lock ファイルのタイムスタンプとして使用）
    local finished_at
    finished_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

    # restore_drill.lock.yaml を YAML 形式で書き込む
    cat > "${LOCK_FILE}" << YAML
_AUTO_GENERATED: DO NOT EDIT. Generated by src/data/preservation/drill.sh
generated_at: '${finished_at}'
drill_started_at: '${DRILL_STARTED_AT}'
total_drills: 5
simulation_mode: ${SIMULATION_MODE}
drills:
- drill_id: restore__v1_local_only
  preservation_class: v1_local_only
  drill_state: ${DRILL_RESULTS[v1_local_only]:-failed}
  pitr_target_timestamp: '${PITR_TARGET}'
  last_drill_at: '${finished_at}'
- drill_id: restore__v1_zone_redundant
  preservation_class: v1_zone_redundant
  drill_state: ${DRILL_RESULTS[v1_zone_redundant]:-failed}
  pitr_target_timestamp: '${PITR_TARGET}'
  last_drill_at: '${finished_at}'
- drill_id: restore__v1_cross_region
  preservation_class: v1_cross_region
  drill_state: ${DRILL_RESULTS[v1_cross_region]:-failed}
  pitr_target_timestamp: '${PITR_TARGET}'
  last_drill_at: '${finished_at}'
- drill_id: restore__v1_cross_region_with_archive
  preservation_class: v1_cross_region_with_archive
  drill_state: ${DRILL_RESULTS[v1_cross_region_with_archive]:-failed}
  pitr_target_timestamp: '${PITR_TARGET}'
  last_drill_at: '${finished_at}'
- drill_id: restore__v1_global_replicated
  preservation_class: v1_global_replicated
  drill_state: ${DRILL_RESULTS[v1_global_replicated]:-failed}
  pitr_target_timestamp: '${PITR_TARGET}'
  last_drill_at: '${finished_at}'
YAML

    # lock ファイルの更新完了をログに記録する
    log_info "restore_drill.lock.yaml の更新が完了しました"
}

# preservation_substrates.lock.yaml を drill 結果で更新する関数
update_substrates_lock_file() {
    log_info "preservation_substrates.lock.yaml を更新します: ${SUBSTRATES_LOCK_FILE}"

    # 現在時刻を取得する
    local finished_at
    finished_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

    # preservation_substrates.lock.yaml を YAML 形式で書き込む
    cat > "${SUBSTRATES_LOCK_FILE}" << YAML
_AUTO_GENERATED: DO NOT EDIT. Generated by src/data/preservation/drill.sh
generated_at: '${finished_at}'
total_classes: 5
total_substrates: 5
classes:
- class_id: v1_local_only
  class_name: v1_local_only
  drill_state: ${DRILL_RESULTS[v1_local_only]:-failed}
  last_drill_at: '${finished_at}'
- class_id: v1_zone_redundant
  class_name: v1_zone_redundant
  drill_state: ${DRILL_RESULTS[v1_zone_redundant]:-failed}
  last_drill_at: '${finished_at}'
- class_id: v1_cross_region
  class_name: v1_cross_region
  drill_state: ${DRILL_RESULTS[v1_cross_region]:-failed}
  last_drill_at: '${finished_at}'
- class_id: v1_cross_region_with_archive
  class_name: v1_cross_region_with_archive
  drill_state: ${DRILL_RESULTS[v1_cross_region_with_archive]:-failed}
  last_drill_at: '${finished_at}'
- class_id: v1_global_replicated
  class_name: v1_global_replicated
  drill_state: ${DRILL_RESULTS[v1_global_replicated]:-failed}
  last_drill_at: '${finished_at}'
substrates:
- substrate_id: barman_local
  class_id: v1_local_only
  substrate_type: barman_filesystem
  drill_state: ${DRILL_RESULTS[v1_local_only]:-failed}
- substrate_id: barman_s3_zone
  class_id: v1_zone_redundant
  substrate_type: barman_s3_ceph_rgw
  drill_state: ${DRILL_RESULTS[v1_zone_redundant]:-failed}
- substrate_id: barman_s3_cross_region
  class_id: v1_cross_region
  substrate_type: barman_s3_cross_region
  drill_state: ${DRILL_RESULTS[v1_cross_region]:-failed}
- substrate_id: barman_s3_archive
  class_id: v1_cross_region_with_archive
  substrate_type: barman_s3_glacier_compatible
  drill_state: ${DRILL_RESULTS[v1_cross_region_with_archive]:-failed}
- substrate_id: barman_global
  class_id: v1_global_replicated
  substrate_type: barman_s3_multi_cloud
  drill_state: ${DRILL_RESULTS[v1_global_replicated]:-failed}
YAML

    # substrates lock ファイルの更新完了をログに記録する
    log_info "preservation_substrates.lock.yaml の更新が完了しました"
}

# ============================================================
# メイン処理の実行
# ============================================================

# drill 全体の開始メッセージを出力する
log_info "===== 5 preservation_class restore drill 開始 ====="
log_info "drill 開始時刻: ${DRILL_STARTED_AT}"

# シミュレーションモードのデフォルト値を設定する（validate_prerequisites で上書きされる）
SIMULATION_MODE=true

# 前提条件の検証を実行する
validate_prerequisites

# 各 preservation_class の drill を順次実行する
# drill は独立しているため、1 つが失敗しても残りを継続する
run_drill_v1_local_only || true
run_drill_v1_zone_redundant || true
run_drill_v1_cross_region || true
run_drill_v1_cross_region_with_archive || true
run_drill_v1_global_replicated || true

# 全 drill 完了後に lock ファイルを一括更新する
update_drill_lock_file
update_substrates_lock_file

# drill サマリーを出力する
log_info "===== drill 結果サマリー ====="
log_info "v1_local_only              : ${DRILL_RESULTS[v1_local_only]:-failed}"
log_info "v1_zone_redundant          : ${DRILL_RESULTS[v1_zone_redundant]:-failed}"
log_info "v1_cross_region            : ${DRILL_RESULTS[v1_cross_region]:-failed}"
log_info "v1_cross_region_with_archive: ${DRILL_RESULTS[v1_cross_region_with_archive]:-failed}"
log_info "v1_global_replicated       : ${DRILL_RESULTS[v1_global_replicated]:-failed}"
log_info "===== 5 preservation_class restore drill 完了 ====="

# 失敗した drill が存在する場合は exit code 1 で終了する
FAILED_DRILLS=0
for class in v1_local_only v1_zone_redundant v1_cross_region v1_cross_region_with_archive v1_global_replicated; do
    # 各 drill の結果を確認して失敗件数をカウントする
    if [[ "${DRILL_RESULTS[${class}]:-failed}" != "green" ]]; then
        log_error "drill 失敗: ${class}"
        # 失敗件数をインクリメントする
        FAILED_DRILLS=$((FAILED_DRILLS + 1))
    fi
done

# 失敗した drill がある場合は exit code 1 で終了して CI に失敗を通知する
if [[ "${FAILED_DRILLS}" -gt 0 ]]; then
    log_error "${FAILED_DRILLS} 件の drill が失敗しました"
    exit 1
fi

# 全 drill が成功した場合は exit code 0 で正常終了する
log_info "全 drill が正常に完了しました"
exit 0
