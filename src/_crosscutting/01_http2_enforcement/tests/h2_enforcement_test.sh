#!/usr/bin/env bash
# HTTP/2 強制テストスクリプト: curl を用いて HTTP/1.1 reject と HTTP/2 accept を検証する
# 業務リスナーが HTTP/1.1 を拒否し HTTP/2 のみを受け付けることを確認する
set -euo pipefail

# テスト対象のベース URL を環境変数または既定値から取得する
BASE_URL="${BASE_URL:-https://localhost:443}"

# テスト用の CA 証明書パスを環境変数または既定値から取得する
CA_CERT="${CA_CERT:-/etc/ssl/certs/k1s0-ca.crt}"

# テスト結果集計用のカウンタを初期化する
PASS_COUNT=0
# 失敗テスト数カウンタを初期化する
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

# HTTP/1.1 拒否テスト: --http1.1 オプションで接続した場合に 4xx/5xx が返ることを確認する
test_http11_rejected() {
  # テスト名称を出力する
  log "テスト開始: HTTP/1.1 接続拒否確認"
  # curl で HTTP/1.1 接続を試行して HTTP ステータスコードを取得する
  HTTP_STATUS=$(curl \
    --http1.1 \
    --silent \
    --output /dev/null \
    --write-out "%{http_code}" \
    --cacert "${CA_CERT}" \
    --max-time 10 \
    "${BASE_URL}/health" 2>/dev/null || echo "000")
  # ステータスコードが 4xx または 5xx であることを確認する
  if [[ "${HTTP_STATUS}" =~ ^[45][0-9][0-9]$ ]] || [[ "${HTTP_STATUS}" == "000" ]]; then
    # HTTP/1.1 が正しく拒否されたことを記録する
    pass "HTTP/1.1 接続が期待通り拒否された (status=${HTTP_STATUS})"
  else
    # HTTP/1.1 が誤って受け付けられた場合は失敗とする
    fail "HTTP/1.1 接続が拒否されるべきだが受け付けられた (status=${HTTP_STATUS})"
  fi
}

# HTTP/2 受容テスト: --http2 オプションで接続した場合に 2xx が返ることを確認する
test_http2_accepted() {
  # テスト名称を出力する
  log "テスト開始: HTTP/2 接続受容確認"
  # curl で HTTP/2 接続を試行して HTTP ステータスコードを取得する
  HTTP_STATUS=$(curl \
    --http2 \
    --silent \
    --output /dev/null \
    --write-out "%{http_code}" \
    --cacert "${CA_CERT}" \
    --max-time 10 \
    "${BASE_URL}/health" 2>/dev/null || echo "000")
  # ステータスコードが 2xx であることを確認する
  if [[ "${HTTP_STATUS}" =~ ^2[0-9][0-9]$ ]]; then
    # HTTP/2 が正しく受け付けられたことを記録する
    pass "HTTP/2 接続が期待通り受け付けられた (status=${HTTP_STATUS})"
  else
    # HTTP/2 が誤って拒否された場合は失敗とする
    fail "HTTP/2 接続が受け付けられるべきだが失敗した (status=${HTTP_STATUS})"
  fi
}

# HTTP/2 プロトコルバージョン確認テスト: 実際に HTTP/2 プロトコルで通信していることを確認する
test_http2_protocol_version() {
  # テスト名称を出力する
  log "テスト開始: HTTP/2 プロトコルバージョン確認"
  # curl の詳細ログから HTTP/2 プロトコルバージョンを取得する
  PROTO=$(curl \
    --http2 \
    --silent \
    --output /dev/null \
    --write-out "%{http_version}" \
    --cacert "${CA_CERT}" \
    --max-time 10 \
    "${BASE_URL}/health" 2>/dev/null || echo "unknown")
  # プロトコルバージョンが 2 であることを確認する
  if [[ "${PROTO}" == "2" ]]; then
    # HTTP/2 プロトコルで通信していることを記録する
    pass "HTTP/2 プロトコルバージョンが確認された (proto=${PROTO})"
  else
    # 期待するプロトコルバージョンでない場合は失敗とする
    fail "HTTP/2 プロトコルバージョンが期待値と異なる (proto=${PROTO})"
  fi
}

# レガシーリスナーの HTTP/1.1 受容テスト: ポート 8443 で HTTP/1.1 が受け付けられることを確認する
test_legacy_http11_accepted() {
  # テスト名称を出力する
  log "テスト開始: レガシーリスナー HTTP/1.1 受容確認"
  # レガシーリスナーの URL を組み立てる (ポート 8443)
  LEGACY_URL="${BASE_URL/443/8443}"
  # curl で HTTP/1.1 接続を試行して HTTP ステータスコードを取得する
  HTTP_STATUS=$(curl \
    --http1.1 \
    --silent \
    --output /dev/null \
    --write-out "%{http_code}" \
    --cacert "${CA_CERT}" \
    --max-time 10 \
    "${LEGACY_URL}/health" 2>/dev/null || echo "000")
  # ステータスコードが 2xx であることを確認する
  if [[ "${HTTP_STATUS}" =~ ^2[0-9][0-9]$ ]]; then
    # レガシーリスナーが HTTP/1.1 を正しく受け付けたことを記録する
    pass "レガシーリスナーが HTTP/1.1 接続を受け付けた (status=${HTTP_STATUS})"
  else
    # レガシーリスナーが HTTP/1.1 を拒否した場合は失敗とする
    fail "レガシーリスナーが HTTP/1.1 接続を受け付けるべきだが失敗した (status=${HTTP_STATUS})"
  fi
}

# 全テストを実行する
log "HTTP/2 強制テスト開始"

# HTTP/1.1 拒否テストを実行する
test_http11_rejected

# HTTP/2 受容テストを実行する
test_http2_accepted

# HTTP/2 プロトコルバージョン確認テストを実行する
test_http2_protocol_version

# レガシーリスナー HTTP/1.1 受容テストを実行する
test_legacy_http11_accepted

# テスト結果サマリーを出力する
log "テスト結果: PASS=${PASS_COUNT}, FAIL=${FAIL_COUNT}"

# 失敗テストが存在する場合はエラーコードで終了する
if [[ "${FAIL_COUNT}" -gt 0 ]]; then
  # 失敗テスト数をエラーメッセージとして出力する
  log "テスト失敗: ${FAIL_COUNT} 件の失敗がある" >&2
  # 非ゼロ終了コードでスクリプトを終了する
  exit 1
fi

# 全テスト成功の場合は正常終了する
log "全テスト成功"
