#!/usr/bin/env bash
#
# tools/sparse/checkout-role.sh — 役割別 cone 切替スクリプト
#
# 設計書:
#   - docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md
#   - docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/04_役割切替運用.md
#   - docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# IMP-DIR-SPARSE-127: 役割別 cone 切替
# IMP-DEV-DC-010: .devcontainer/ と tools/devcontainer/profiles/ の二層構造を symlink で接続
#
# 役割を引数に取り、以下を実行する:
#   1. .sparse-checkout/roles/<role>.txt を読み出し git sparse-checkout set
#   2. .devcontainer/devcontainer.json を tools/devcontainer/profiles/<role>/devcontainer.json
#      への相対 symlink に張り替える
#
# Usage:
#   tools/sparse/checkout-role.sh <role>                 # 切替実行
#   tools/sparse/checkout-role.sh -m <r1>,<r2>,...       # 兼任マージ（symlink は最初の role）
#   tools/sparse/checkout-role.sh <role> --verify        # 現在の cone と symlink の整合チェック
#   tools/sparse/checkout-role.sh <role> --dry-run       # 変更内容の表示のみ
#   tools/sparse/checkout-role.sh --list                 # 利用可能 role の一覧

set -euo pipefail

ROLES=(
    tier1-rust-dev
    tier1-go-dev
    tier2-dev
    tier3-web-dev
    tier3-native-dev
    platform-cli-dev
    sdk-dev
    infra-ops
    docs-writer
    full
)

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
ROLES_DIR="${REPO_ROOT}/.sparse-checkout/roles"
PROFILES_DIR="${REPO_ROOT}/tools/devcontainer/profiles"
DEVCONTAINER_LINK="${REPO_ROOT}/.devcontainer/devcontainer.json"

usage() {
    sed -n '3,22p' "$0" | sed 's/^# \{0,1\}//'
    exit 1
}

list_roles() {
    printf '利用可能な role:\n'
    for r in "${ROLES[@]}"; do printf '  - %s\n' "$r"; done
}

is_valid_role() {
    local role="$1"
    for r in "${ROLES[@]}"; do
        [[ "$r" == "$role" ]] && return 0
    done
    return 1
}

ensure_repo_state() {
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "[error] git リポジトリ内で実行してください" >&2
        exit 2
    fi
    if ! git config --get core.sparseCheckoutCone >/dev/null 2>&1; then
        echo "[info] cone mode を有効化します"
        git sparse-checkout init --cone
    fi
}

apply_cone() {
    local merged_file="$1"
    if [[ "${DRY_RUN:-0}" == "1" ]]; then
        echo "[dry-run] git sparse-checkout set (from ${merged_file})"
        sed 's/^/  /' "$merged_file"
        return 0
    fi
    # cone mode は先頭 / で始まるディレクトリパスのみ受理。コメント行と空行を除外。
    grep -v '^\s*#' "$merged_file" | grep -v '^\s*$' \
        | git sparse-checkout set --stdin
}

apply_symlink() {
    local role="$1"
    local target_rel="../tools/devcontainer/profiles/${role}/devcontainer.json"
    local target_abs="${PROFILES_DIR}/${role}/devcontainer.json"
    if [[ ! -f "${target_abs}" ]]; then
        echo "[error] プロファイルが存在しません: ${target_abs}" >&2
        exit 3
    fi
    if [[ "${DRY_RUN:-0}" == "1" ]]; then
        echo "[dry-run] ln -sf ${target_rel} ${DEVCONTAINER_LINK}"
        return 0
    fi
    mkdir -p "$(dirname "${DEVCONTAINER_LINK}")"
    # 既存ファイルが symlink でない場合はバックアップして退避
    if [[ -e "${DEVCONTAINER_LINK}" && ! -L "${DEVCONTAINER_LINK}" ]]; then
        local backup
        backup="${DEVCONTAINER_LINK}.bak.$(date +%s)"
        mv "${DEVCONTAINER_LINK}" "${backup}"
        echo "[info] 既存の非 symlink ファイルを退避: ${backup}"
    fi
    ln -sfn "${target_rel}" "${DEVCONTAINER_LINK}"
    echo "[ok] .devcontainer/devcontainer.json -> ${target_rel}"
}

