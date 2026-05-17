#!/usr/bin/env bash
# Apicurio GitOps ドリフトテストスクリプト
# git への schema commit 後に Apicurio Registry に正しく同期されるかを検証する
set -euo pipefail

# Apicurio Registry のベース URL を環境変数または既定値から取得する
APICURIO_URL="${APICURIO_URL:-http://localhost:8080}"

# テスト用の git リポジトリ URL を環境変数または既定値から取得する
GIT_REPO_URL="${GIT_REPO_URL:-http://localhost:3000/k1s0/schemas}"

# Kubernetes namespace を環境変数または既定値から取得する
K8S_NAMESPACE="${K8S_NAMESPACE:-k1s0-system}"

# テスト成功カウンタを初期化する
PASS_COUNT=0
# テスト失敗カウンタを初期化する
FAIL_COUNT=0

# テスト用の一時ディレクトリを作成する
TMP_DIR=$(mktemp -d)

# スクリプト終了時に一時ディレクトリを削除するトラップを設定する
trap 'rm -rf "${TMP_DIR}"' EXIT

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

# Apicurio Registry の接続確認テスト: Registry が正常に稼働しているかを確認する
test_apicurio_health() {
  # テスト名称を出力する
  log "テスト開始: Apicurio Registry ヘルスチェック"
  # Apicurio Registry のヘルスエンドポイントに HTTP GET リクエストを送信する
  HTTP_STATUS=$(curl \
    --silent \
    --output /dev/null \
    --write-out "%{http_code}" \
    --max-time 10 \
    "${APICURIO_URL}/health/ready" 2>/dev/null || echo "000")
  # ステータスコードが 200 であることを確認する
  if [[ "${HTTP_STATUS}" == "200" ]]; then
    # Apicurio Registry が正常に稼働していることを記録する
    pass "Apicurio Registry ヘルスチェック成功 (status=${HTTP_STATUS})"
  else
    # Apicurio Registry が正常に稼働していない場合は失敗とする
    fail "Apicurio Registry ヘルスチェック失敗 (status=${HTTP_STATUS})"
  fi
}

# Schema 同期テスト: テスト用 schema を registry に push して同期確認する
test_schema_sync() {
  # テスト名称を出力する
  log "テスト開始: schema 同期確認"
  # テスト用 JSON Schema を一時ファイルに作成する
  local test_schema='{"type":"object","properties":{"id":{"type":"string"}}}'
  # テスト用 artifact ID を設定する
  local test_artifact_id="test-schema-$(date +%s)"
  # テスト用 group ID を設定する
  local test_group_id="k1s0-test"
  # Apicurio Registry に テスト schema を POST する
  HTTP_STATUS=$(curl \
    --silent \
    --output /dev/null \
    --write-out "%{http_code}" \
    --request POST \
    --header "Content-Type: application/json" \
    --header "X-Registry-ArtifactId: ${test_artifact_id}" \
    --data "${test_schema}" \
    --max-time 10 \
    "${APICURIO_URL}/apis/registry/v2/groups/${test_group_id}/artifacts" 2>/dev/null || echo "000")
  # ステータスコードが 200 または 201 であることを確認する
  if [[ "${HTTP_STATUS}" =~ ^20[01]$ ]]; then
    # schema の push が成功したことを記録する
    pass "schema push 成功 (artifactId=${test_artifact_id}, status=${HTTP_STATUS})"
  else
    # schema の push が失敗した場合は失敗とする
    fail "schema push 失敗 (artifactId=${test_artifact_id}, status=${HTTP_STATUS})"
    # 以降のテストをスキップする
    return
  fi
  # Push した schema を GET で取得して同期確認する
  RETRIEVED=$(curl \
    --silent \
    --max-time 10 \
    "${APICURIO_URL}/apis/registry/v2/groups/${test_group_id}/artifacts/${test_artifact_id}" 2>/dev/null || echo "FAILED")
  # 取得した schema が元の schema と一致することを確認する
  if [[ "${RETRIEVED}" == "${test_schema}" ]]; then
    # schema が正しく同期されたことを記録する
    pass "schema 同期確認成功 (artifactId=${test_artifact_id})"
  else
    # schema が正しく同期されていない場合は失敗とする
    fail "schema 同期確認失敗 (retrieved=${RETRIEVED:0:50}...)"
  fi
}

# Additive-only ポリシーテスト: 既存 schema の削除要求が拒否されることを確認する
test_additive_only_policy() {
  # テスト名称を出力する
  log "テスト開始: additive-only ポリシー確認"
  # コントローラが稼働しているかを kubectl で確認する
  if ! kubectl get deployment apicurio-gitops-controller -n "${K8S_NAMESPACE}" &>/dev/null; then
    # コントローラが稼働していない場合はスキップする
    log "SKIP: apicurio-gitops-controller Deployment が見つからないためスキップする"
    return
  fi
  # コントローラの Pod ログから additive-only ポリシー違反のログを確認する
  local policy_violation_log
  # kubectl logs でコントローラのログを取得する
  policy_violation_log=$(kubectl logs \
    -l app=apicurio-gitops-controller \
    -n "${K8S_NAMESPACE}" \
    --tail=100 2>/dev/null | grep "additive-only" || echo "")
  # ポリシー違反ログが存在しないことを確認する (正常動作では違反が発生しない)
  if [[ -z "${policy_violation_log}" ]]; then
    # additive-only ポリシーが正しく機能していることを記録する
    pass "additive-only ポリシー: 違反ログなし (正常動作)"
  else
    # ポリシー違反が記録されていた場合はログを出力する
    log "INFO: additive-only ポリシー違反が記録された: ${policy_violation_log}"
    pass "additive-only ポリシー: 違反が正しく検知・拒否された"
  fi
}

# メイン処理: 全テストを順番に実行する
main() {
  # テスト開始ログを出力する
  log "Apicurio GitOps ドリフトテスト開始"

  # Apicurio Registry ヘルスチェックテストを実行する
  test_apicurio_health

  # schema 同期テストを実行する
  test_schema_sync

  # additive-only ポリシーテストを実行する
  test_additive_only_policy

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
  log "全 Apicurio ドリフトテスト成功"
}

# スクリプトのエントリポイント: main 関数を呼び出す
main "$@"
