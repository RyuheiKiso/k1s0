#!/usr/bin/env bash
# clock skew injection drill: 時刻ずれ注入とeBPF probe 検出を検証するスクリプト
# コンテナ内の時刻を一時的に変更し、clock_skew_probe.bpf.c が検出するか確認する
# 結果を src/infra/lock/clock_causality_proof.lock.yaml に記録する
set -euo pipefail

# ============================================================
# 定数定義: スクリプト全体で使用するパスと設定値を宣言する
# ============================================================
# このスクリプトが配置されているディレクトリの絶対パスを取得する
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# リポジトリルートへの相対パスを解決する
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
# clock_causality_proof.lock.yaml の書き込み先パスを定義する
LOCK_FILE="${REPO_ROOT}/src/infra/lock/clock_causality_proof.lock.yaml"
# 対象クラスタ名（kind コンテナ名のプレフィックス）
TARGET_CLUSTER="${TARGET_CLUSTER:-k1s0-target}"
# clock skew 注入量（秒）：この値だけ時刻を進める
SKEW_SECONDS="${SKEW_SECONDS:-30}"
# eBPF probe の検出を待機する最大秒数
DETECTION_TIMEOUT="${DETECTION_TIMEOUT:-60}"

# ============================================================
# ユーティリティ関数: ログ出力と時刻取得を提供する
# ============================================================
# ログメッセージを ISO 8601 タイムスタンプ付きで出力する関数
log() {
  # 第一引数をメッセージとして受け取り標準エラーに出力する
  echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*" >&2
}

# エラーメッセージを出力してスクリプトを終了する関数
die() {
  # エラーメッセージを標準エラーに出力する
  log "ERROR: $*"
  # 終了コード 1 でスクリプトを終了する
  exit 1
}

# 現在時刻を ISO 8601 形式で返す関数
now_iso() {
  # UTC 時刻を ISO 8601 形式で出力する
  date -u +%Y-%m-%dT%H:%M:%SZ
}

# ============================================================
# 前提条件チェック: 必要なコマンドが使用可能か確認する
# ============================================================
check_prerequisites() {
  # チェック対象コマンドを配列で定義する
  local required_cmds=("docker" "kubectl" "yq")
  # 各コマンドの存在を確認するループ
  for cmd in "${required_cmds[@]}"; do
    # command -v でコマンドの存在を確認する
    if ! command -v "${cmd}" &>/dev/null; then
      die "Required command not found: ${cmd}"
    fi
  done
  # 全コマンドが揃っている場合は成功ログを出力する
  log "All prerequisites satisfied."
}

# ============================================================
# ターゲットコンテナ取得: ワーカーノードの docker コンテナ ID を返す
# ============================================================
get_worker_container() {
  # kind クラスタのワーカーコンテナ名を取得する
  local container
  container=$(docker ps --filter "name=${TARGET_CLUSTER}-worker" --format "{{.Names}}" | head -1)
  # コンテナが見つからない場合はエラーを返す
  if [[ -z "${container}" ]]; then
    die "Worker container not found for cluster: ${TARGET_CLUSTER}"
  fi
  # コンテナ名を標準出力に返す
  echo "${container}"
}

# ============================================================
# clock skew 注入: docker exec で date コマンドを使って時刻を変更する
# ============================================================
inject_clock_skew() {
  # 第一引数を対象コンテナ名として受け取る
  local container="$1"
  # 第二引数をスキュー量（秒）として受け取る
  local skew_seconds="$2"
  # 現在時刻を取得する
  local current_time
  current_time=$(date -u +%s)
  # スキューを加算した新しい時刻を計算する
  local new_time=$((current_time + skew_seconds))
  # 新しい時刻を date コマンドの引数形式に変換する
  local new_time_str
  new_time_str=$(date -u -d "@${new_time}" +%Y-%m-%dT%H:%M:%S 2>/dev/null || date -u -r "${new_time}" +%Y-%m-%dT%H:%M:%S)

  # clock skew 注入開始ログを出力する
  log "Injecting clock skew: container=${container}, skew=${skew_seconds}s, target_time=${new_time_str}"
  # docker exec で対象コンテナの時刻を変更する（CAP_SYS_TIME が必要）
  docker exec --privileged "${container}" \
    date -s "${new_time_str}" \
    2>/dev/null || {
    # 時刻変更に失敗した場合は警告を出力してスキップする
    log "WARN: Failed to inject clock skew via date command (may need --privileged). Simulating detection."
    return 1
  }
  # 注入成功ログを出力する
  log "Clock skew injected successfully: ${skew_seconds}s forward."
}

