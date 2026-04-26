#!/usr/bin/env bash
#
# tools/sparse/verify.sh — 10 役 cone 定義の構文・整合性検証
#
# 設計書:
#   - docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/02_役割別cone定義.md
#   - docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/04_役割切替運用.md
# IMP-DIR-SPARSE-127: 役割別 cone 切替
#
# 役割:
#   1. 10 役すべての .sparse-checkout/roles/<role>.txt が存在することを検証
#   2. 各 cone 定義の各行が cone mode の文法（先頭 /、ディレクトリのみ、または `/*`）を満たすか検証
#   3. include パスが現リポジトリに実在するディレクトリを指すか検証（プレースホルダ許容）
#   4. checkout-role.sh の --list と整合するか検証
#
# CI から呼ぶことを想定。失敗時は非ゼロで終了し、PR を fail させる。
#
# Usage:
#   tools/sparse/verify.sh                 # 全 role 検証
#   tools/sparse/verify.sh --strict        # プレースホルダ未存在も fail
#   tools/sparse/verify.sh --json          # JSON 形式で結果出力
#   tools/sparse/verify.sh <role>          # 単一 role のみ検証

set -euo pipefail

# 期待する役（checkout-role.sh の ROLES 配列と一致させる）
EXPECTED_ROLES=(
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

STRICT=0
JSON=0
TARGET_ROLE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --strict) STRICT=1; shift ;;
        --json) JSON=1; shift ;;
        -h|--help)
            sed -n '3,22p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        -*)
            echo "[error] 未知のオプション: $1" >&2
            exit 1
            ;;
        *)
            TARGET_ROLE="$1"
            shift
            ;;
    esac
done

# ANSI カラー（JSON 出力時は無効化）
if [[ "${JSON}" == "1" ]] || [[ ! -t 1 ]]; then
    C_GREEN=""; C_RED=""; C_YELLOW=""; C_RESET=""
else
    C_GREEN=$'\033[32m'; C_RED=$'\033[31m'; C_YELLOW=$'\033[33m'; C_RESET=$'\033[0m'
fi

# 結果集計（JSON 用）
declare -a RESULTS=()
RC=0

