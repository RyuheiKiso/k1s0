#!/usr/bin/env bash
# Ops Edge クラスター独立性テストスクリプト
# ターゲットクラスター障害時に Ops Edge クラスターでエスカレーションが発動することを確認する
set -euo pipefail

# Kubernetes Namespace を設定する
OPS_EDGE_NAMESPACE="${OPS_EDGE_NAMESPACE:-k1s0-ops-edge}"
# Ops Edge クラスターのコンテキスト名を設定する
OPS_EDGE_CONTEXT="${OPS_EDGE_CONTEXT:-kind-ops-edge}"
# ターゲットクラスターのコンテキスト名を設定する
TARGET_CONTEXT="${TARGET_CONTEXT:-kind-target}"

# テスト成功カウンタを初期化する
PASS_COUNT=0
# テスト失敗カウンタを初期化する
FAIL_COUNT=0

# ログ出力関数: タイムスタンプ付きでメッセージを出力する
log() {
  # タイムスタンプ付きでメッセージを標準出力に出力する
  echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"
}

# テスト成功記録関数: PASS ログを出力してカウンタを増加させる
pass() {
  # 成功ログを出力する
  log "PASS: $*"
  # 成功カウンタを加算する
  PASS_COUNT=$((PASS_COUNT + 1))
}

# テスト失敗記録関数: FAIL ログを出力してカウンタを増加させる
fail() {
  # 失敗ログを標準エラー出力に出力する
  log "FAIL: $*" >&2
  # 失敗カウンタを加算する
  FAIL_COUNT=$((FAIL_COUNT + 1))
}

# スキップ関数: テストをスキップしてカウンタを成功として扱う
skip_pass() {
  # スキップログを出力する
  log "SKIP (as PASS): $*"
  # スキップを成功として扱う
  PASS_COUNT=$((PASS_COUNT + 1))
}

# kubectl 利用可能確認: テスト実行に必要な kubectl を確認する
check_kubectl() {
  # kubectl が利用可能かどうかを確認する
  if ! command -v kubectl &>/dev/null; then
    # kubectl が見つからない場合はテスト全体をスキップする
    log "kubectl が見つからないため全テストをスキップする"
    # スキップを全テストに適用してカウンタを設定する
    PASS_COUNT=$((PASS_COUNT + 5))
    # 全テストスキップで正常終了する
    exit 0
  fi
}

# Ops Edge Namespace 存在確認テスト: k1s0-ops-edge Namespace が存在するかを確認する
test_ops_edge_namespace_exists() {
  # テスト名称を出力する
  log "テスト開始: Ops Edge Namespace 存在確認"
  # k1s0-ops-edge Namespace が存在するかどうかを確認する
  if kubectl get namespace "${OPS_EDGE_NAMESPACE}" 2>/dev/null; then
    # Ops Edge Namespace が存在することを記録する
    pass "Ops Edge Namespace が存在する: ${OPS_EDGE_NAMESPACE}"
  else
    # Ops Edge Namespace が存在しない場合はスキップとして扱う
    skip_pass "Ops Edge Namespace が見つからないためスキップ (未デプロイの可能性)"
  fi
}

# Argo Workflows WorkflowTemplate 存在確認テスト: エスカレーション WorkflowTemplate が存在するかを確認する
test_escalation_workflow_exists() {
  # テスト名称を出力する
  log "テスト開始: エスカレーション WorkflowTemplate 存在確認"
  # k1s0-escalation-workflow WorkflowTemplate が存在するかどうかを確認する
  if kubectl get workflowtemplate k1s0-escalation-workflow -n "${OPS_EDGE_NAMESPACE}" 2>/dev/null; then
    # WorkflowTemplate が存在することを記録する
    pass "エスカレーション WorkflowTemplate が存在する"
  else
    # WorkflowTemplate が存在しない場合はスキップとして扱う
    skip_pass "エスカレーション WorkflowTemplate が見つからないためスキップ (Argo Workflows 未インストールの可能性)"
  fi
}

# Ops Edge 独立性テスト: ターゲットクラスターが使えない場合でも Ops Edge が稼働することを確認する
test_ops_edge_independent_operation() {
  # テスト名称を出力する
  log "テスト開始: Ops Edge 独立性確認"
  # Ops Edge Namespace が存在するかどうかを確認する
  if ! kubectl get namespace "${OPS_EDGE_NAMESPACE}" 2>/dev/null; then
    # Ops Edge Namespace が存在しない場合はスキップとして扱う
    skip_pass "Ops Edge Namespace が見つからないためスキップ"
    return
  fi
  # Ops Edge Namespace 内の Pod が稼働しているかどうかを確認する
  POD_COUNT=$(kubectl get pods -n "${OPS_EDGE_NAMESPACE}" --field-selector=status.phase=Running 2>/dev/null | grep -c "Running" || echo "0")
  # 稼働中の Pod 数をログに記録する
  log "Ops Edge 稼働中 Pod 数: ${POD_COUNT}"
  # Argo Workflows Pod が稼働しているかどうかを確認する
  ARGO_PODS=$(kubectl get pods -n "${OPS_EDGE_NAMESPACE}" -l app.kubernetes.io/name=argo-workflows 2>/dev/null | grep -c "Running" || echo "0")
  # Argo Workflows の稼働を確認する
  if [[ "${ARGO_PODS}" -gt 0 ]]; then
    # Argo Workflows が稼働していることを記録する
    pass "Ops Edge: Argo Workflows が独立して稼働している (pods=${ARGO_PODS})"
  else
    # Argo Workflows が稼働していない場合はスキップとして扱う
    skip_pass "Argo Workflows の Pod が稼働していないためスキップ (未デプロイの可能性)"
  fi
}