# ============================================================
# eBPF 検出確認: clock_skew_probe が時刻ずれを検出したか確認する
# ============================================================
verify_ebpf_detection() {
  # 第一引数を対象コンテナ名として受け取る
  local container="$1"
  # 第二引数を検出待機タイムアウト（秒）として受け取る
  local timeout_seconds="$2"
  # 検出確認開始ログを出力する
  log "Waiting for eBPF probe detection (timeout=${timeout_seconds}s)..."

  # カウンタを初期化する
  local elapsed=0
  # タイムアウト内で検出を待機するループ
  while [[ ${elapsed} -lt ${timeout_seconds} ]]; do
    # /sys/kernel/debug/tracing/trace_pipe から probe イベントを確認する
    local probe_output
    probe_output=$(docker exec "${container}" \
      cat /sys/kernel/debug/tracing/trace_pipe 2>/dev/null | head -5 || echo "")
    # probe 出力に clock_skew_detected キーワードが含まれるか確認する
    if echo "${probe_output}" | grep -q "clock_skew_detected"; then
      log "eBPF probe detected clock skew! (elapsed=${elapsed}s)"
      echo "detected"
      return 0
    fi
    # 1 秒待機してリトライする
    sleep 1
    elapsed=$((elapsed + 1))
  done

  # タイムアウト内に検出されなかった場合の警告ログを出力する
  log "WARN: eBPF probe did not detect clock skew within ${timeout_seconds}s."
  # 検出されなかった場合は not_detected を返す
  echo "not_detected"
}

# ============================================================
# 時刻復旧: 注入した clock skew を元に戻す
# ============================================================
restore_clock() {
  # 第一引数を対象コンテナ名として受け取る
  local container="$1"
  # 時刻復旧開始ログを出力する
  log "Restoring clock to correct time: container=${container}"
  # host の時刻をコンテナに sync する
  local host_time
  host_time=$(date -u +%Y-%m-%dT%H:%M:%S)
  # docker exec で対象コンテナの時刻を正常値に戻す
  docker exec --privileged "${container}" \
    date -s "${host_time}" \
    2>/dev/null || log "WARN: Failed to restore clock (may need --privileged)."
  # 時刻復旧完了ログを出力する
  log "Clock restored to: ${host_time}"
}

# ============================================================
# lock.yaml 更新: drill 結果を clock_causality_proof.lock.yaml に書き込む
# ============================================================
update_clock_lock_yaml() {
  # 第一引数を clock_class として受け取る
  local clock_class="$1"
  # 第二引数を新しい drill_state として受け取る
  local new_state="$2"
  # 第三引数を実行時刻として受け取る
  local executed_at="$3"

  # lock.yaml ファイルが存在するか確認する
  if [[ ! -f "${LOCK_FILE}" ]]; then
    die "Lock file not found: ${LOCK_FILE}"
  fi

  # yq を使って指定 clock_class の drill_state を更新する
  yq e \
    "(.classes[] | select(.class_id == \"${clock_class}\") | .drill_state) = \"${new_state}\"" \
    -i "${LOCK_FILE}"

  # executed_at フィールドを追加・更新する
  yq e \
    "(.classes[] | select(.class_id == \"${clock_class}\") | .executed_at) = \"${executed_at}\"" \
    -i "${LOCK_FILE}"

  # 更新完了ログを出力する
  log "Clock lock yaml updated: ${clock_class} -> ${new_state} at ${executed_at}"
}

# ============================================================
# メイン処理: 全 clock_class のドリルを順次実行する
# ============================================================
main() {
  # 前提条件チェックを実行する
  check_prerequisites

  # ドリル開始メッセージを出力する
  log "=== k1s0 clock skew injection drill start ==="
  # 対象クラスタ名を出力する
  log "Target cluster: ${TARGET_CLUSTER}, skew=${SKEW_SECONDS}s"

  # 対象ワーカーコンテナを取得する
  local worker_container
  worker_container=$(get_worker_container)
  log "Worker container: ${worker_container}"

  # ドリル対象 clock_class リストを定義する
  local -a clock_classes=(
    "v1_intra_rack_ptp"
    "v1_dc_chrony_stratum1"
    "v1_multi_region_ntp"
    "v1_legacy_skew_tolerant"
    "v1_air_gapped_legacy"
  )

  # 各 clock_class に対してドリルを実行するループ
  for clock_class in "${clock_classes[@]}"; do
    # ドリル開始時刻を記録する
    local start_time
    start_time=$(now_iso)
    log "--- Running clock drill: ${clock_class} ---"

    # clock skew を注入する（失敗した場合はスキップする）
    inject_clock_skew "${worker_container}" "${SKEW_SECONDS}" || {
      log "WARN: Skipping eBPF detection for ${clock_class} (injection failed)."
      update_clock_lock_yaml "${clock_class}" "skipped" "${start_time}"
      continue
    }

    # eBPF probe による検出を確認する
    local detection_result
    detection_result=$(verify_ebpf_detection "${worker_container}" "${DETECTION_TIMEOUT}")

    # 時刻を正常値に復旧する
    restore_clock "${worker_container}"

    # 検出結果に応じて drill_state を設定する
    local drill_state
    if [[ "${detection_result}" == "detected" ]]; then
      # 検出された場合は green とする
      drill_state="green"
    else
      # 検出されなかった場合は red とする（要調査）
      drill_state="red"
    fi

    # lock.yaml にドリル結果を書き込む
    update_clock_lock_yaml "${clock_class}" "${drill_state}" "${start_time}"
    # ドリル結果サマリを出力する
    log "Clock drill ${clock_class} completed: ${drill_state}"
  done

  # 全ドリル完了メッセージを出力する
  log "=== k1s0 clock skew injection drill complete ==="
}

# エントリーポイント: main 関数を呼び出す
main "$@"
