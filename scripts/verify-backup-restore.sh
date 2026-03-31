#!/usr/bin/env bash
# scripts/verify-backup-restore.sh
# staging 環境のバックアップファイルを取得し、一時 Pod でリストアを検証するスクリプト。
# GitHub Actions の backup-verification.yaml から呼び出される。
# 終了コード: 0=全コンポーネント成功, 1=いずれかのコンポーネントで失敗
set -euo pipefail

# --- 設定 ---
# ターゲット環境（staging のみ対象とし、本番の誤操作を防止する）
NAMESPACE="${NAMESPACE:-k1s0-system}"
KUBECONFIG="${KUBECONFIG:-${HOME}/.kube/config}"
BACKUP_PVC="backup-pvc"
VERIFY_NAMESPACE="${VERIFY_NAMESPACE:-k1s0-backup-verify}"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RESULTS_FILE="/tmp/backup-verify-results-${TIMESTAMP}.txt"

# 検証対象コンポーネント（スペース区切りで指定可能）
COMPONENTS="${COMPONENTS:-postgres vault}"

# --- ユーティリティ関数 ---

# ログ出力（タイムスタンプ付き）
log() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

# 成功を記録する
record_success() {
  local component="$1"
  local detail="${2:-}"
  echo "✅ PASS: ${component} — ${detail}" | tee -a "${RESULTS_FILE}"
}

# 失敗を記録する（スクリプトは継続し、最後に終了コードで報告）
record_failure() {
  local component="$1"
  local detail="${2:-}"
  echo "❌ FAIL: ${component} — ${detail}" | tee -a "${RESULTS_FILE}"
  FAILED=1
}

# 一時 Pod を起動して完了を待ち、後始末する
run_verify_pod() {
  local pod_name="$1"
  local image="$2"
  local command="$3"

  # 既存の同名 Pod を削除してから作成する
  kubectl delete pod "${pod_name}" -n "${VERIFY_NAMESPACE}" --ignore-not-found=true

  # K8S-SCRIPT-001 監査対応: インライン JSON を外部テンプレートファイル（verify-pod-spec.json.tpl）に分離し、
  # envsubst で変数展開する。インライン JSON の特殊文字エスケープによるメンテナンス性の低下を解消する。
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  local spec_tpl="${script_dir}/verify-pod-spec.json.tpl"

  if [ ! -f "${spec_tpl}" ]; then
    log "ERROR: Pod spec template not found: ${spec_tpl}"
    return 1
  fi

  # テンプレート変数をエクスポートして envsubst で展開する
  export POD_NAME="${pod_name}"
  export IMAGE="${image}"
  export COMMAND="${command}"
  # BACKUP_PVC は上位スコープで定義済み（envsubst 用に export）
  export BACKUP_PVC
  local overrides
  overrides=$(envsubst < "${spec_tpl}")

  kubectl run "${pod_name}" \
    --image="${image}" \
    --restart=Never \
    --namespace="${VERIFY_NAMESPACE}" \
    --overrides="${overrides}"

  # Pod 完了を最大 300 秒待機する
  if kubectl wait pod "${pod_name}" \
    --for=condition=Succeeded \
    --timeout=300s \
    -n "${VERIFY_NAMESPACE}" 2>/dev/null; then
    kubectl logs "${pod_name}" -n "${VERIFY_NAMESPACE}"
    kubectl delete pod "${pod_name}" -n "${VERIFY_NAMESPACE}" --ignore-not-found=true
    return 0
  else
    kubectl logs "${pod_name}" -n "${VERIFY_NAMESPACE}" || true
    kubectl delete pod "${pod_name}" -n "${VERIFY_NAMESPACE}" --ignore-not-found=true
    return 1
  fi
}

# --- 検証用 Namespace の準備 ---
prepare_verify_namespace() {
  log "検証用 Namespace ${VERIFY_NAMESPACE} を準備中..."
  kubectl create namespace "${VERIFY_NAMESPACE}" --dry-run=client -o yaml | kubectl apply -f -
}

