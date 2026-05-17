#!/usr/bin/env bash
# Shamir Secret Sharing M=3 of N=5 鍵分割セレモニースクリプト
# KEK (Key Encryption Key) を M-of-N Shamir Secret Sharing で分割して管理する
# PKCS#11 HSM 対応コメント付き: 本番環境では SoftHSM2 から実 HSM へ切り替える
set -euo pipefail

# シェアの生成数 (N=5): 合計 5 人のキーホルダーに分割する
N=5
# 復元閾値 (M=3): 3 人以上のシェアが揃わないと KEK を復元できない
M=3

# 出力ディレクトリ: シェアファイルを格納するディレクトリを設定する
OUTPUT_DIR="${OUTPUT_DIR:-./kek_shares}"

# SoftHSM2 のライブラリパスを設定する (kind cluster 環境で使用する)
SOFTHSM2_LIB="${SOFTHSM2_LIB:-/usr/lib/softhsm/libsofthsm2.so}"

# PKCS#11 スロット番号を設定する (SoftHSM2 初期化済みスロット)
PKCS11_SLOT="${PKCS11_SLOT:-0}"

# セレモニー用の一時ディレクトリを作成する
TMP_DIR=$(mktemp -d)

# スクリプト終了時に一時ディレクトリを削除するトラップを設定する
trap 'rm -rf "${TMP_DIR}"' EXIT

# ログ出力関数: タイムスタンプ付きでセレモニーログを出力する
log() {
  # タイムスタンプ付きでメッセージを標準出力に出力する
  echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] CEREMONY: $*"
}

# エラー出力関数: エラーメッセージを標準エラー出力に出力する
err() {
  # エラーメッセージを標準エラー出力に出力する
  echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] ERROR: $*" >&2
}

# SoftHSM2 のセットアップ確認関数: SoftHSM2 が利用可能かを確認する
check_softhsm2() {
  # SoftHSM2 ライブラリの存在を確認する
  if [[ ! -f "${SOFTHSM2_LIB}" ]]; then
    # SoftHSM2 が見つからない場合はエラーを出力する
    err "SoftHSM2 ライブラリが見つからない: ${SOFTHSM2_LIB}"
    # 本番 HSM への切り替え手順を出力する
    err "本番環境では SOFTHSM2_LIB を実 HSM の PKCS#11 ライブラリパスに変更すること"
    # エラーコードで終了する
    exit 1
  fi
  # SoftHSM2 が利用可能であることをログに記録する
  log "SoftHSM2 確認完了: ${SOFTHSM2_LIB}"
}

# ssss-split コマンドの存在確認関数: Shamir ツールが利用可能かを確認する
check_ssss() {
  # ssss-split コマンドの存在を確認する
  if ! command -v ssss-split &>/dev/null; then
    # ssss-split が見つからない場合はインストール手順を出力する
    err "ssss-split コマンドが見つからない: apt-get install ssss でインストールすること"
    # エラーコードで終了する
    exit 1
  fi
  # ssss-combine コマンドの存在を確認する
  if ! command -v ssss-combine &>/dev/null; then
    # ssss-combine が見つからない場合はエラーを出力する
    err "ssss-combine コマンドが見つからない: apt-get install ssss でインストールすること"
    # エラーコードで終了する
    exit 1
  fi
  # ssss ツールが利用可能であることをログに記録する
  log "ssss ツール確認完了"
}

# 出力ディレクトリの初期化関数: シェアファイル格納ディレクトリを準備する
init_output_dir() {
  # 出力ディレクトリが存在しない場合は作成する
  mkdir -p "${OUTPUT_DIR}"
  # 出力ディレクトリのパーミッションを制限する (所有者のみ読み書き可能)
  chmod 700 "${OUTPUT_DIR}"
  # 出力ディレクトリの初期化完了をログに記録する
  log "出力ディレクトリ初期化完了: ${OUTPUT_DIR}"
}

# KEK 生成関数: 256 ビットのランダムな KEK を生成する
generate_kek() {
  # /dev/urandom から 256 ビット (32 バイト) のランダムデータを生成する
  local kek
  # OpenSSL で 256 ビットのランダムな KEK を 16 進数形式で生成する
  kek=$(openssl rand -hex 32)
  # 生成した KEK を返す
  echo "${kek}"
}

