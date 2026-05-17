#!/usr/bin/env bash
# topology drill: Litmus chaos を使った zone outage シミュレーションスクリプト
# 各 topology_class の failover_drill.yaml に定義されたシナリオを実行し
# 結果を src/infra/lock/failover_drill.lock.yaml に書き込む
set -euo pipefail

# ============================================================
# 定数定義: スクリプト全体で使用するパスと設定値を宣言する
# ============================================================
# このスクリプトが配置されているディレクトリの絶対パスを取得する
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# リポジトリルートへの相対パスを解決する
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
# lock.yaml の書き込み先パスを定義する
LOCK_FILE="${REPO_ROOT}/src/infra/lock/failover_drill.lock.yaml"
# topology classes 定義ファイルのパスを定義する
CLASSES_FILE="${SCRIPT_DIR}/classes.yaml"
# ドリルのタイムアウト秒数（デフォルト値を設定する）
DRILL_TIMEOUT="${DRILL_TIMEOUT:-300}"
# 対象クラスタ名（kind コンテナ名のプレフィックス）
TARGET_CLUSTER="${TARGET_CLUSTER:-k1s0-target}"

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
# 必要なコマンドリストを定義する
check_prerequisites() {
  # チェック対象コマンドを配列で定義する
  local required_cmds=("docker" "kubectl" "yq")
  # 各コマンドの存在を確認するループ
  for cmd in "${required_cmds[@]}"; do
    # command -v でコマンドの存在を確認する
    if ! command -v "${cmd}" &>/dev/null; then
      # コマンドが見つからない場合はエラーで終了する
      die "Required command not found: ${cmd}"
    fi
  done
  # 全コマンドが存在する場合は成功ログを出力する
  log "All prerequisites satisfied."
}

# ============================================================
# kind クラスタの状態確認: ノード一覧を取得する
# ============================================================
# 対象クラスタのワーカーノード一覧を取得する関数
get_worker_nodes() {
  # kubectl でワーカーロールを持つノード名を取得する
  kubectl get nodes \
    --context "kind-${TARGET_CLUSTER}" \
    --selector='!node-role.kubernetes.io/control-plane' \
    -o jsonpath='{.items[*].metadata.name}' \
    2>/dev/null || echo ""
}

# 指定したノード名に対応する docker コンテナ ID を返す関数
get_container_id() {
  # 第一引数をノード名として受け取る
  local node_name="$1"
  # docker ps でノード名に一致するコンテナ ID を取得する
  docker ps --filter "name=${node_name}" --format "{{.ID}}" | head -1
}

# ============================================================
# drill 実行: zone outage シミュレーションを実施する
# ============================================================
# 単一ノードを停止して Pod 再スケジュールを確認する関数
run_single_node_outage_drill() {
  # 第一引数を topology_class として受け取る
  local topology_class="$1"
  # 第二引数を drill_id として受け取る
  local drill_id="$2"
  # ドリル開始ログを出力する
  log "Starting drill: ${drill_id} (topology_class=${topology_class})"

  # ワーカーノード一覧を取得する
  local workers
  workers=$(get_worker_nodes)
  # ワーカーが見つからない場合はスキップする
  if [[ -z "${workers}" ]]; then
    log "WARN: No worker nodes found for cluster ${TARGET_CLUSTER}, skipping drill."
    return 1
  fi

  # 停止対象ノードを選択する（先頭のワーカーを選択する）
  local target_node
  target_node=$(echo "${workers}" | tr ' ' '\n' | head -1)
  log "Target node for outage: ${target_node}"

  # 対象ノードの docker コンテナ ID を取得する
  local container_id
  container_id=$(get_container_id "${target_node}")
  # コンテナが見つからない場合はエラーを返す
  if [[ -z "${container_id}" ]]; then
    log "WARN: Container not found for node ${target_node}, skipping docker stop."
    return 1
  fi

  # docker stop でノードコンテナを停止して zone outage を模擬する
  log "Stopping container ${container_id} (node: ${target_node})..."
  docker stop "${container_id}" || die "Failed to stop container ${container_id}"

  # Pod 再スケジュールを待機する（kubectl wait を使用する）
  log "Waiting for pod rescheduling (timeout=${DRILL_TIMEOUT}s)..."
  # kubectl wait でデフォルト Namespace の全 Pod が Ready になるまで待機する
  kubectl wait pods \
    --context "kind-${TARGET_CLUSTER}" \
    --all \
    --for=condition=Ready \
    --timeout="${DRILL_TIMEOUT}s" \
    --all-namespaces \
    2>/dev/null || log "WARN: Some pods did not become Ready within timeout."

  # ドリル結果を記録する変数を設定する
  local drill_result="green"
  # 停止したノードを docker start で復旧する
  log "Restoring container ${container_id}..."
  docker start "${container_id}" || {
    # 復旧失敗の場合は警告ログを出力して結果を失敗に更新する
    log "WARN: Failed to restore container ${container_id}"
    drill_result="red"
  }

  # ノード復旧後にクラスタが正常に戻るまで待機する
  log "Waiting for node ${target_node} to become Ready again..."
  kubectl wait node \
    --context "kind-${TARGET_CLUSTER}" \
    "${target_node}" \
    --for=condition=Ready \
    --timeout="${DRILL_TIMEOUT}s" \
    2>/dev/null || log "WARN: Node ${target_node} did not become Ready within timeout."

  # ドリル結果を返す（呼び出し元が lock.yaml 更新に使用する）
  echo "${drill_result}"
}