# --- PostgreSQL バックアップ検証 ---
verify_postgres() {
  log "=== PostgreSQL バックアップ検証開始 ==="

  # 最新の dump ファイルを特定する
  local latest_dump
  latest_dump=$(kubectl exec -n "${NAMESPACE}" deploy/postgres \
    -- bash -c "ls -t /backup/postgres/*.dump 2>/dev/null | head -1" 2>/dev/null || echo "")

  if [ -z "${latest_dump}" ]; then
    record_failure "postgres" "バックアップファイルが見つかりません (/backup/postgres/*.dump)"
    return
  fi

  log "検証対象: ${latest_dump}"

  # pg_restore --list で dump ファイルの整合性チェック（実際にリストアせず構造のみ確認）
  local verify_cmd
  verify_cmd="pg_restore --list '${latest_dump}' > /dev/null && echo 'dump_integrity_ok'"

  if run_verify_pod "pg-backup-verify-${TIMESTAMP}" "postgres:17-alpine" "${verify_cmd}"; then
    record_success "postgres" "dump ファイル整合性OK: ${latest_dump}"
  else
    record_failure "postgres" "dump ファイル検証失敗: ${latest_dump}"
  fi
}

# --- Vault バックアップ検証 ---
verify_vault() {
  log "=== Vault バックアップ検証開始 ==="

  # 最新のスナップショットを特定する
  local latest_snap
  latest_snap=$(kubectl exec -n "${NAMESPACE}" deploy/vault \
    -- sh -c "ls -t /backup/vault/*.snap 2>/dev/null | head -1" 2>/dev/null || echo "")

  if [ -z "${latest_snap}" ]; then
    record_failure "vault" "スナップショットファイルが見つかりません (/backup/vault/*.snap)"
    return
  fi

  log "検証対象: ${latest_snap}"

  # スナップショットファイルのサイズ確認（0 バイトは異常）
  local snap_size
  snap_size=$(kubectl exec -n "${NAMESPACE}" deploy/vault \
    -- sh -c "stat -c%s '${latest_snap}' 2>/dev/null || echo 0")

  if [ "${snap_size:-0}" -gt 1024 ]; then
    record_success "vault" "スナップショット存在確認OK: ${latest_snap} (${snap_size} bytes)"
  else
    record_failure "vault" "スナップショットが空または存在しません: ${latest_snap} (${snap_size:-0} bytes)"
  fi
}

# --- etcd バックアップ検証 ---
verify_etcd() {
  log "=== etcd バックアップ検証開始 ==="

  # etcd バックアップは PVC ではなく /tmp に保存される（etcd-backup-cronjob.yaml 参照）
  # ここでは CronJob の最終実行ステータスを確認する
  local last_job_status
  last_job_status=$(kubectl get cronjob etcd-backup -n "${NAMESPACE}" \
    -o jsonpath='{.status.lastSuccessfulTime}' 2>/dev/null || echo "")

  if [ -n "${last_job_status}" ]; then
    record_success "etcd" "最終成功時刻: ${last_job_status}"
  else
    record_failure "etcd" "etcd-backup CronJob の最終成功時刻が記録されていません"
  fi
}

# --- Harbor バックアップ検証 ---
verify_harbor() {
  log "=== Harbor バックアップ検証開始 ==="

  local last_job_status
  last_job_status=$(kubectl get cronjob harbor-backup -n "${NAMESPACE}" \
    -o jsonpath='{.status.lastSuccessfulTime}' 2>/dev/null || echo "")

  if [ -n "${last_job_status}" ]; then
    record_success "harbor" "最終成功時刻: ${last_job_status}"
  else
    record_failure "harbor" "harbor-backup CronJob の最終成功時刻が記録されていません"
  fi
}

# --- メイン処理 ---
main() {
  log "バックアップ検証開始 (対象: ${COMPONENTS})"
  FAILED=0

  prepare_verify_namespace

  # COMPONENTS に含まれるコンポーネントのみ検証する
  for component in ${COMPONENTS}; do
    case "${component}" in
      postgres) verify_postgres ;;
      vault)    verify_vault ;;
      etcd)     verify_etcd ;;
      harbor)   verify_harbor ;;
      *)
        log "不明なコンポーネント: ${component} — スキップします"
        ;;
    esac
  done

  log "=== 検証結果サマリ ==="
  cat "${RESULTS_FILE}"

  if [ "${FAILED}" -eq 1 ]; then
    log "一部のバックアップ検証が失敗しました。上記サマリを確認してください。"
    exit 1
  else
    log "全バックアップ検証が成功しました。"
    exit 0
  fi
}

main "$@"
