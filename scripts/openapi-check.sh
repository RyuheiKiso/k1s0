#!/bin/bash
# OpenAPI 差分検知スクリプト
# 使用方法:
#   ./scripts/openapi-check.sh lint           - すべてのサービスでlintを実行
#   ./scripts/openapi-check.sh breaking       - mainブランチとの破壊的変更を検出
#   ./scripts/openapi-check.sh breaking HEAD~1 - 前コミットとの破壊的変更を検出
#   ./scripts/openapi-check.sh generate       - クライアントコード生成

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# サービスディレクトリ（feature サービスは動的に検出）
FRAMEWORK_SERVICES=(
    "framework/backend/rust/services/auth-service"
    "framework/backend/rust/services/config-service"
    "framework/backend/rust/services/endpoint-service"
)

# カラー出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

# 必要なツールの確認
check_tools() {
    local missing=()

    # oasdiff (OpenAPI diff tool)
    if ! command -v oasdiff &> /dev/null; then
        missing+=("oasdiff")
    fi

    # spectral (OpenAPI linter)
    if ! command -v spectral &> /dev/null; then
        missing+=("spectral")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Required tools not found: ${missing[*]}"
        log_info "Installation instructions:"
        log_info "  oasdiff: go install github.com/tufin/oasdiff@latest"
        log_info "  spectral: npm install -g @stoplight/spectral-cli"
        return 1
    fi

    return 0
}

