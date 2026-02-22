#!/bin/bash
# coverage.sh - Rust プロジェクトのテストカバレッジ測定スクリプト
# cargo-tarpaulin を使用して全 Rust ライブラリ・サーバーのカバレッジを測定し、
# HTML レポートを coverage/ ディレクトリに出力する。
#
# 使用方法:
#   ./scripts/coverage.sh           # 全プロジェクトのカバレッジ測定
#   ./scripts/coverage.sh --lib     # ライブラリのみ
#   ./scripts/coverage.sh --server  # サーバーのみ

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
COVERAGE_DIR="${PROJECT_ROOT}/coverage"
REGIONS_DIR="${PROJECT_ROOT}/regions"

# カラー出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# cargo-tarpaulin インストール確認
check_tarpaulin() {
    if ! command -v cargo-tarpaulin &> /dev/null; then
        warn "cargo-tarpaulin がインストールされていません。インストールします..."
        cargo install cargo-tarpaulin
    fi
    info "cargo-tarpaulin $(cargo tarpaulin --version)"
}

# Rust プロジェクトを検出
find_rust_projects() {
    local filter="${1:-all}"
    local projects=()

    if [[ "$filter" == "all" || "$filter" == "lib" ]]; then
        while IFS= read -r cargo_toml; do
            projects+=("$(dirname "$cargo_toml")")
        done < <(find "${REGIONS_DIR}" -path "*/library/rust/*/Cargo.toml" -type f 2>/dev/null)
    fi

    if [[ "$filter" == "all" || "$filter" == "server" ]]; then
        while IFS= read -r cargo_toml; do
            projects+=("$(dirname "$cargo_toml")")
        done < <(find "${REGIONS_DIR}" -path "*/server/rust/*/Cargo.toml" -type f 2>/dev/null)
    fi

    printf '%s\n' "${projects[@]}"
}

# 単一プロジェクトのカバレッジ測定
run_coverage() {
    local project_dir="$1"
    local project_name
    project_name=$(basename "$project_dir")
    local report_dir="${COVERAGE_DIR}/${project_name}"

    mkdir -p "$report_dir"

    info "カバレッジ測定中: ${project_name}"

    if cargo tarpaulin \
        --manifest-path "${project_dir}/Cargo.toml" \
        --out Html Json \
        --output-dir "$report_dir" \
        --skip-clean \
        --timeout 300 \
        --exclude-files "*/tests/*" "*/proto/*" \
        2>&1 | tee "${report_dir}/tarpaulin.log"; then
        info "${project_name}: カバレッジ測定完了"
    else
        warn "${project_name}: カバレッジ測定でエラーが発生しました（ログ: ${report_dir}/tarpaulin.log）"
    fi
}

# サマリーレポート生成
generate_summary() {
    local summary_file="${COVERAGE_DIR}/summary.txt"
    info "サマリーレポート生成中..."

    {
        echo "======================================"
        echo "  k1s0 テストカバレッジサマリー"
        echo "  生成日時: $(date '+%Y-%m-%d %H:%M:%S')"
        echo "======================================"
        echo ""

        for json_file in "${COVERAGE_DIR}"/*/tarpaulin-report.json; do
            if [[ -f "$json_file" ]]; then
                local project_name
                project_name=$(basename "$(dirname "$json_file")")
                local coverage
                coverage=$(python3 -c "
import json, sys
with open('${json_file}') as f:
    data = json.load(f)
covered = sum(1 for f in data.get('files', []) for t in f.get('traces', []) if t.get('stats', {}).get('Line', 0) > 0)
total = sum(len(f.get('traces', [])) for f in data.get('files', []))
print(f'{covered}/{total} ({covered*100//max(total,1)}%)' if total else 'N/A')
" 2>/dev/null || echo "N/A")
                printf "  %-30s %s\n" "$project_name" "$coverage"
            fi
        done

        echo ""
        echo "詳細レポート: ${COVERAGE_DIR}/<project>/tarpaulin-report.html"
    } > "$summary_file"

    cat "$summary_file"
}

# メイン処理
main() {
    local filter="all"

    case "${1:-}" in
        --lib)    filter="lib" ;;
        --server) filter="server" ;;
        --help|-h)
            echo "Usage: $0 [--lib|--server|--help]"
            echo "  --lib     ライブラリプロジェクトのみ"
            echo "  --server  サーバープロジェクトのみ"
            exit 0
            ;;
    esac

    info "k1s0 テストカバレッジ測定を開始します (filter: ${filter})"

    check_tarpaulin

    mkdir -p "$COVERAGE_DIR"

    local projects
    mapfile -t projects < <(find_rust_projects "$filter")

    if [[ ${#projects[@]} -eq 0 ]]; then
        error "Rust プロジェクトが見つかりません"
        exit 1
    fi

    info "${#projects[@]} 個のプロジェクトを検出しました"

    for project in "${projects[@]}"; do
        run_coverage "$project"
    done

    generate_summary

    info "カバレッジ測定完了。レポート: ${COVERAGE_DIR}/"
}

main "$@"
