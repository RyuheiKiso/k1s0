#!/bin/bash
# 生成一致チェックスクリプト
# 契約（proto/OpenAPI）から生成されるコードの一致性を検証
#
# 使用方法:
#   ./scripts/gen-check.sh fingerprint  - fingerprintファイルを生成
#   ./scripts/gen-check.sh verify       - fingerprintを検証
#   ./scripts/gen-check.sh generate     - コード生成を実行
#   ./scripts/gen-check.sh check        - 生成して検証

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

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

# SHA256 ハッシュを計算
compute_hash() {
    local file="$1"
    if command -v sha256sum &> /dev/null; then
        sha256sum "$file" | cut -d' ' -f1
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "$file" | cut -d' ' -f1
    else
        log_error "No SHA256 tool found"
        return 1
    fi
}

# ディレクトリのハッシュを計算（ファイル内容の連結ハッシュ）
compute_dir_hash() {
    local dir="$1"
    local hash_file=$(mktemp)

    # ファイルをソートして連結し、ハッシュを計算
    find "$dir" -type f -name "*.proto" -o -name "*.yaml" -o -name "*.yml" 2>/dev/null | sort | while read -r file; do
        compute_hash "$file" >> "$hash_file"
    done

    if [ -s "$hash_file" ]; then
        compute_hash "$hash_file"
    else
        echo "empty"
    fi

    rm -f "$hash_file"
}

# Proto fingerprint を生成
generate_proto_fingerprint() {
    local service_dir="$1"
    local proto_dir="$service_dir/proto"
    local fingerprint_file="$service_dir/gen/.k1s0-gen.sha256"

    if [ ! -d "$proto_dir" ]; then
        return 0
    fi

    log_info "Generating proto fingerprint for $service_dir"

    # gen ディレクトリを作成
    mkdir -p "$service_dir/gen"

    # proto ディレクトリのハッシュを計算
    local proto_hash=$(compute_dir_hash "$proto_dir")

    # buf.yaml のハッシュを追加
    local buf_hash="none"
    if [ -f "$service_dir/buf.yaml" ]; then
        buf_hash=$(compute_hash "$service_dir/buf.yaml")
    fi

    # fingerprint を書き込み
    cat > "$fingerprint_file" << EOF
# k1s0 Generation Fingerprint
# Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
#
# This file tracks the hash of source files (proto, buf.yaml)
# to detect when regeneration is needed.
#
proto_hash=$proto_hash
buf_hash=$buf_hash
combined_hash=$(echo "$proto_hash$buf_hash" | sha256sum | cut -d' ' -f1 2>/dev/null || echo "$proto_hash$buf_hash" | shasum -a 256 | cut -d' ' -f1)
EOF

    log_info "Proto fingerprint generated: $fingerprint_file"
}

# OpenAPI fingerprint を生成
generate_openapi_fingerprint() {
    local service_dir="$1"
    local openapi_file="$service_dir/openapi/openapi.yaml"
    local fingerprint_file="$service_dir/openapi/openapi.fingerprint"

    if [ ! -f "$openapi_file" ]; then
        return 0
    fi

    log_info "Generating OpenAPI fingerprint for $service_dir"

    local hash=$(compute_hash "$openapi_file")
    echo "$hash" > "$fingerprint_file"

    log_info "OpenAPI fingerprint generated: $fingerprint_file"
}

# Proto fingerprint を検証
verify_proto_fingerprint() {
    local service_dir="$1"
    local proto_dir="$service_dir/proto"
    local fingerprint_file="$service_dir/gen/.k1s0-gen.sha256"

    if [ ! -d "$proto_dir" ]; then
        return 0
    fi

    if [ ! -f "$fingerprint_file" ]; then
        log_warn "No fingerprint file for $service_dir"
        return 0
    fi

    log_info "Verifying proto fingerprint for $service_dir"

    # 現在のハッシュを計算
    local current_proto_hash=$(compute_dir_hash "$proto_dir")
    local current_buf_hash="none"
    if [ -f "$service_dir/buf.yaml" ]; then
        current_buf_hash=$(compute_hash "$service_dir/buf.yaml")
    fi

    # fingerprint ファイルからハッシュを読み取り
    local expected_proto_hash=$(grep "^proto_hash=" "$fingerprint_file" | cut -d'=' -f2)
    local expected_buf_hash=$(grep "^buf_hash=" "$fingerprint_file" | cut -d'=' -f2)

    if [ "$current_proto_hash" != "$expected_proto_hash" ]; then
        log_error "Proto hash mismatch for $service_dir"
        log_error "  Expected: $expected_proto_hash"
        log_error "  Actual:   $current_proto_hash"
        return 1
    fi

    if [ "$current_buf_hash" != "$expected_buf_hash" ]; then
        log_error "buf.yaml hash mismatch for $service_dir"
        log_error "  Expected: $expected_buf_hash"
        log_error "  Actual:   $current_buf_hash"
        return 1
    fi

    log_info "Proto fingerprint verified for $service_dir"
    return 0
}