# Shamir 分割関数: KEK を M-of-N Shamir Secret Sharing で分割する
split_kek_shamir() {
  # 引数: 分割する KEK 文字列
  local kek="$1"
  # ssss-split コマンドで KEK を N=5 シェアに分割し閾値 M=3 を設定する
  log "Shamir 分割開始: M=${M}, N=${N}"
  # ssss-split に KEK を入力してシェアを生成する
  echo "${kek}" | ssss-split -t "${M}" -n "${N}" -q 2>/dev/null
}

# シェア保存関数: 分割されたシェアをファイルに保存する
save_shares() {
  # 引数: シェア配列
  local -a shares=("$@")
  # シェアのインデックスを初期化する
  local idx=1
  # 各シェアをファイルに保存する
  for share in "${shares[@]}"; do
    # シェアファイルのパスを組み立てる
    local share_file="${OUTPUT_DIR}/kek_share_${idx}.txt"
    # シェアをファイルに書き込む
    echo "${share}" > "${share_file}"
    # シェアファイルのパーミッションを制限する (所有者のみ読み取り可能)
    chmod 600 "${share_file}"
    # シェアファイルの保存完了をログに記録する
    log "シェア ${idx}/${N} 保存完了: ${share_file}"
    # インデックスを増加させる
    idx=$((idx + 1))
  done
}

# PKCS#11 への格納関数: 生成された KEK を SoftHSM2 (または実 HSM) に格納する
store_in_hsm() {
  # 引数: 格納する KEK 文字列
  local kek="$1"
  # PKCS#11 ライブラリを使用して KEK を HSM に格納する (p11tool を使用)
  log "HSM への KEK 格納を開始する (PKCS#11 スロット: ${PKCS11_SLOT})"
  # 本番環境では以下のコマンドで実際に HSM に格納する
  # p11tool --provider="${SOFTHSM2_LIB}" --login --set-id=01 --label=k1s0-kek --write-secret="${kek}"
  # kind cluster 環境ではシミュレーションログを出力する
  log "HSM 格納シミュレーション完了 (本番環境では実 HSM に格納すること)"
}

# メイン処理: セレモニーの全ステップを順番に実行する
main() {
  # セレモニー開始ログを出力する
  log "KEK Shamir セレモニー開始: M=${M}, N=${N}"

  # SoftHSM2 の利用可能性を確認する
  check_softhsm2

  # ssss ツールの利用可能性を確認する
  check_ssss

  # 出力ディレクトリを初期化する
  init_output_dir

  # KEK を生成する
  log "KEK 生成中..."
  # 256 ビットのランダム KEK を生成する
  KEK=$(generate_kek)
  # KEK 生成完了ログを出力する
  log "KEK 生成完了 (先頭 8 文字: ${KEK:0:8}...)"

  # KEK を SoftHSM2 に格納する
  store_in_hsm "${KEK}"

  # KEK を Shamir Secret Sharing で分割する
  log "Shamir 分割処理中..."
  # シェアを配列に格納する
  mapfile -t SHARES < <(split_kek_shamir "${KEK}")

  # 分割されたシェアの数を確認する
  if [[ "${#SHARES[@]}" -ne "${N}" ]]; then
    # シェア数が期待値と異なる場合はエラーを出力する
    err "シェア数が期待値と異なる: 期待=${N}, 実際=${#SHARES[@]}"
    # エラーコードで終了する
    exit 1
  fi

  # シェアをファイルに保存する
  save_shares "${SHARES[@]}"

  # KEK をメモリから消去する (セキュリティのため変数をリセットする)
  KEK=""

  # セレモニー完了ログを出力する
  log "KEK Shamir セレモニー完了: シェアは ${OUTPUT_DIR} に保存された"
  # セキュリティ注意事項を出力する
  log "重要: 各シェアを別々の安全な場所に保管し、単一箇所に集めないこと"
}

# スクリプトのエントリポイント: main 関数を呼び出す
main "$@"