verify_only() {
    local role="$1"
    local rc=0
    echo "## verify role=${role}"
    # symlink check
    if [[ -L "${DEVCONTAINER_LINK}" ]]; then
        local link_target
        link_target="$(readlink "${DEVCONTAINER_LINK}")"
        local expected="../tools/devcontainer/profiles/${role}/devcontainer.json"
        if [[ "${link_target}" == "${expected}" ]]; then
            echo "  [ok] symlink target = ${link_target}"
        else
            echo "  [ng] symlink target != expected"
            echo "       actual:   ${link_target}"
            echo "       expected: ${expected}"
            rc=1
        fi
    else
        echo "  [ng] .devcontainer/devcontainer.json は symlink ではない"
        rc=1
    fi
    # cone check（簡易: include パスが role 定義に含まれているか）
    if [[ -f "${ROLES_DIR}/${role}.txt" ]]; then
        local current
        current="$(git sparse-checkout list 2>/dev/null | sort -u || true)"
        local expected_cone
        expected_cone="$(grep -v '^\s*#' "${ROLES_DIR}/${role}.txt" | grep -v '^\s*$' | sort -u || true)"
        if [[ "${current}" == "${expected_cone}" ]]; then
            echo "  [ok] sparse-checkout cone matches"
        else
            echo "  [ng] sparse-checkout cone differs from ${role}.txt"
            diff <(echo "${current}") <(echo "${expected_cone}") | sed 's/^/    /'
            rc=1
        fi
    fi
    return ${rc}
}

# --- argument parsing -----------------------------------------------------

DRY_RUN=0
VERIFY=0
MERGE_ROLES=""
ROLE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help) usage ;;
        --list) list_roles; exit 0 ;;
        --dry-run) DRY_RUN=1; shift ;;
        --verify) VERIFY=1; shift ;;
        -m|--merge) MERGE_ROLES="$2"; shift 2 ;;
        -*)
            echo "[error] 未知のオプション: $1" >&2
            usage
            ;;
        *)
            if [[ -z "${ROLE}" ]]; then
                ROLE="$1"
            else
                echo "[error] role は 1 つだけ指定できます。兼任は -m を使用" >&2
                exit 1
            fi
            shift
            ;;
    esac
done

# --- main -----------------------------------------------------------------

if [[ -n "${MERGE_ROLES}" ]]; then
    # 兼任モード: カンマ区切りの role を全て検証 → cone をマージ → 先頭 role の symlink を張る
    IFS=',' read -ra ROLE_LIST <<< "${MERGE_ROLES}"
    for r in "${ROLE_LIST[@]}"; do
        if ! is_valid_role "$r"; then
            echo "[error] 不明な role: $r" >&2
            list_roles
            exit 1
        fi
    done
    ensure_repo_state
    MERGED_TMP="$(mktemp)"
    trap 'rm -f "${MERGED_TMP}"' EXIT
    for r in "${ROLE_LIST[@]}"; do
        cat "${ROLES_DIR}/${r}.txt"
    done | sort -u > "${MERGED_TMP}"
    apply_cone "${MERGED_TMP}"
    PRIMARY_ROLE="${ROLE_LIST[0]}"
    apply_symlink "${PRIMARY_ROLE}"
    echo "[ok] 兼任モードで切替完了: ${MERGE_ROLES}（symlink primary=${PRIMARY_ROLE}）"
    exit 0
fi

if [[ -z "${ROLE}" ]]; then
    list_roles
    exit 1
fi

if ! is_valid_role "${ROLE}"; then
    echo "[error] 不明な role: ${ROLE}" >&2
    list_roles
    exit 1
fi

if [[ "${VERIFY}" == "1" ]]; then
    verify_only "${ROLE}"
    exit $?
fi

ensure_repo_state
apply_cone "${ROLES_DIR}/${ROLE}.txt"
apply_symlink "${ROLE}"
echo "[ok] role=${ROLE} に切替えました"