# OpenAPI fingerprint を検証
verify_openapi_fingerprint() {
    local service_dir="$1"
    local openapi_file="$service_dir/openapi/openapi.yaml"
    local fingerprint_file="$service_dir/openapi/openapi.fingerprint"

    if [ ! -f "$openapi_file" ]; then
        return 0
    fi

    if [ ! -f "$fingerprint_file" ]; then
        log_warn "No fingerprint file for $service_dir"
        return 0
    fi

    log_info "Verifying OpenAPI fingerprint for $service_dir"

    local expected=$(cat "$fingerprint_file")
    local actual=$(compute_hash "$openapi_file")

    if [ "$expected" != "$actual" ]; then
        log_error "OpenAPI fingerprint mismatch for $service_dir"
        log_error "  Expected: $expected"
        log_error "  Actual:   $actual"
        return 1
    fi

    log_info "OpenAPI fingerprint verified for $service_dir"
    return 0
}

# サービスディレクトリを検出
find_services() {
    local services=()

    # Framework サービス
    for dir in "$ROOT_DIR"/framework/backend/rust/services/*/; do
        if [ -d "$dir" ]; then
            services+=("${dir%/}")
        fi
    done

    # Feature サービス
    if [ -d "$ROOT_DIR/feature/backend/rust" ]; then
        for dir in "$ROOT_DIR"/feature/backend/rust/*/; do
            if [ -d "$dir" ]; then
                services+=("${dir%/}")
            fi
        done
    fi

    echo "${services[@]}"
}

# 全サービスの fingerprint を生成
generate_all_fingerprints() {
    local services=($(find_services))

    for service in "${services[@]}"; do
        generate_proto_fingerprint "$service"
        generate_openapi_fingerprint "$service"
    done

    log_info "All fingerprints generated!"
}

# 全サービスの fingerprint を検証
verify_all_fingerprints() {
    local errors=0
    local services=($(find_services))

    for service in "${services[@]}"; do
        if ! verify_proto_fingerprint "$service"; then
            ((errors++))
        fi
        if ! verify_openapi_fingerprint "$service"; then
            ((errors++))
        fi
    done

    if [ $errors -gt 0 ]; then
        log_error "Fingerprint verification failed for $errors check(s)"
        log_error ""
        log_error "This means the contract files (proto or OpenAPI) have changed"
        log_error "but the fingerprint files have not been updated."
        log_error ""
        log_error "To fix this, run:"
        log_error "  ./scripts/gen-check.sh fingerprint"
        log_error ""
        log_error "Then commit the updated fingerprint files."
        return 1
    fi

    log_info "All fingerprints verified!"
}

# 生成状態をチェック
check_generation_status() {
    log_info "Checking generation status..."

    local services=($(find_services))
    local needs_regen=()

    for service in "${services[@]}"; do
        local service_name=$(basename "$service")

        # Proto チェック
        if [ -d "$service/proto" ]; then
            local fingerprint_file="$service/gen/.k1s0-gen.sha256"
            if [ ! -f "$fingerprint_file" ]; then
                needs_regen+=("$service_name (proto: no fingerprint)")
            elif ! verify_proto_fingerprint "$service" 2>/dev/null; then
                needs_regen+=("$service_name (proto: changed)")
            fi
        fi

        # OpenAPI チェック
        if [ -f "$service/openapi/openapi.yaml" ]; then
            local fingerprint_file="$service/openapi/openapi.fingerprint"
            if [ ! -f "$fingerprint_file" ]; then
                needs_regen+=("$service_name (openapi: no fingerprint)")
            elif ! verify_openapi_fingerprint "$service" 2>/dev/null; then
                needs_regen+=("$service_name (openapi: changed)")
            fi
        fi
    done

    if [ ${#needs_regen[@]} -gt 0 ]; then
        log_warn "The following services need regeneration:"
        for item in "${needs_regen[@]}"; do
            echo "  - $item"
        done
        return 1
    fi

    log_info "All services are up to date!"
}

# メイン
main() {
    case "${1:-help}" in
        fingerprint|fp)
            generate_all_fingerprints
            ;;
        verify)
            verify_all_fingerprints
            ;;
        status)
            check_generation_status
            ;;
        *)
            echo "Usage: $0 {fingerprint|verify|status}"
            echo ""
            echo "Commands:"
            echo "  fingerprint, fp  Generate fingerprint files for all services"
            echo "  verify           Verify fingerprint files match source files"
            echo "  status           Check which services need regeneration"
            echo ""
            echo "Fingerprint files:"
            echo "  gen/.k1s0-gen.sha256     - Proto fingerprint"
            echo "  openapi/openapi.fingerprint - OpenAPI fingerprint"
            exit 1
            ;;
    esac
}

main "$@"