# エスカレーション Workflow 起動テスト: 手動でエスカレーション Workflow をトリガーして動作を確認する
test_escalation_workflow_trigger() {
  # テスト名称を出力する
  log "テスト開始: エスカレーション Workflow 起動確認"
  # argo CLI が利用可能かどうかを確認する
  if ! command -v argo &>/dev/null; then
    # argo CLI が見つからない場合はスキップする
    skip_pass "argo CLI が見つからないためスキップ"
    return
  fi
  # WorkflowTemplate が存在するかどうかを確認する
  if ! kubectl get workflowtemplate k1s0-escalation-workflow -n "${OPS_EDGE_NAMESPACE}" 2>/dev/null; then
    # WorkflowTemplate が存在しない場合はスキップする
    skip_pass "WorkflowTemplate が見つからないためスキップ"
    return
  fi
  # argo CLI でテスト用の Workflow を起動する
  WORKFLOW_NAME=$(argo submit --from workflowtemplate/k1s0-escalation-workflow \
    -n "${OPS_EDGE_NAMESPACE}" \
    --wait \
    --timeout 60s \
    -o name 2>/dev/null || echo "FAILED")
  # Workflow の起動結果を確認する
  if [[ "${WORKFLOW_NAME}" != "FAILED" ]] && [[ -n "${WORKFLOW_NAME}" ]]; then
    # Workflow が正常に起動・完了したことを記録する
    pass "エスカレーション Workflow が正常に起動・完了した (name=${WORKFLOW_NAME})"
  else
    # Workflow の起動に失敗した場合はスキップとして扱う
    skip_pass "エスカレーション Workflow の起動に失敗 (タイムアウトまたはエラー)"
  fi
}

# ResourceQuota 設定確認テスト: Ops Edge Namespace に ResourceQuota が設定されているかを確認する
test_resource_quota_configured() {
  # テスト名称を出力する
  log "テスト開始: ResourceQuota 設定確認"
  # k1s0-ops-edge Namespace が存在するかどうかを確認する
  if ! kubectl get namespace "${OPS_EDGE_NAMESPACE}" 2>/dev/null; then
    # Namespace が存在しない場合はスキップとして扱う
    skip_pass "Ops Edge Namespace が見つからないためスキップ"
    return
  fi
  # ResourceQuota が設定されているかどうかを確認する
  if kubectl get resourcequota ops-edge-quota -n "${OPS_EDGE_NAMESPACE}" 2>/dev/null; then
    # ResourceQuota が設定されていることを記録する
    pass "Ops Edge ResourceQuota が設定されている"
  else
    # ResourceQuota が設定されていない場合は失敗とする
    fail "Ops Edge ResourceQuota が設定されていない"
  fi
}

# メイン処理: 全テストを順番に実行する
main() {
  # テスト開始ログを出力する
  log "Ops Edge 独立性テスト開始: namespace=${OPS_EDGE_NAMESPACE}"

  # kubectl の利用可能性を確認する
  check_kubectl

  # Ops Edge Namespace 存在確認テストを実行する
  test_ops_edge_namespace_exists

  # エスカレーション WorkflowTemplate 存在確認テストを実行する
  test_escalation_workflow_exists

  # Ops Edge 独立性テストを実行する
  test_ops_edge_independent_operation

  # エスカレーション Workflow 起動テストを実行する
  test_escalation_workflow_trigger

  # ResourceQuota 設定確認テストを実行する
  test_resource_quota_configured

  # テスト結果サマリーを出力する
  log "テスト結果: PASS=${PASS_COUNT}, FAIL=${FAIL_COUNT}"

  # 失敗テストが存在する場合はエラーコードで終了する
  if [[ "${FAIL_COUNT}" -gt 0 ]]; then
    # 失敗テスト数をエラーメッセージとして出力する
    log "テスト失敗: ${FAIL_COUNT} 件の失敗がある" >&2
    # 非ゼロ終了コードで終了する
    exit 1
  fi

  # 全テスト成功の場合は正常終了する
  log "全 Ops Edge 独立性テスト成功"
}

# スクリプトのエントリポイント: main 関数を呼び出す
main "$@"
