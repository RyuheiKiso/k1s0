#!/bin/bash
# buf lint/breaking チェックスクリプト
# 使用方法:
#   ./scripts/buf-check.sh lint           - すべてのサービスでlintを実行
#   ./scripts/buf-check.sh breaking       - mainブランチとの破壊的変更を検出
#   ./scripts/buf-check.sh breaking HEAD~1 - 前コミットとの破壊的変更を検出

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# サービスディレクトリの一覧
SERVICES=(
    "framework/backend/rust/services/auth-service"
    "framework/backend/rust/services/config-service"
    "framework/backend/rust/services/endpoint-service"
)

# カラー出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# bufがインストールされているか確認
check_buf() {
    if ! command -v buf &> /dev/null; then
        log_error "buf CLI is not installed. Please install it first."
        log_info "Installation: https://docs.buf.build/installation"
        exit 1
    fi
}

# lint実行
run_lint() {
    local errors=0

    for service in "${SERVICES[@]}"; do
        local service_path="$ROOT_DIR/$service"

        if [ ! -f "$service_path/buf.yaml" ]; then
            log_warn "Skipping $service (no buf.yaml found)"
            continue
        fi

        log_info "Running buf lint for $service"

        if ! (cd "$service_path" && buf lint); then
            log_error "Lint failed for $service"
            ((errors++))
        else
            log_info "Lint passed for $service"
        fi
    done

    if [ $errors -gt 0 ]; then
        log_error "Lint failed for $errors service(s)"
        exit 1
    fi

    log_info "All lint checks passed!"
}

# breaking変更検出
run_breaking() {
    local against="${1:-origin/main}"
    local errors=0

    log_info "Checking breaking changes against: $against"

    for service in "${SERVICES[@]}"; do
        local service_path="$ROOT_DIR/$service"

        if [ ! -f "$service_path/buf.yaml" ]; then
            log_warn "Skipping $service (no buf.yaml found)"
            continue
        fi

        log_info "Checking breaking changes for $service"

        # 比較対象のprotoファイルが存在するか確認
        if ! git show "$against:$service/proto" &> /dev/null 2>&1; then
            log_warn "No proto files in $against for $service, skipping breaking check"
            continue
        fi

        # 一時ディレクトリに比較対象を展開
        local temp_dir=$(mktemp -d)
        trap "rm -rf $temp_dir" EXIT

        git show "$against:$service/buf.yaml" > "$temp_dir/buf.yaml" 2>/dev/null || true
        mkdir -p "$temp_dir/proto"
        git archive "$against" -- "$service/proto" | tar -x -C "$temp_dir" --strip-components=$(echo "$service/proto" | tr '/' '\n' | wc -l) 2>/dev/null || true

        if [ ! -d "$temp_dir/proto" ] || [ -z "$(ls -A $temp_dir/proto 2>/dev/null)" ]; then
            log_warn "No proto files to compare for $service"
            continue
        fi

        if ! (cd "$service_path" && buf breaking --against "$temp_dir"); then
            log_error "Breaking changes detected in $service"
            ((errors++))
        else
            log_info "No breaking changes in $service"
        fi
    done

    if [ $errors -gt 0 ]; then
        log_error "Breaking changes detected in $errors service(s)"
        exit 1
    fi

    log_info "No breaking changes detected!"
}

# フォーマットチェック
run_format() {
    local errors=0

    for service in "${SERVICES[@]}"; do
        local service_path="$ROOT_DIR/$service"

        if [ ! -f "$service_path/buf.yaml" ]; then
            continue
        fi

        log_info "Checking format for $service"

        if ! (cd "$service_path" && buf format --diff --exit-code); then
            log_error "Format check failed for $service"
            ((errors++))
        fi
    done

    if [ $errors -gt 0 ]; then
        log_error "Format check failed for $errors service(s)"
        log_info "Run 'buf format -w' to fix formatting"
        exit 1
    fi

    log_info "All format checks passed!"
}

# メイン
main() {
    check_buf

    case "${1:-lint}" in
        lint)
            run_lint
            ;;
        breaking)
            run_breaking "${2:-origin/main}"
            ;;
        format)
            run_format
            ;;
        all)
            run_lint
            run_format
            run_breaking "${2:-origin/main}"
            ;;
        *)
            echo "Usage: $0 {lint|breaking|format|all} [against]"
            echo ""
            echo "Commands:"
            echo "  lint              Run buf lint on all services"
            echo "  breaking [ref]    Check for breaking changes (default: origin/main)"
            echo "  format            Check proto file formatting"
            echo "  all [ref]         Run all checks"
            exit 1
            ;;
    esac
}

main "$@"
