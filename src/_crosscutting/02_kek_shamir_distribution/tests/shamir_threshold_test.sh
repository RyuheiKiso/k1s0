#!/usr/bin/env bash
# Shamir Secret Sharing 閾値テストスクリプト
# M-1 シェアでは復元失敗、M シェアでは復元成功することを検証する
set -euo pipefail

# 復元閾値 (M=3): 3 シェア揃わないと復元できない設定を使用する
M=3
# シェア総数 (N=5): 合計 5 シェアに分割する設定を使用する
N=5

# テスト用の一時ディレクトリを作成する
TMP_DIR=$(mktemp -d)

# スクリプト終了時に一時ディレクトリを削除するトラップを設定する
trap 'rm -rf "${TMP_DIR}"' EXIT

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

# ssss ツールの存在確認: テスト実行に必要な ssss コマンドを確認する
check_ssss() {
  # ssss-split コマンドの存在を確認する
  if ! command -v ssss-split &>/dev/null || ! command -v ssss-combine &>/dev/null; then
    # ssss ツールが見つからない場合はスキップメッセージを出力する
    log "SKIP: ssss ツールが見つからないためテストをスキップする"
    # 正常終了する (CI 環境でのスキップを許容する)
    exit 0
  fi
}

# テスト用シークレットの生成: 32 バイトのランダムなシークレットを生成する
generate_test_secret() {
  # OpenSSL で 32 バイトのランダムな 16 進数文字列を生成する
  openssl rand -hex 32
}

# Shamir 分割実行関数: シークレットを M-of-N で分割してファイルに保存する
split_secret() {
  # 引数: 分割するシークレット文字列
  local secret="$1"
  # シェアを配列に格納する (ssss-split でシークレットを分割する)
  mapfile -t shares < <(echo "${secret}" | ssss-split -t "${M}" -n "${N}" -q 2>/dev/null)
  # シェアをファイルに保存する
  for i in "${!shares[@]}"; do
    # シェアファイルのパスを組み立てる
    echo "${shares[$i]}" > "${TMP_DIR}/share_$((i+1)).txt"
  done
  # 分割完了ログを出力する
  log "シークレット分割完了: ${#shares[@]} シェアを生成した"
}

# M-1 シェアでの復元失敗テスト: 閾値未満のシェアでは復元できないことを確認する
test_m_minus_1_shares_fails() {
  # テスト名称を出力する
  log "テスト開始: M-1=${$((M-1))} シェアでの復元失敗確認"
  # M-1 シェアを結合して復元を試みる
  local m_minus_1=$((M - 1))
  # M-1 シェアを一時ファイルにまとめる
  local combined_shares=""
  # M-1 個のシェアを結合する
  for i in $(seq 1 "${m_minus_1}"); do
    # シェアファイルの内容を取得する
    combined_shares="${combined_shares}$(cat "${TMP_DIR}/share_${i}.txt")"$'\n'
  done
  # ssss-combine で M-1 シェアを使って復元を試みる (失敗が期待される)
  local result
  # 復元コマンドの実行結果を取得する (エラーを無視して exit code のみ確認する)
  if echo "${combined_shares}" | ssss-combine -t "${M}" -q 2>/dev/null; then
    # 復元が成功した場合は失敗とする (閾値未満なのに復元できてしまった)
    fail "M-1 シェアで復元が成功してしまった (セキュリティ上の問題)"
  else
    # 復元が失敗した場合は期待通りの動作とする
    pass "M-1 シェアでの復元が期待通り失敗した"
  fi
}

# M シェアでの復元成功テスト: 閾値ちょうどのシェアで復元できることを確認する
test_m_shares_succeeds() {
  # テスト名称を出力する
  log "テスト開始: M=${M} シェアでの復元成功確認"
  # M シェアを結合して復元を試みる
  local combined_shares=""
  # M 個のシェアを結合する
  for i in $(seq 1 "${M}"); do
    # シェアファイルの内容を取得する
    combined_shares="${combined_shares}$(cat "${TMP_DIR}/share_${i}.txt")"$'\n'
  done
  # ssss-combine で M シェアを使って復元を試みる (成功が期待される)
  local recovered
  # 復元コマンドを実行して結果を取得する
  recovered=$(echo "${combined_shares}" | ssss-combine -t "${M}" -q 2>/dev/null || echo "FAILED")
  # 復元されたシークレットを元のシークレットと比較する
  if [[ "${recovered}" == "${TEST_SECRET}" ]]; then
    # 復元成功かつシークレットが一致した場合は成功とする
    pass "M シェアでの復元が成功し、シークレットが一致した"
  elif [[ "${recovered}" == "FAILED" ]]; then
    # 復元が失敗した場合は失敗とする
    fail "M シェアでの復元が失敗した"
  else
    # シークレットが一致しない場合は失敗とする
    fail "M シェアでの復元は成功したがシークレットが不一致 (復元値: ${recovered:0:8}...)"
  fi
}

# 異なるシェアの組み合わせテスト: 任意の M シェアで復元できることを確認する
test_any_m_combination_succeeds() {
  # テスト名称を出力する
  log "テスト開始: 任意の M シェア組み合わせでの復元確認 (シェア 2,3,5)"
  # シェア 2, 3, 5 の組み合わせで復元を試みる
  local combined_shares=""
  # 選択したシェアを結合する
  for i in 2 3 5; do
    # シェアファイルの内容を取得する
    combined_shares="${combined_shares}$(cat "${TMP_DIR}/share_${i}.txt")"$'\n'
  done
  # ssss-combine で選択したシェアを使って復元を試みる (成功が期待される)
  local recovered
  # 復元コマンドを実行して結果を取得する
  recovered=$(echo "${combined_shares}" | ssss-combine -t "${M}" -q 2>/dev/null || echo "FAILED")
  # 復元されたシークレットを元のシークレットと比較する
  if [[ "${recovered}" == "${TEST_SECRET}" ]]; then
    # 任意の M シェア組み合わせでも復元できることを確認する
    pass "任意の M シェア組み合わせでの復元が成功した"
  else
    # 復元が失敗した場合は失敗とする
    fail "任意の M シェア組み合わせでの復元が失敗した (recovered=${recovered:0:8}...)"
  fi
}

# メイン処理: 全テストを順番に実行する
main() {
  # テスト開始ログを出力する
  log "Shamir 閾値テスト開始: M=${M}, N=${N}"

  # ssss ツールの存在を確認する
  check_ssss

  # テスト用シークレットを生成する
  TEST_SECRET=$(generate_test_secret)
  # テスト用シークレットをエクスポートして各テスト関数でアクセス可能にする
  export TEST_SECRET
  # テスト用シークレット生成完了ログを出力する
  log "テスト用シークレット生成完了 (先頭 8 文字: ${TEST_SECRET:0:8}...)"

  # シークレットを Shamir 分割する
  split_secret "${TEST_SECRET}"

  # M-1 シェアでの復元失敗テストを実行する
  test_m_minus_1_shares_fails

  # M シェアでの復元成功テストを実行する
  test_m_shares_succeeds

  # 任意の M シェア組み合わせでの復元テストを実行する
  test_any_m_combination_succeeds

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
  log "全 Shamir 閾値テスト成功"
}

# スクリプトのエントリポイント: main 関数を呼び出す
main "$@"
