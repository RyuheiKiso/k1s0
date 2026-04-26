#!/usr/bin/env bash
#
# tools/devcontainer/doctor.sh — Dev Container 役別 toolchain 診断
#
# 設計書:
#   - docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
#   - docs/05_実装/00_ディレクトリ設計/70_共通資産/05_devcontainer配置.md
# IMP-DEV-DC-016: postCreate / doctor が toolchain 期待値を保証
#
# 役割:
#   1. 役を引数（または K1S0_DEV_ROLE 環境変数 / readlink .devcontainer/devcontainer.json）から決定
#   2. その役で **必須** とされるツールが PATH 上に存在するか
#   3. 主要ツールの最低バージョンを満たすか（Rust/Go/Node/.NET 等）
#   4. CI 用に matrix で並列実行できる
#
# postCreate.sh は「環境構築の通過点（fail-soft）」で軽い列挙を行う。
# 本 doctor.sh は「現状診断（fail-hard）」で CI からも呼ばれる。
#
# Usage:
#   tools/devcontainer/doctor.sh                  # 自動検出（symlink から role 判定）
#   tools/devcontainer/doctor.sh tier1-rust-dev   # 役を明示
#   tools/devcontainer/doctor.sh --all            # 全 10 役の必須ツール集合を一括確認
#   tools/devcontainer/doctor.sh --json           # 機械可読出力
#   K1S0_DEV_ROLE=tier1-go-dev tools/devcontainer/doctor.sh

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
DEVCONTAINER_LINK="${REPO_ROOT}/.devcontainer/devcontainer.json"

# ----- 役別の期待ツール定義 ---------------------------------------------------
# 形式: required="cmd1 cmd2 ..."  recommended="cmd3 cmd4 ..."
# required は exit 1 の対象、recommended は warn のみ。
# 設計書 (10_DevContainer_10役/01_DevContainer_10役設計.md) のサイズ目標とツール記述に整合。

declare -A REQUIRED
declare -A RECOMMENDED
declare -A MIN_VERSION  # "cmd:semver" 形式で羅列

REQUIRED[docs-writer]="git markdownlint-cli2"
RECOMMENDED[docs-writer]="textlint mmdc pandoc drawio-export"

REQUIRED[tier1-rust-dev]="git rustc cargo buf protoc"
RECOMMENDED[tier1-rust-dev]="cargo-nextest cargo-audit cargo-deny cargo-fuzz mold"
MIN_VERSION[tier1-rust-dev]="rustc:1.83.0"

REQUIRED[tier1-go-dev]="git go buf protoc"
RECOMMENDED[tier1-go-dev]="dapr golangci-lint dlv"
MIN_VERSION[tier1-go-dev]="go:1.22.0"

REQUIRED[tier2-dev]="git dotnet go"
RECOMMENDED[tier2-dev]="temporal buf"
MIN_VERSION[tier2-dev]="dotnet:8.0.0 go:1.22.0"

REQUIRED[tier3-web-dev]="git node pnpm"
RECOMMENDED[tier3-web-dev]="playwright"
MIN_VERSION[tier3-web-dev]="node:20.0.0 pnpm:9.0.0"

REQUIRED[tier3-native-dev]="git dotnet"
RECOMMENDED[tier3-native-dev]="adb"
MIN_VERSION[tier3-native-dev]="dotnet:8.0.0"

REQUIRED[platform-cli-dev]="git rustc cargo node"
RECOMMENDED[platform-cli-dev]="cosign syft"
MIN_VERSION[platform-cli-dev]="rustc:1.83.0 node:20.0.0"

REQUIRED[sdk-dev]="git rustc cargo go dotnet node pnpm buf"
RECOMMENDED[sdk-dev]=""
MIN_VERSION[sdk-dev]="rustc:1.83.0 go:1.22.0 dotnet:8.0.0 node:20.0.0 pnpm:9.0.0"

REQUIRED[infra-ops]="git kubectl helm kustomize"
RECOMMENDED[infra-ops]="kind argocd istioctl flagd tofu k6 dapr cosign syft"

