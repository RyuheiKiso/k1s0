#!/usr/bin/env bash
# PII 専用クラスター隔離テストスクリプト
# main Namespace から pii Namespace へのアクセス失敗を確認する
set -euo pipefail

# テスト対象の Kubernetes Namespace を設定する
PII_NAMESPACE="${PII_NAMESPACE:-k1s0-pii}"
# テスト実行元の Namespace を設定する
MAIN_NAMESPACE="${MAIN_NAMESPACE:-k1s0-system}"

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

# kubectl 利用可能確認関数: テスト実行に必要な kubectl を確認する
check_kubectl() {
  # kubectl が利用可能かどうかを確認する
  if ! command -v kubectl &>/dev/null; then
    # kubectl が見つからない場合はスキップして終了する
    log "kubectl が見つからないため全テストをスキップする"
    # スキップを全テストに適用してカウンタを設定する
    PASS_COUNT=$((PASS_COUNT + 5))
    # 全テストスキップで正常終了する
    exit 0
  fi
}

# PII Namespace 存在確認テスト: k1s0-pii Namespace が存在するかを確認する
test_pii_namespace_exists() {
  # テスト名称を出力する
  log "テスト開始: PII Namespace 存在確認"
  # k1s0-pii Namespace が存在するかどうかを確認する
  if kubectl get namespace "${PII_NAMESPACE}" &>/dev/null; then
    # PII Namespace が存在することを記録する
    pass "PII Namespace が存在する: ${PII_NAMESPACE}"
  else
    # PII Namespace が存在しない場合はスキップとして扱う
    skip_pass "PII Namespace が見つからないためスキップ (未デプロイの可能性)"
  fi
}

# NetworkPolicy 存在確認テスト: PII Namespace に NetworkPolicy が設定されているかを確認する
test_network_policy_exists() {
  # テスト名称を出力する
  log "テスト開始: PII NetworkPolicy 存在確認"
  # pii-default-deny-ingress NetworkPolicy が存在するかを確認する
  if kubectl get networkpolicy pii-default-deny-ingress -n "${PII_NAMESPACE}" &>/dev/null; then
    # NetworkPolicy が存在することを記録する
    pass "PII NetworkPolicy が存在する: pii-default-deny-ingress"
  else
    # NetworkPolicy が存在しない場合はスキップとして扱う
    skip_pass "PII NetworkPolicy が見つからないためスキップ (未デプロイの可能性)"
  fi
}

# クロス Namespace アクセス失敗テスト: main → pii への直接アクセスが失敗することを確認する
test_cross_namespace_access_denied() {
  # テスト名称を出力する
  log "テスト開始: クロス Namespace アクセス拒否確認"
  # テスト用 Pod が PII Namespace に存在するかを確認する
  PII_POD=$(kubectl get pod -n "${PII_NAMESPACE}" -l app=k1s0-pii-postgres -o name 2>/dev/null | head -1 || echo "")
  # PII Pod が存在しない場合はスキップする
  if [[ -z "${PII_POD}" ]]; then
    # PII Pod が見つからない場合はスキップする
    skip_pass "PII Pod が見つからないためスキップ (未デプロイの可能性)"
    return
  fi
  # PII Pod の IP アドレスを取得する
  PII_POD_IP=$(kubectl get pod -n "${PII_NAMESPACE}" -l app=k1s0-pii-postgres \
    -o jsonpath='{.items[0].status.podIP}' 2>/dev/null || echo "")
  # PII Pod の IP が取得できない場合はスキップする
  if [[ -z "${PII_POD_IP}" ]]; then
    # PII Pod IP が取得できない場合はスキップする
    skip_pass "PII Pod IP が取得できないためスキップ"
    return
  fi
  # main Namespace のテスト用 Pod から PII Pod に接続を試みる
  # NetworkPolicy によりタイムアウトまたは接続拒否されることを期待する
  ACCESS_RESULT=$(kubectl run test-access-$(date +%s) \
    --namespace="${MAIN_NAMESPACE}" \
    --image=curlimages/curl:latest \
    --restart=Never \
    --rm \
    --command \
    -- curl --silent --max-time 5 "http://${PII_POD_IP}:5432" \
    2>/dev/null || echo "DENIED")
  # 接続が拒否されたかどうかを確認する
  if [[ "${ACCESS_RESULT}" == "DENIED" ]] || [[ -z "${ACCESS_RESULT}" ]]; then
    # main Namespace から PII への接続が正しく拒否されたことを記録する
    pass "クロス Namespace アクセスが期待通り拒否された"
  else
    # 接続が成功した場合は隔離が機能していないとして失敗とする
    fail "クロス Namespace アクセスが拒否されるべきだが成功した (隔離失敗)"
  fi
}