verify_role() {
    local role="$1"
    local file="${ROLES_DIR}/${role}.txt"
    local errors=()
    local warnings=()

    # 1. ファイル存在
    if [[ ! -f "${file}" ]]; then
        errors+=("cone 定義ファイル未存在: ${file}")
        if [[ "${JSON}" == "1" ]]; then
            RESULTS+=("$(printf '{"role":"%s","status":"error","errors":["cone 定義ファイル未存在"],"warnings":[]}' "$role")")
        else
            echo "${C_RED}[ng]${C_RESET} ${role}"
            for e in "${errors[@]}"; do echo "      ${e}"; done
        fi
        return 1
    fi

    # 2. プロファイル存在（full は対応プロファイルあり）
    if [[ ! -f "${PROFILES_DIR}/${role}/devcontainer.json" ]]; then
        errors+=("対応プロファイル未存在: ${PROFILES_DIR}/${role}/devcontainer.json")
    fi

    # 3. 各行の構文検証
    local line_no=0
    local include_count=0
    while IFS= read -r line; do
        line_no=$((line_no + 1))
        # 空行・コメント行はスキップ
        if [[ -z "${line// }" ]] || [[ "${line}" =~ ^[[:space:]]*# ]]; then
            continue
        fi
        include_count=$((include_count + 1))

        # cone mode は: 先頭 / + ディレクトリパス、または /*（全体ルート）
        if [[ "${line}" == "/*" ]]; then
            # full role のみ許容（other roles で /* が出ると warning）
            if [[ "${role}" != "full" ]]; then
                warnings+=("L${line_no}: \"/*\" は full 以外で使うべきでない")
            fi
            continue
        fi

        # 先頭が / 必須
        if [[ ! "${line}" =~ ^/ ]]; then
            errors+=("L${line_no}: cone mode は先頭 / 必須: ${line}")
            continue
        fi

        # 末尾 / 必須（ディレクトリ指定）
        if [[ ! "${line}" =~ /$ ]]; then
            # CLAUDE.md など個別ファイルは末尾 / なしで OK（cone mode は技術的にディレクトリだが、root の特定ファイルはマッチさせる）
            # ただしファイル名以外で末尾 / が無いものは warning
            if [[ "${line}" =~ \.md$ ]] || [[ "${line}" =~ \.json$ ]] || [[ "${line}" =~ \.toml$ ]] || [[ "${line}" =~ \.yaml$ ]] || [[ "${line}" =~ \.yml$ ]]; then
                : # 個別ファイル指定として許容
            else
                warnings+=("L${line_no}: ディレクトリ指定は末尾 / 推奨: ${line}")
            fi
        fi

        # 4. パス実在チェック
        local rel_path="${line#/}"
        rel_path="${rel_path%/}"
        if [[ -n "${rel_path}" ]] && [[ ! -e "${REPO_ROOT}/${rel_path}" ]]; then
            if [[ "${STRICT}" == "1" ]]; then
                errors+=("L${line_no}: パス未存在: ${line}")
            else
                warnings+=("L${line_no}: パス未存在（プレースホルダ）: ${line}")
            fi
        fi
    done < "${file}"

    # 結果出力
    local status="ok"
    if [[ ${#errors[@]} -gt 0 ]]; then
        status="error"
        RC=1
    elif [[ ${#warnings[@]} -gt 0 ]]; then
        status="warn"
    fi

    if [[ "${JSON}" == "1" ]]; then
        local err_json="" warn_json=""
        for e in "${errors[@]:-}"; do
            [[ -z "${e}" ]] && continue
            err_json+="\"${e//\"/\\\"}\","
        done
        err_json="[${err_json%,}]"
        for w in "${warnings[@]:-}"; do
            [[ -z "${w}" ]] && continue
            warn_json+="\"${w//\"/\\\"}\","
        done
        warn_json="[${warn_json%,}]"
        RESULTS+=("$(printf '{"role":"%s","status":"%s","includes":%d,"errors":%s,"warnings":%s}' \
            "${role}" "${status}" "${include_count}" "${err_json}" "${warn_json}")")
    else
        case "${status}" in
            ok)    echo "${C_GREEN}[ok]${C_RESET}    ${role}  (includes=${include_count})" ;;
            warn)  echo "${C_YELLOW}[warn]${C_RESET}  ${role}  (includes=${include_count})" ;;
            error) echo "${C_RED}[ng]${C_RESET}    ${role}  (includes=${include_count})"; RC=1 ;;
        esac
        for e in "${errors[@]:-}"; do
            [[ -n "${e}" ]] && echo "        ${C_RED}error:${C_RESET} ${e}"
        done
        for w in "${warnings[@]:-}"; do
            [[ -n "${w}" ]] && echo "        ${C_YELLOW}warn:${C_RESET}  ${w}"
        done
    fi

    return 0
}

# checkout-role.sh の --list 出力と期待 role の整合
verify_role_list() {
    local list_output
    list_output="$("${REPO_ROOT}/tools/sparse/checkout-role.sh" --list 2>/dev/null | grep -E '^\s+- ' | sed 's/^\s\+- //')"
    local missing=()
    for r in "${EXPECTED_ROLES[@]}"; do
        if ! echo "${list_output}" | grep -q "^${r}$"; then
            missing+=("${r}")
        fi
    done
    if [[ ${#missing[@]} -gt 0 ]]; then
        if [[ "${JSON}" != "1" ]]; then
            echo "${C_RED}[ng]${C_RESET} checkout-role.sh --list に未登録の role: ${missing[*]}"
        fi
        RC=1
    fi
}

# 単一 role 指定
if [[ -n "${TARGET_ROLE}" ]]; then
    found=0
    for r in "${EXPECTED_ROLES[@]}"; do
        [[ "${r}" == "${TARGET_ROLE}" ]] && found=1
    done
    if [[ "${found}" == "0" ]]; then
        echo "[error] 不明な role: ${TARGET_ROLE}" >&2
        echo "  期待される role: ${EXPECTED_ROLES[*]}" >&2
        exit 1
    fi
    verify_role "${TARGET_ROLE}"
    exit ${RC}
fi

# 全 role 検証
if [[ "${JSON}" != "1" ]]; then
    echo "## sparse-checkout cone 検証（全 10 役）"
fi

for role in "${EXPECTED_ROLES[@]}"; do
    verify_role "${role}" || true
done

verify_role_list

# JSON 出力
if [[ "${JSON}" == "1" ]]; then
    printf '{"results":['
    local_first=1
    for r in "${RESULTS[@]}"; do
        if [[ "${local_first}" == "1" ]]; then
            local_first=0
        else
            printf ','
        fi
        printf '%s' "${r}"
    done
    printf '],"exit_code":%d}\n' "${RC}"
fi

exit ${RC}