REQUIRED[full]="git rustc cargo go dotnet node pnpm buf protoc kubectl helm"
RECOMMENDED[full]="kustomize kind argocd istioctl flagd tofu k6 dapr cosign syft markdownlint-cli2"
MIN_VERSION[full]="rustc:1.83.0 go:1.22.0 dotnet:8.0.0 node:20.0.0 pnpm:9.0.0"

ALL_ROLES=(docs-writer tier1-rust-dev tier1-go-dev tier2-dev tier3-web-dev tier3-native-dev platform-cli-dev sdk-dev infra-ops full)

# ----- 引数 / 出力モード ------------------------------------------------------

JSON=0
ALL=0
ROLE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --json) JSON=1; shift ;;
        --all) ALL=1; shift ;;
        -h|--help)
            sed -n '3,25p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        -*)
            echo "[error] 未知のオプション: $1" >&2
            exit 1
            ;;
        *)
            ROLE="$1"
            shift
            ;;
    esac
done

# 自動検出: symlink → role
if [[ -z "${ROLE}" ]] && [[ "${ALL}" == "0" ]]; then
    if [[ -n "${K1S0_DEV_ROLE:-}" ]]; then
        ROLE="${K1S0_DEV_ROLE}"
    elif [[ -L "${DEVCONTAINER_LINK}" ]]; then
        target="$(readlink "${DEVCONTAINER_LINK}")"
        # ../tools/devcontainer/profiles/<role>/devcontainer.json から <role> を抜き出す
        ROLE="$(echo "${target}" | sed -E 's|.*/profiles/([^/]+)/devcontainer\.json$|\1|')"
    fi
fi

if [[ "${ALL}" == "0" ]]; then
    if [[ -z "${ROLE}" ]]; then
        echo "[error] 役を判定できません。引数で指定するか K1S0_DEV_ROLE を設定してください" >&2
        exit 1
    fi
    if [[ -z "${REQUIRED[$ROLE]+x}" ]]; then
        echo "[error] 不明な役: ${ROLE}" >&2
        echo "  期待: ${ALL_ROLES[*]}" >&2
        exit 1
    fi
fi

# ANSI カラー
if [[ "${JSON}" == "1" ]] || [[ ! -t 1 ]]; then
    C_GREEN=""; C_RED=""; C_YELLOW=""; C_RESET=""
else
    C_GREEN=$'\033[32m'; C_RED=$'\033[31m'; C_YELLOW=$'\033[33m'; C_RESET=$'\033[0m'
fi

# ----- バージョン抽出ヘルパー -------------------------------------------------

extract_version() {
    local cmd="$1"
    case "${cmd}" in
        rustc)   rustc --version 2>/dev/null | awk '{print $2}' ;;
        cargo)   cargo --version 2>/dev/null | awk '{print $2}' ;;
        go)      go version 2>/dev/null | awk '{print $3}' | sed 's/^go//' ;;
        dotnet)  dotnet --version 2>/dev/null ;;
        node)    node --version 2>/dev/null | sed 's/^v//' ;;
        pnpm)    pnpm --version 2>/dev/null ;;
        buf)     buf --version 2>/dev/null ;;
        kubectl) kubectl version --client -o yaml 2>/dev/null | awk '/gitVersion/ {print $2; exit}' | sed 's/^v//' ;;
        helm)    helm version --short 2>/dev/null | sed 's/^v//;s/+.*//' ;;
        *)       "$cmd" --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?' | head -1 ;;
    esac
}

# semver 比較: $1 が $2 以上なら 0、それ以外 1
version_ge() {
    local a="$1" b="$2"
    [[ "$(printf '%s\n%s' "$b" "$a" | sort -V | head -1)" == "$b" ]]
}

# ----- 1 役分の診断 -----------------------------------------------------------

declare -a JSON_RESULTS=()
TOTAL_RC=0