# ============================================================
# lock.yaml 更新: drill 結果を failover_drill.lock.yaml に書き込む
# ============================================================
# 指定した drill_id の drill_state を更新する関数
update_lock_yaml() {
  # 第一引数を drill_id として受け取る
  local drill_id="$1"
  # 第二引数を新しい drill_state として受け取る
  local new_state="$2"
  # 第三引数を実行時刻として受け取る
  local executed_at="$3"

  # lock.yaml ファイルが存在するか確認する
  if [[ ! -f "${LOCK_FILE}" ]]; then
    die "Lock file not found: ${LOCK_FILE}"
  fi

  # yq を使って指定 drill_id の drill_state を更新する
  yq e \
    "(.drills[] | select(.drill_id == \"${drill_id}\") | .drill_state) = \"${new_state}\"" \
    -i "${LOCK_FILE}"

  # executed_at フィールドを追加・更新する
  yq e \
    "(.drills[] | select(.drill_id == \"${drill_id}\") | .executed_at) = \"${executed_at}\"" \
    -i "${LOCK_FILE}"

  # 更新完了ログを出力する
  log "Lock yaml updated: ${drill_id} -> ${new_state} at ${executed_at}"
}

# ============================================================
# メイン処理: 全 drill を順次実行する
# ============================================================
main() {
  # 前提条件チェックを実行する
  check_prerequisites

  # ドリル開始メッセージを出力する
  log "=== k1s0 topology failover drill start ==="
  # 対象クラスタ名を出力する
  log "Target cluster: ${TARGET_CLUSTER}"

  # drill 実行対象リストを定義する（topology_class : drill_id のペア）
  local -a drills=(
    "v1_single_zone:failover__v1_single_zone"
    "v1_multi_zone_per_cluster:failover__v1_multi_zone_per_cluster"
    "v1_active_passive_dr:failover__v1_active_passive_dr"
    "v1_multi_region_active_passive:failover__v1_multi_region_active_passive"
    "v1_multi_region_active_active:failover__v1_multi_region_active_active"
  )

  # 各 drill を順次実行するループ
  for drill_entry in "${drills[@]}"; do
    # コロン区切りで topology_class と drill_id を分離する
    local topology_class="${drill_entry%%:*}"
    local drill_id="${drill_entry##*:}"

    # ドリル実行開始時刻を記録する
    local start_time
    start_time=$(now_iso)
    log "--- Running drill: ${drill_id} ---"

    # drill を実行し結果を取得する
    local result
    result=$(run_single_node_outage_drill "${topology_class}" "${drill_id}" 2>/dev/null) || result="red"

    # lock.yaml にドリル結果を書き込む
    update_lock_yaml "${drill_id}" "${result}" "${start_time}"
    # ドリル結果サマリを出力する
    log "Drill ${drill_id} completed: ${result}"
  done

  # 全ドリル完了メッセージを出力する
  log "=== k1s0 topology failover drill complete ==="
  # lock.yaml の現在の状態を出力する
  log "Lock file updated: ${LOCK_FILE}"
}

# エントリーポイント: main 関数を呼び出す
main "$@"