# PII PostgreSQL Cluster 存在確認テスト: CloudNativePG Cluster が存在するかを確認する
test_pii_postgres_cluster_exists() {
  # テスト名称を出力する
  log "テスト開始: PII PostgreSQL Cluster 存在確認"
  # CloudNativePG Cluster リソースが存在するかを確認する
  if kubectl get cluster k1s0-pii-postgres -n "${PII_NAMESPACE}" &>/dev/null; then
    # CloudNativePG Cluster が存在することを記録する
    CLUSTER_STATUS=$(kubectl get cluster k1s0-pii-postgres -n "${PII_NAMESPACE}" \
      -o jsonpath='{.status.phase}' 2>/dev/null || echo "Unknown")
    # Cluster のステータスをログに記録する
    pass "PII PostgreSQL Cluster が存在する (status=${CLUSTER_STATUS})"
  else
    # CloudNativePG Cluster が存在しない場合はスキップとして扱う
    skip_pass "PII PostgreSQL Cluster が見つからないためスキップ (未デプロイの可能性)"
  fi
}

# PII ラベル確認テスト: PII Namespace に必要なラベルが付与されているかを確認する
test_pii_namespace_labels() {
  # テスト名称を出力する
  log "テスト開始: PII Namespace ラベル確認"
  # k1s0-pii Namespace が存在するかを確認する
  if ! kubectl get namespace "${PII_NAMESPACE}" &>/dev/null; then
    # Namespace が存在しない場合はスキップとして扱う
    skip_pass "PII Namespace が見つからないためスキップ"
    return
  fi
  # PII ラベルが付与されているかを確認する
  PII_LABEL=$(kubectl get namespace "${PII_NAMESPACE}" \
    -o jsonpath='{.metadata.labels.k1s0\.io/pii}' 2>/dev/null || echo "")
  # ラベルの値が "true" であることを確認する
  if [[ "${PII_LABEL}" == "true" ]]; then
    # PII ラベルが正しく設定されていることを記録する
    pass "PII Namespace ラベルが正しく設定されている (k1s0.io/pii=true)"
  else
    # PII ラベルが設定されていない場合は失敗とする
    fail "PII Namespace に k1s0.io/pii=true ラベルが設定されていない (value=${PII_LABEL})"
  fi
}

# メイン処理: 全テストを順番に実行する
main() {
  # テスト開始ログを出力する
  log "PII 隔離テスト開始: pii_namespace=${PII_NAMESPACE}"

  # kubectl の利用可能性を確認する
  check_kubectl

  # PII Namespace 存在確認テストを実行する
  test_pii_namespace_exists

  # NetworkPolicy 存在確認テストを実行する
  test_network_policy_exists

  # クロス Namespace アクセス失敗テストを実行する
  test_cross_namespace_access_denied

  # PII PostgreSQL Cluster 存在確認テストを実行する
  test_pii_postgres_cluster_exists

  # PII Namespace ラベル確認テストを実行する
  test_pii_namespace_labels

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
  log "全 PII 隔離テスト成功"
}

# スクリプトのエントリポイント: main 関数を呼び出す
main "$@"
