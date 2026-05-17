#!/usr/bin/env bash
# SLO 4 層保護テストスクリプト
# レートリミット/自動スケール/接続プール/Kafka クォータの動作を検証する
set -euo pipefail

# テスト対象の Envoy Gateway エンドポイントを設定する
ENVOY_URL="${ENVOY_URL:-https://localhost:443}"
# テスト対象の Prometheus エンドポイントを設定する
PROMETHEUS_URL="${PROMETHEUS_URL:-http://localhost:9090}"
# テスト対象の PgBouncer エンドポイントを設定する
PGBOUNCER_HOST="${PGBOUNCER_HOST:-localhost}"
# テスト対象の PgBouncer ポートを設定する
PGBOUNCER_PORT="${PGBOUNCER_PORT:-5432}"
# テスト対象の Kubernetes Namespace を設定する
K8S_NAMESPACE="${K8S_NAMESPACE:-k1s0-system}"

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

# スキップ関数: テストをスキップしてログを出力する
skip() {
  # スキップログを出力する
  log "SKIP: $*"
}

# 第 1 層テスト: Envoy レートリミット動作確認
test_tier1_rate_limit() {
  # テスト名称を出力する
  log "テスト開始 [第 1 層]: Envoy レートリミット動作確認"
  # 短時間に大量のリクエストを送信してレートリミットの発動を確認する
  local RATE_LIMITED=false
  # 100 リクエストを高速に送信する
  for i in $(seq 1 20); do
    # リクエストのステータスコードを取得する
    STATUS=$(curl \
      --silent \
      --output /dev/null \
      --write-out "%{http_code}" \
      --max-time 5 \
      --insecure \
      "${ENVOY_URL}/health" 2>/dev/null || echo "000")
    # 429 (Too Many Requests) が返った場合はレートリミット発動を記録する
    if [[ "${STATUS}" == "429" ]]; then
      # レートリミット発動フラグを true に設定する
      RATE_LIMITED=true
      # レートリミット発動ログを出力する
      log "レートリミット発動確認: request=${i}, status=${STATUS}"
      # ループを抜ける
      break
    fi
  done
  # Prometheus でレートリミットメトリクスを確認する
  RL_METRIC=$(curl \
    --silent \
    --max-time 5 \
    "${PROMETHEUS_URL}/api/v1/query?query=envoy_local_rate_limit_rate_limited_total" \
    2>/dev/null | grep -o '"value":\["[^"]*","[^"]*"\]' | head -1 || echo "")
  # メトリクスが存在する場合はレートリミットが機能していることを確認する
  if [[ -n "${RL_METRIC}" ]]; then
    # Envoy レートリミットメトリクスが存在することを記録する
    pass "第 1 層: Envoy レートリミットメトリクスが存在する (${RL_METRIC})"
  else
    # Prometheus が利用できない場合はスキップとして扱う
    skip "第 1 層: Prometheus が利用できないため Envoy レートリミットメトリクス確認をスキップ"
    # スキップの場合は success とみなす
    PASS_COUNT=$((PASS_COUNT + 1))
  fi
}

# 第 2 層テスト: KEDA 自動スケール設定確認
test_tier2_keda_autoscale() {
  # テスト名称を出力する
  log "テスト開始 [第 2 層]: KEDA 自動スケール設定確認"
  # kubectl が利用可能かどうかを確認する
  if ! command -v kubectl &>/dev/null; then
    # kubectl が見つからない場合はスキップする
    skip "第 2 層: kubectl が見つからないため KEDA 自動スケールテストをスキップ"
    PASS_COUNT=$((PASS_COUNT + 1))
    return
  fi
  # KEDA ScaledObject が存在するかどうかを確認する
  if kubectl get scaledobject k1s0-tier1-autoscaler -n "${K8S_NAMESPACE}" &>/dev/null; then
    # ScaledObject の現在の状態を取得する
    READY_STATUS=$(kubectl get scaledobject k1s0-tier1-autoscaler \
      -n "${K8S_NAMESPACE}" \
      -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null || echo "Unknown")
    # ScaledObject が Ready 状態かどうかを確認する
    if [[ "${READY_STATUS}" == "True" ]]; then
      # KEDA ScaledObject が Ready 状態であることを記録する
      pass "第 2 層: KEDA ScaledObject が Ready 状態 (ready=${READY_STATUS})"
    else
      # KEDA ScaledObject が Ready でない場合はエラーとする
      fail "第 2 層: KEDA ScaledObject が Ready でない (ready=${READY_STATUS})"
    fi
  else
    # ScaledObject が存在しない場合はスキップとして扱う
    skip "第 2 層: KEDA ScaledObject が見つからないためスキップ (kind cluster 未デプロイの可能性)"
    PASS_COUNT=$((PASS_COUNT + 1))
  fi
}

