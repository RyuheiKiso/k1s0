#!/usr/bin/env bash
# エラー発生時に即座に終了し、未定義変数をエラーとして扱い、パイプラインの途中エラーも検知する（M-26対応）
set -euo pipefail
# events.yaml から各言語（Rust/Go）のイベントコードを生成するスクリプト。
# Rust codegen ライブラリを使用して proto ファイル、型定義、Producer/Consumer を生成する。
#
# 使用方法:
#   ./scripts/generate-event-codegen.sh [events.yaml のパス] [出力ディレクトリ]
#
# 引数:
#   $1 - events.yaml のパス（デフォルト: ./events.yaml）
#   $2 - 出力ディレクトリ（デフォルト: カレントディレクトリ）
#
# 環境変数:
#   CODEGEN_BIN - codegen バイナリのパス（デフォルト: cargo run -p codegen）

set -euo pipefail

# スクリプト自身のディレクトリを取得
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

# プロジェクトルートディレクトリを取得
ROOT_DIR=$(cd "$SCRIPT_DIR/.." && pwd)

# 引数またはデフォルト値で events.yaml のパスを設定
EVENTS_YAML="${1:-./events.yaml}"

# 引数またはデフォルト値で出力ディレクトリを設定
OUTPUT_DIR="${2:-.}"

# カラー出力用の ANSI エスケープシーケンス
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 情報ログを出力する関数
log_info() {
  echo -e "${GREEN}[INFO]${NC} $1"
}

# 成功ログを出力する関数
log_success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# エラーログを出力して終了する関数
log_error() {
  echo -e "${RED}[ERROR]${NC} $1" >&2
}

# 警告ログを出力する関数
log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

# events.yaml の存在を確認
if [[ ! -f "$EVENTS_YAML" ]]; then
  log_error "events.yaml が見つかりません: $EVENTS_YAML"
  echo "使用方法: $0 [events.yaml のパス] [出力ディレクトリ]"
  exit 1
fi

# 出力ディレクトリが存在しない場合は作成
if [[ ! -d "$OUTPUT_DIR" ]]; then
  log_info "出力ディレクトリを作成します: $OUTPUT_DIR"
  mkdir -p "$OUTPUT_DIR"
fi

log_info "=== イベントコード生成を開始 ==="
log_info "  events.yaml: $EVENTS_YAML"
log_info "  出力先: $OUTPUT_DIR"

# codegen バイナリのパスを決定（環境変数または cargo run を使用）
if [[ -n "${CODEGEN_BIN:-}" ]]; then
  log_info "CODEGEN_BIN を使用: $CODEGEN_BIN"
  CODEGEN_CMD="$CODEGEN_BIN"
else
  log_info "cargo run -p codegen を使用してコード生成を実行"
  CODEGEN_CMD="cargo run -p codegen --features event-codegen --"
fi

# Rust codegen バイナリを実行してイベントコードを生成
log_info "コード生成を実行中..."
if ! $CODEGEN_CMD event-gen --input "$EVENTS_YAML" --output "$OUTPUT_DIR"; then
  log_error "コード生成に失敗しました"
  exit 1
fi

# 生成されたファイルの一覧を表示
log_info "生成されたファイル一覧:"
if command -v find &>/dev/null; then
  find "$OUTPUT_DIR" -type f -newer "$EVENTS_YAML" -o -type f -name "*.rs" -o -type f -name "*.go" -o -type f -name "*.proto" 2>/dev/null | sort | while read -r file; do
    echo "  $file"
  done
fi

log_success "=== イベントコード生成が完了しました ==="