# OpenAPI ファイルを持つサービスを検出
find_openapi_services() {
    local services=()

    # Framework サービス
    for service in "${FRAMEWORK_SERVICES[@]}"; do
        local openapi_file="$ROOT_DIR/$service/openapi/openapi.yaml"
        if [ -f "$openapi_file" ]; then
            services+=("$service")
        fi
    done

    # Feature サービス
    if [ -d "$ROOT_DIR/feature/backend/rust" ]; then
        for feature_dir in "$ROOT_DIR/feature/backend/rust"/*/; do
            if [ -d "$feature_dir" ]; then
                local openapi_file="${feature_dir}openapi/openapi.yaml"
                if [ -f "$openapi_file" ]; then
                    local relative_path="${feature_dir#$ROOT_DIR/}"
                    services+=("${relative_path%/}")
                fi
            fi
        done
    fi

    echo "${services[@]}"
}

# Spectral 設定ファイルを作成（存在しない場合）
ensure_spectral_config() {
    local config_file="$ROOT_DIR/.spectral.yaml"
    if [ ! -f "$config_file" ]; then
        log_info "Creating default Spectral configuration..."
        cat > "$config_file" << 'EOF'
extends:
  - spectral:oas

rules:
  # 必須ルール
  oas3-api-servers: error
  operation-operationId: error
  operation-tags: error
  info-contact: warn
  info-description: warn
  operation-description: warn

  # 命名規則
  operation-operationId-valid-in-url: error

  # セキュリティ
  oas3-operation-security-defined: error

  # 無効化するルール（必要に応じてコメント解除）
  # oas3-unused-component: off
EOF
    fi
}

# lint実行
run_lint() {
    local errors=0

    ensure_spectral_config

    local services=($(find_openapi_services))

    if [ ${#services[@]} -eq 0 ]; then
        log_warn "No OpenAPI files found"
        return 0
    fi

    for service in "${services[@]}"; do
        local openapi_file="$ROOT_DIR/$service/openapi/openapi.yaml"

        if [ ! -f "$openapi_file" ]; then
            continue
        fi

        log_info "Linting $service"

        if ! spectral lint "$openapi_file" --ruleset "$ROOT_DIR/.spectral.yaml"; then
            log_error "Lint failed for $service"
            ((errors++))
        else
            log_info "Lint passed for $service"
        fi
    done

    if [ $errors -gt 0 ]; then
        log_error "Lint failed for $errors service(s)"
        return 1
    fi

    log_info "All OpenAPI lint checks passed!"
}

# breaking変更検出
run_breaking() {
    local against="${1:-origin/main}"
    local errors=0
    local warnings=0

    log_info "Checking breaking changes against: $against"

    local services=($(find_openapi_services))

    if [ ${#services[@]} -eq 0 ]; then
        log_warn "No OpenAPI files found"
        return 0
    fi

    for service in "${services[@]}"; do
        local openapi_file="$ROOT_DIR/$service/openapi/openapi.yaml"

        if [ ! -f "$openapi_file" ]; then
            continue
        fi

        log_info "Checking breaking changes for $service"

        # 比較対象のファイルを取得
        local base_content
        if ! base_content=$(git show "$against:$service/openapi/openapi.yaml" 2>/dev/null); then
            log_warn "No OpenAPI file in $against for $service, skipping breaking check"
            continue
        fi

        # 一時ファイルに保存
        local temp_base=$(mktemp)
        echo "$base_content" > "$temp_base"

        # oasdiff で破壊的変更をチェック
        local diff_output
        if diff_output=$(oasdiff breaking "$temp_base" "$openapi_file" 2>&1); then
            log_info "No breaking changes in $service"
        else
            if echo "$diff_output" | grep -q "error"; then
                log_error "Breaking changes detected in $service:"
                echo "$diff_output"
                ((errors++))
            else
                log_warn "Potential issues in $service:"
                echo "$diff_output"
                ((warnings++))
            fi
        fi

        rm -f "$temp_base"
    done

    if [ $errors -gt 0 ]; then
        log_error "Breaking changes detected in $errors service(s)"
        return 1
    fi

    if [ $warnings -gt 0 ]; then
        log_warn "Warnings in $warnings service(s)"
    fi

    log_info "No breaking changes detected!"
}

# 差分表示
run_diff() {
    local against="${1:-origin/main}"

    log_info "Showing OpenAPI diff against: $against"

    local services=($(find_openapi_services))

    for service in "${services[@]}"; do
        local openapi_file="$ROOT_DIR/$service/openapi/openapi.yaml"

        if [ ! -f "$openapi_file" ]; then
            continue
        fi

        local base_content
        if ! base_content=$(git show "$against:$service/openapi/openapi.yaml" 2>/dev/null); then
            log_warn "No OpenAPI file in $against for $service"
            continue
        fi

        local temp_base=$(mktemp)
        echo "$base_content" > "$temp_base"

        log_info "Diff for $service:"
        oasdiff diff "$temp_base" "$openapi_file" --format text || true

        rm -f "$temp_base"
    done
}

# Fingerprint 生成
generate_fingerprint() {
    local services=($(find_openapi_services))

    for service in "${services[@]}"; do
        local openapi_file="$ROOT_DIR/$service/openapi/openapi.yaml"
        local fingerprint_file="$ROOT_DIR/$service/openapi/openapi.fingerprint"

        if [ ! -f "$openapi_file" ]; then
            continue
        fi

        log_info "Generating fingerprint for $service"

        # SHA256 ハッシュを生成
        local hash
        if command -v sha256sum &> /dev/null; then
            hash=$(sha256sum "$openapi_file" | cut -d' ' -f1)
        elif command -v shasum &> /dev/null; then
            hash=$(shasum -a 256 "$openapi_file" | cut -d' ' -f1)
        else
            log_error "No SHA256 tool found"
            return 1
        fi

        echo "$hash" > "$fingerprint_file"
        log_info "Fingerprint: $hash"
    done

    log_info "Fingerprints generated!"
}

# Fingerprint 検証
verify_fingerprint() {
    local errors=0
    local services=($(find_openapi_services))

    for service in "${services[@]}"; do
        local openapi_file="$ROOT_DIR/$service/openapi/openapi.yaml"
        local fingerprint_file="$ROOT_DIR/$service/openapi/openapi.fingerprint"

        if [ ! -f "$openapi_file" ]; then
            continue
        fi

        if [ ! -f "$fingerprint_file" ]; then
            log_warn "No fingerprint file for $service"
            continue
        fi

        log_info "Verifying fingerprint for $service"

        local expected_hash=$(cat "$fingerprint_file")
        local actual_hash
        if command -v sha256sum &> /dev/null; then
            actual_hash=$(sha256sum "$openapi_file" | cut -d' ' -f1)
        elif command -v shasum &> /dev/null; then
            actual_hash=$(shasum -a 256 "$openapi_file" | cut -d' ' -f1)
        fi

        if [ "$expected_hash" = "$actual_hash" ]; then
            log_info "Fingerprint verified for $service"
        else
            log_error "Fingerprint mismatch for $service"
            log_error "  Expected: $expected_hash"
            log_error "  Actual:   $actual_hash"
            ((errors++))
        fi
    done

    if [ $errors -gt 0 ]; then
        log_error "Fingerprint verification failed for $errors service(s)"
        return 1
    fi

    log_info "All fingerprints verified!"
}

# メイン
main() {
    case "${1:-help}" in
        lint)
            check_tools || exit 1
            run_lint
            ;;
        breaking)
            check_tools || exit 1
            run_breaking "${2:-origin/main}"
            ;;
        diff)
            check_tools || exit 1
            run_diff "${2:-origin/main}"
            ;;
        fingerprint)
            generate_fingerprint
            ;;
        verify)
            verify_fingerprint
            ;;
        all)
            check_tools || exit 1
            run_lint
            run_breaking "${2:-origin/main}"
            ;;
        *)
            echo "Usage: $0 {lint|breaking|diff|fingerprint|verify|all} [against]"
            echo ""
            echo "Commands:"
            echo "  lint              Run OpenAPI lint on all services"
            echo "  breaking [ref]    Check for breaking changes (default: origin/main)"
            echo "  diff [ref]        Show diff with base (default: origin/main)"
            echo "  fingerprint       Generate fingerprint files"
            echo "  verify            Verify fingerprint files"
            echo "  all [ref]         Run all checks"
            echo ""
            echo "Required tools:"
            echo "  oasdiff   - OpenAPI diff tool (go install github.com/tufin/oasdiff@latest)"
            echo "  spectral  - OpenAPI linter (npm install -g @stoplight/spectral-cli)"
            exit 1
            ;;
    esac
}

main "$@"
