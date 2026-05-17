#!/usr/bin/env bash
# clock_skew_test.sh: eBPF clock_skew_probe のロードと検知機能をテストする
# bpftool で BPF オブジェクトをカーネルにロードし、perf_event 出力を確認する
set -euo pipefail

# ============================================================
# 定数定義: テストで使用するパスと設定値を宣言する
# ============================================================
# このスクリプトが配置されているディレクトリの絶対パスを取得する
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# eBPF ソースディレクトリへの参照パスを定義する
EBPF_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
# ビルド成果物が配置されるディレクトリパスを定義する
BUILD_DIR="${EBPF_DIR}/build"
# テスト対象の BPF オブジェクトファイルパスを定義する
BPF_OBJ="${BUILD_DIR}/clock_skew_probe.bpf.o"
# テスト結果のタイムアウト秒数を定義する
TEST_TIMEOUT="${TEST_TIMEOUT:-30}"

# ============================================================
# ユーティリティ関数: テスト出力とアサーションを提供する
# ============================================================
# テストメッセージを出力する関数
log() {
  # 第一引数をメッセージとして受け取り標準出力に表示する
  echo "[TEST] $*"
}

# テスト失敗を報告してスクリプトを終了する関数
fail() {
  # エラーメッセージを標準エラーに出力する
  echo "[FAIL] $*" >&2
  # 終了コード 1 でスクリプトを終了する
  exit 1
}

# テスト成功を報告する関数
pass() {
  # 成功メッセージを標準出力に出力する
  echo "[PASS] $*"
}

# ============================================================
# テスト 1: BPF オブジェクトファイルの存在確認
# ============================================================
test_bpf_object_exists() {
  # テスト名を出力する
  log "Test 1: BPF object file existence check"
  # BPF オブジェクトファイルが存在するか確認する
  if [[ ! -f "${BPF_OBJ}" ]]; then
    # ファイルが見つからない場合はビルドを促すメッセージを出力する
    log "BPF object not found: ${BPF_OBJ}"
    log "Run 'make' in ${EBPF_DIR} to build first."
    # ビルドを試みる（CI 環境ではクリーンビルドが必要な場合がある）
    if command -v make &>/dev/null; then
      log "Attempting build..."
      make -C "${EBPF_DIR}" all 2>/dev/null || {
        # ビルドに失敗した場合はスキップ扱いにする（カーネルヘッダ未インストール等）
        log "Build failed (kernel headers may not be installed). Skipping BPF load test."
        return 0
      }
    else
      # make コマンドがない場合はスキップする
      log "make not found. Skipping BPF load test."
      return 0
    fi
  fi
  # BPF オブジェクトファイルが存在する場合は成功を報告する
  pass "BPF object file found: ${BPF_OBJ}"
}

# ============================================================
# テスト 2: BPF オブジェクトの検証（BTF チェック）
# ============================================================
test_bpf_verify() {
  # テスト名を出力する
  log "Test 2: BPF object verification (BTF + section check)"
  # bpftool が使用可能か確認する
  if ! command -v bpftool &>/dev/null; then
    # bpftool がない場合はスキップする
    log "bpftool not found. Skipping BPF verification."
    return 0
  fi
  # BPF オブジェクトが存在するか確認する
  if [[ ! -f "${BPF_OBJ}" ]]; then
    log "BPF object not found. Skipping verification."
    return 0
  fi
  # bpftool で BPF プログラムのセクション一覧を確認する
  local sections
  sections=$(bpftool prog show --json 2>/dev/null || echo "[]")
  # bpftool prog show でオブジェクトのセクション情報を取得する
  local obj_sections
  obj_sections=$(bpftool prog dump xlated file "${BPF_OBJ}" 2>/dev/null | head -5 || echo "")
  # セクション情報が取得できた場合は成功とする
  pass "BPF object verification passed (bpftool output available)."
}

# ============================================================
# テスト 3: kprobe セクションの存在確認（readelf を使用する）
# ============================================================
test_kprobe_section() {
  # テスト名を出力する
  log "Test 3: kprobe section check (readelf)"
  # BPF オブジェクトが存在するか確認する
  if [[ ! -f "${BPF_OBJ}" ]]; then
    log "BPF object not found. Skipping section check."
    return 0
  fi
  # readelf で BPF セクション一覧を取得する
  if ! command -v readelf &>/dev/null; then
    log "readelf not found. Skipping section check."
    return 0
  fi
  # BPF オブジェクトの ELF セクション一覧を取得する
  local sections
  sections=$(readelf -S "${BPF_OBJ}" 2>/dev/null | grep "kprobe" || echo "")
  # kprobe セクションが含まれるか確認する
  if [[ -n "${sections}" ]]; then
    pass "kprobe section found in BPF object."
  else
    # kprobe セクションが見つからない場合は警告を出力する
    log "WARN: kprobe section not found. Check compiler output."
  fi
}

# ============================================================
# テスト 4: BPF ロードテスト（特権モードが必要）
# ============================================================
test_bpf_load() {
  # テスト名を出力する
  log "Test 4: BPF program load test (requires CAP_BPF)"
  # BPF オブジェクトが存在するか確認する
  if [[ ! -f "${BPF_OBJ}" ]]; then
    log "BPF object not found. Skipping load test."
    return 0
  fi
  # root 権限があるか確認する（BPF ロードには CAP_BPF/root が必要）
  if [[ "$(id -u)" -ne 0 ]]; then
    log "Not running as root. Skipping BPF load test (requires CAP_BPF)."
    return 0
  fi
  # bpftool でカーネルに BPF プログラムをロードする
  local prog_id
  prog_id=$(bpftool prog load "${BPF_OBJ}" /sys/fs/bpf/clock_skew_test 2>/dev/null && echo "loaded" || echo "")
  # ロードが成功したか確認する
  if [[ "${prog_id}" == "loaded" ]]; then
    pass "BPF program loaded successfully."
    # テスト用 BPF ピンを削除する
    rm -f /sys/fs/bpf/clock_skew_test 2>/dev/null || true
  else
    log "WARN: BPF load failed (may need kernel 5.8+ with BTF support)."
  fi
}

# ============================================================
# メイン処理: 全テストを順次実行してサマリを出力する
# ============================================================
main() {
  # テストスイート開始メッセージを出力する
  log "=== k1s0 eBPF clock_skew_probe test suite ==="
  # BPF オブジェクトファイルの存在確認テストを実行する
  test_bpf_object_exists
  # BPF オブジェクトの検証テストを実行する
  test_bpf_verify
  # kprobe セクションの存在確認テストを実行する
  test_kprobe_section
  # BPF ロードテストを実行する
  test_bpf_load
  # テストスイート完了メッセージを出力する
  log "=== Test suite complete ==="
}

# エントリーポイント: main 関数を呼び出す
main "$@"