diagnose_role() {
    local role="$1"
    local req="${REQUIRED[$role]}"
    local rec="${RECOMMENDED[$role]:-}"
    local min_spec="${MIN_VERSION[$role]:-}"

    local missing_req=()
    local missing_rec=()
    local low_version=()
    local ok_count=0

    for cmd in $req; do
        if command -v "$cmd" >/dev/null 2>&1; then
            ok_count=$((ok_count + 1))
        else
            missing_req+=("$cmd")
        fi
    done

    for cmd in $rec; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing_rec+=("$cmd")
        fi
    done

    for spec in $min_spec; do
        local cmd="${spec%%:*}"
        local min="${spec##*:}"
        if command -v "$cmd" >/dev/null 2>&1; then
            local actual
            actual="$(extract_version "$cmd")"
            if [[ -n "${actual}" ]]; then
                if ! version_ge "${actual}" "${min}"; then
                    low_version+=("${cmd}: ${actual} < ${min}")
                fi
            fi
        fi
    done

    local status="ok"
    if [[ ${#missing_req[@]} -gt 0 ]] || [[ ${#low_version[@]} -gt 0 ]]; then
        status="error"
        TOTAL_RC=1
    elif [[ ${#missing_rec[@]} -gt 0 ]]; then
        status="warn"
    fi

    if [[ "${JSON}" == "1" ]]; then
        local mr_json="" mc_json="" lv_json=""
        for x in "${missing_req[@]:-}"; do [[ -n "$x" ]] && mr_json+="\"${x}\","; done
        mr_json="[${mr_json%,}]"
        for x in "${missing_rec[@]:-}"; do [[ -n "$x" ]] && mc_json+="\"${x}\","; done
        mc_json="[${mc_json%,}]"
        for x in "${low_version[@]:-}"; do [[ -n "$x" ]] && lv_json+="\"${x//\"/\\\"}\","; done
        lv_json="[${lv_json%,}]"
        JSON_RESULTS+=("$(printf '{"role":"%s","status":"%s","required_ok":%d,"missing_required":%s,"missing_recommended":%s,"low_version":%s}' \
            "${role}" "${status}" "${ok_count}" "${mr_json}" "${mc_json}" "${lv_json}")")
    else
        case "${status}" in
            ok)    echo "${C_GREEN}[ok]${C_RESET}    ${role}  required=${ok_count}/$(echo "$req" | wc -w)" ;;
            warn)  echo "${C_YELLOW}[warn]${C_RESET}  ${role}  required=${ok_count}/$(echo "$req" | wc -w)" ;;
            error) echo "${C_RED}[ng]${C_RESET}    ${role}  required=${ok_count}/$(echo "$req" | wc -w)" ;;
        esac
        for x in "${missing_req[@]:-}"; do
            [[ -n "$x" ]] && echo "        ${C_RED}missing-required:${C_RESET}    ${x}"
        done
        for x in "${low_version[@]:-}"; do
            [[ -n "$x" ]] && echo "        ${C_RED}version-too-low:${C_RESET}     ${x}"
        done
        for x in "${missing_rec[@]:-}"; do
            [[ -n "$x" ]] && echo "        ${C_YELLOW}missing-recommended:${C_RESET} ${x}"
        done
    fi
}

# ----- main -------------------------------------------------------------------

if [[ "${JSON}" != "1" ]]; then
    echo "## doctor.sh — Dev Container toolchain 診断"
    if [[ "${ALL}" == "1" ]]; then
        echo "## 全 10 役検査モード"
    else
        echo "## role=${ROLE}"
    fi
    echo
fi

if [[ "${ALL}" == "1" ]]; then
    for r in "${ALL_ROLES[@]}"; do
        diagnose_role "${r}" || true
    done
else
    diagnose_role "${ROLE}"
fi

if [[ "${JSON}" == "1" ]]; then
    printf '{"results":['
    first=1
    for r in "${JSON_RESULTS[@]}"; do
        if [[ "${first}" == "1" ]]; then first=0; else printf ','; fi
        printf '%s' "$r"
    done
    printf '],"exit_code":%d}\n' "${TOTAL_RC}"
fi

exit ${TOTAL_RC}