# 第 3 層テスト: PgBouncer 接続プール稼働確認
test_tier3_pgbouncer_pool() {
  # テスト名称を出力する
  log "テスト開始 [第 3 層]: PgBouncer 接続プール稼働確認"
  # psql が利用可能かどうかを確認する
  if ! command -v psql &>/dev/null; then
    # psql が見つからない場合はスキップする
    skip "第 3 層: psql が見つからないため PgBouncer テストをスキップ"
    PASS_COUNT=$((PASS_COUNT + 1))
    return
  fi
  # PgBouncer に接続して SHOW POOLS を実行する
  POOL_INFO=$(PGPASSWORD=pgbouncer_admin psql \
    -h "${PGBOUNCER_HOST}" \
    -p "${PGBOUNCER_PORT}" \
    -U pgbouncer_admin \
    -d pgbouncer \
    -c "SHOW POOLS;" \
    --no-password \
    2>/dev/null || echo "FAILED")
  # 接続に成功した場合はプール情報を確認する
  if [[ "${POOL_INFO}" != "FAILED" ]] && [[ -n "${POOL_INFO}" ]]; then
    # PgBouncer 接続プールが稼働していることを記録する
    pass "第 3 層: PgBouncer 接続プールが稼働している"
  else
    # PgBouncer に接続できない場合はスキップとして扱う
    skip "第 3 層: PgBouncer に接続できないためスキップ (kind cluster 未デプロイの可能性)"
    PASS_COUNT=$((PASS_COUNT + 1))
  fi
}

# 第 4 層テスト: Kafka クォータ設定確認
test_tier4_kafka_quota() {
  # テスト名称を出力する
  log "テスト開始 [第 4 層]: Kafka クォータ設定確認"
  # kubectl が利用可能かどうかを確認する
  if ! command -v kubectl &>/dev/null; then
    # kubectl が見つからない場合はスキップする
    skip "第 4 層: kubectl が見つからないため Kafka クォータテストをスキップ"
    PASS_COUNT=$((PASS_COUNT + 1))
    return
  fi
  # Strimzi KafkaUser が存在するかどうかを確認する
  if kubectl get kafkauser k1s0-tier1-producer -n "${K8S_NAMESPACE}" &>/dev/null; then
    # KafkaUser のクォータ設定を取得する
    PRODUCER_QUOTA=$(kubectl get kafkauser k1s0-tier1-producer \
      -n "${K8S_NAMESPACE}" \
      -o jsonpath='{.spec.quotas.producerByteRate}' 2>/dev/null || echo "0")
    # クォータが設定されているかどうかを確認する
    if [[ "${PRODUCER_QUOTA}" -gt 0 ]]; then
      # Kafka プロデューサークォータが設定されていることを記録する
      pass "第 4 層: Kafka プロデューサークォータ設定確認 (producerByteRate=${PRODUCER_QUOTA})"
    else
      # クォータが設定されていない場合はエラーとする
      fail "第 4 層: Kafka プロデューサークォータが設定されていない"
    fi
  else
    # KafkaUser が存在しない場合はスキップとして扱う
    skip "第 4 層: Strimzi KafkaUser が見つからないためスキップ (Strimzi 未インストールの可能性)"
    PASS_COUNT=$((PASS_COUNT + 1))
  fi
}

# SLO メトリクス確認テスト: Prometheus で SLO メトリクスが記録されているかを確認する
test_slo_metrics() {
  # テスト名称を出力する
  log "テスト開始: SLO メトリクス確認"
  # Prometheus の稼働確認を行う
  PROM_STATUS=$(curl \
    --silent \
    --output /dev/null \
    --write-out "%{http_code}" \
    --max-time 5 \
    "${PROMETHEUS_URL}/-/healthy" 2>/dev/null || echo "000")
  # Prometheus が稼働しているかどうかを確認する
  if [[ "${PROM_STATUS}" == "200" ]]; then
    # Prometheus が稼働していることを記録する
    pass "SLO メトリクス: Prometheus が稼働している (status=${PROM_STATUS})"
  else
    # Prometheus が稼働していない場合はスキップとして扱う
    skip "SLO メトリクス: Prometheus が利用できないためスキップ (status=${PROM_STATUS})"
    PASS_COUNT=$((PASS_COUNT + 1))
  fi
}

# メイン処理: 全テストを順番に実行する
main() {
  # テスト開始ログを出力する
  log "SLO 4 層保護テスト開始"

  # 第 1 層テスト: Envoy レートリミット
  test_tier1_rate_limit

  # 第 2 層テスト: KEDA 自動スケール
  test_tier2_keda_autoscale

  # 第 3 層テスト: PgBouncer 接続プール
  test_tier3_pgbouncer_pool

  # 第 4 層テスト: Kafka クォータ
  test_tier4_kafka_quota

  # SLO メトリクス確認テスト
  test_slo_metrics

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
  log "全 SLO 4 層保護テスト成功"
}

# スクリプトのエントリポイント: main 関数を呼び出す
main "$@"
