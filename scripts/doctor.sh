#!/bin/bash
# scripts/doctor.sh
# k1s0開発環境の自己診断スクリプト
# 各チェック項目をOK/WARN/ERRORで色分け表示する
# 冪等・読み取りのみで副作用なし

set -euo pipefail

# =============================================================================
# 色定数・カウンター初期化
# =============================================================================
# ANSIエスケープコードによる色分け（端末が対応していない場合は空文字）
if [[ -t 1 ]]; then
    COLOR_OK="\033[32m"
    COLOR_WARN="\033[33m"
    COLOR_ERROR="\033[31m"
    COLOR_RESET="\033[0m"
    COLOR_BOLD="\033[1m"
else
    COLOR_OK=""
    COLOR_WARN=""
    COLOR_ERROR=""
    COLOR_RESET=""
    COLOR_BOLD=""
fi

# エラー・警告カウンター
COUNT_ERROR=0
COUNT_WARN=0

# =============================================================================
# .tool-versions からバージョン要件を読み込む
# =============================================================================
# リポジトリルートの .tool-versions がバージョンの唯一の情報源
REPO_ROOT_FOR_VERSIONS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOL_VERSIONS_FILE="${REPO_ROOT_FOR_VERSIONS}/.tool-versions"

# .tool-versions からバージョンを取得するヘルパー関数
# 引数: ツール名（例: rust, golang, nodejs）
get_required_version() {
    local tool="$1"
    if [[ -f "$TOOL_VERSIONS_FILE" ]]; then
        grep "^${tool} " "$TOOL_VERSIONS_FILE" | awk '{print $2}' | head -1
    else
        echo ""
    fi
}

# .tool-versions から各ツールの要件バージョンを取得する
REQ_RUST=$(get_required_version "rust")
REQ_GO=$(get_required_version "golang")
REQ_NODE=$(get_required_version "nodejs")
REQ_BUF=$(get_required_version "buf")

# フォールバック（.tool-versions が存在しない場合のハードコード値）
REQ_RUST="${REQ_RUST:-1.93.0}"
REQ_GO="${REQ_GO:-1.24.0}"
REQ_NODE="${REQ_NODE:-22.0.0}"
REQ_BUF="${REQ_BUF:-1.47.2}"

# =============================================================================
# 出力ヘルパー関数
# =============================================================================
# OKメッセージを緑色で表示する
ok() {
    echo -e "${COLOR_OK}[OK]   ${COLOR_RESET} $*"
}

# WARNメッセージを黄色で表示する
warn() {
    echo -e "${COLOR_WARN}[WARN] ${COLOR_RESET} $*"
    COUNT_WARN=$((COUNT_WARN + 1))
}

# ERRORメッセージを赤色で表示する
error() {
    echo -e "${COLOR_ERROR}[ERROR]${COLOR_RESET} $*"
    COUNT_ERROR=$((COUNT_ERROR + 1))
}

# コマンドが存在するか確認するヘルパー
has() { command -v "$1" &>/dev/null; }

# セマンティックバージョンを数値に変換して比較する（major.minor.patch → MMmmPP）
# 引数1: チェック対象バージョン文字列（例: "1.93.0"）
# 引数2: 最低要求バージョン文字列（例: "1.93"）
version_ge() {
    local actual="$1"
    local required="$2"
    # バージョン文字列を整数列に変換し辞書順で比較する
    # printf で左詰めゼロパディングしてソート可能な文字列を生成する
    local actual_norm required_norm
    actual_norm=$(echo "$actual" | awk -F. '{ printf "%05d%05d%05d", $1, $2, $3 }')
    required_norm=$(echo "$required" | awk -F. '{ printf "%05d%05d%05d", $1, $2, $3 }')
    # bash の [[ ]] では >= は文字列比較として未定義のため、> と == を明示的に組み合わせる（CLI-01 監査対応）
    [[ "$actual_norm" > "$required_norm" ]] || [[ "$actual_norm" == "$required_norm" ]]
}

# =============================================================================
# ヘッダー表示
# =============================================================================
echo ""
echo -e "${COLOR_BOLD}k1s0 開発環境診断 (doctor)${COLOR_RESET}"
echo "============================"

# =============================================================================
# チェック1: Git設定
# =============================================================================
echo ""
echo "--- Git 設定 ---"

# core.autocrlf が true の場合、改行コード変換によるシェルスクリプト破損を引き起こす
autocrlf=$(git config --global core.autocrlf 2>/dev/null || echo "unset")
if [[ "$autocrlf" == "true" ]]; then
    error "Git: core.autocrlf = true → 修正: git config --global core.autocrlf input"
elif [[ "$autocrlf" == "input" || "$autocrlf" == "false" ]]; then
    ok "Git: core.autocrlf = ${autocrlf}"
else
    warn "Git: core.autocrlf が未設定です → 推奨: git config --global core.autocrlf input"
fi

# core.longpaths が true の場合、Windows でのパス長制限（260文字）を回避できる
longpaths=$(git config --global core.longpaths 2>/dev/null || echo "unset")
if [[ "$longpaths" == "true" ]]; then
    ok "Git: core.longpaths = true"
else
    warn "Git: core.longpaths が未設定または false です → 推奨 (Windows): git config --global core.longpaths true"
fi

# =============================================================================
# チェック2: 実行環境
# =============================================================================
echo ""
echo "--- 実行環境 ---"

# WSL2上での実行を検出するための指標ファイルをチェックする
IS_WSL2=false
if [[ -f "/proc/sys/fs/binfmt_misc/WSLInterop" ]]; then
    IS_WSL2=true
    ok "WSL2: WSL2環境上で実行されています"

    # WSL2でWindows側ドライブ（/mnt/c など）にリポジトリがあるとI/Oが大幅に遅くなる
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [[ "$SCRIPT_DIR" == /mnt/* ]]; then
        error "WSL2: ファイルが /mnt/ 配下にあります (${SCRIPT_DIR})"
        error "       → I/O遅延の原因です。WSL2ファイルシステム (~/) にクローンしてください"
    else
        ok "WSL2: ファイルはWSL2ファイルシステム上にあります (${SCRIPT_DIR})"
    fi
else
    ok "実行環境: WSL2以外の環境で実行されています（ネイティブLinux / Git Bash / devcontainer）"
fi

# =============================================================================
# チェック3: 必須ツールのバージョンチェック
# =============================================================================
echo ""
echo "--- 必須ツール ---"

# Rust: .tool-versions で要求されるバージョン以上が必要
if has rustc; then
    rust_ver=$(rustc --version | awk '{print $2}')
    if version_ge "$rust_ver" "${REQ_RUST}"; then
        ok "Rust: rustc ${rust_ver}"
    else
        warn "Rust: rustc ${rust_ver} (期待: ${REQ_RUST}以上) → rustup update で更新してください"
    fi
else
    warn "Rust: rustc が見つかりません → https://rustup.rs/ からインストールしてください"
fi

# Go: .tool-versions で要求されるバージョン以上が必要
if has go; then
    go_ver=$(go version | awk '{print $3}' | sed 's/go//')
    if version_ge "$go_ver" "${REQ_GO}"; then
        ok "Go: go ${go_ver}"
    else
        warn "Go: go ${go_ver} (期待: ${REQ_GO}以上) → https://go.dev/dl/ からインストールしてください"
    fi
else
    warn "Go: go が見つかりません → https://go.dev/dl/ からインストールしてください"
fi

# Node.js: .tool-versions で要求されるバージョン以上が必要
if has node; then
    node_ver=$(node --version | sed 's/v//')
    if version_ge "$node_ver" "${REQ_NODE}"; then
        ok "Node.js: v${node_ver}"
    else
        warn "Node.js: v${node_ver} (期待: ${REQ_NODE}以上) → https://nodejs.org/ またはnvmでインストールしてください"
    fi
else
    warn "Node.js: node が見つかりません → https://nodejs.org/ またはnvmでインストールしてください"
fi

# pnpm: TypeScriptワークスペース管理に必要
if has pnpm; then
    pnpm_ver=$(pnpm --version 2>/dev/null)
    ok "pnpm: ${pnpm_ver}"
else
    warn "pnpm: pnpm が見つかりません → corepack enable pnpm または npm install -g pnpm"
fi

# k1s0 CLI: プロジェクト管理・コード生成ツール
if has k1s0; then
    k1s0_ver=$(k1s0 --version 2>/dev/null | head -1 || echo "(version不明)")
    ok "k1s0 CLI: ${k1s0_ver}"
else
    warn "k1s0 CLI: k1s0 が見つかりません → cargo install --path CLI/crates/k1s0-cli"
fi

# Docker: 存在確認のみ（バージョン問わずdocker composeが動けば良い）
if has docker; then
    docker_ver=$(docker --version 2>/dev/null | awk '{print $3}' | tr -d ',')
    ok "Docker: ${docker_ver}"
else
    warn "Docker: docker が見つかりません → https://docs.docker.com/get-docker/ からインストールしてください"
fi

# just: 存在確認のみ（justfileの実行に必要）
if has just; then
    just_ver=$(just --version 2>/dev/null | awk '{print $2}')
    ok "just: ${just_ver}"
else
    warn "just: just が見つかりません → https://just.systems/install.sh でインストールしてください"
fi

# buf: .tool-versions で要求されるバージョン以上が必要
if has buf; then
    buf_ver=$(buf --version 2>/dev/null)
    if version_ge "$buf_ver" "${REQ_BUF}"; then
        ok "buf: ${buf_ver}"
    else
        warn "buf: ${buf_ver} (期待: ${REQ_BUF}以上) → scripts/setup-wsl.sh を参考に更新してください"
    fi
else
    warn "buf: buf が見つかりません → scripts/setup-wsl.sh を参考にインストールしてください"
fi

# =============================================================================
# チェック4: Dockerデーモンチェック
# =============================================================================
echo ""
echo "--- Docker デーモン ---"

# dockerコマンドが存在する場合のみデーモン接続を確認する
if has docker; then
    # docker infoの終了コードとエラー出力でデーモン状態を判定する
    docker_info_output=$(docker info 2>&1)
    docker_info_exit=$?
    if [[ $docker_info_exit -eq 0 ]]; then
        ok "Docker: Dockerデーモンに接続できます"
        # Docker メモリ割り当てが推奨値（8GB）以上かチェックする
        # infra+systemプロファイル全起動で5GB以上消費するため8GB以上を推奨
        docker_mem_bytes=$(docker info --format '{{.MemTotal}}' 2>/dev/null || echo "0")
        if [[ "$docker_mem_bytes" -gt 0 ]]; then
            docker_mem_gb=$(echo "$docker_mem_bytes / 1073741824" | bc 2>/dev/null || echo "0")
            if [[ "$docker_mem_gb" -ge 8 ]]; then
                ok "Docker: メモリ ${docker_mem_gb}GB (推奨: 8GB以上)"
            else
                warn "Docker: メモリ ${docker_mem_gb}GB (推奨: 8GB以上) → Docker Desktop の Settings → Resources でメモリを増やしてください"
            fi
        fi
    elif echo "$docker_info_output" | grep -qi "permission denied"; then
        error "Docker: 権限エラーでDockerデーモンに接続できません → sudo usermod -aG docker \$USER を実行し再ログインしてください"
    else
        warn "Docker: Dockerデーモンに接続できません (docker info が失敗) → Dockerを起動してください"
    fi
else
    warn "Docker: docker コマンドが存在しないためデーモンチェックをスキップします"
fi

# =============================================================================
# チェック5: ポート競合チェック
# =============================================================================
echo ""
echo "--- ポート競合チェック ---"

# ssコマンドまたはnetstatコマンドのいずれかを使用してリスニングポートを取得する
get_listening_ports() {
    if has ss; then
        ss -tlnp 2>/dev/null
    elif has netstat; then
        netstat -tlnp 2>/dev/null
    else
        echo ""
    fi
}

# ポート競合を確認する関数（引数: ポート番号、サービス名）
check_port() {
    local port="$1"
    local service="$2"
    local ports_output
    ports_output=$(get_listening_ports)
    if [[ -z "$ports_output" ]]; then
        # ssもnetstatも使えない場合はスキップ
        return
    fi
    # ポート番号がリスニングポートの一覧に含まれているかを確認する
    if echo "$ports_output" | grep -qE ":${port}\s"; then
        warn "ポート ${port} (${service}) は既に使用中です → docker compose 起動時に競合する可能性があります"
    else
        ok "ポート ${port} (${service}): 空き"
    fi
}

# インフラサービスポートを順番にチェックする
check_port 5432 "PostgreSQL"
check_port 3306 "MySQL"
check_port 6379 "Redis"
check_port 6380 "Redis session"
check_port 9092 "Kafka"
check_port 8090 "Kafka UI"
check_port 8081 "Schema Registry"
check_port 8180 "Keycloak"
check_port 8200 "Vault"

# =============================================================================
# チェック6: k1s0ワークスペース検出
# =============================================================================
echo ""
echo "--- k1s0 ワークスペース ---"

# スクリプトの位置からリポジトリルートを特定する（scripts/doctor.shを想定）
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# regions/ ディレクトリはモノリポの中核構造であることを確認する
if [[ -d "${REPO_ROOT}/regions" ]]; then
    ok "ワークスペース: regions/ が存在します (${REPO_ROOT}/regions)"
else
    error "ワークスペース: regions/ が見つかりません → k1s0リポジトリのルートディレクトリで実行してください"
fi

# infra/helm/services/ ディレクトリはKubernetesデプロイ構成の存在を確認する
if [[ -d "${REPO_ROOT}/infra/helm/services" ]]; then
    ok "ワークスペース: infra/helm/services/ が存在します"
else
    warn "ワークスペース: infra/helm/services/ が見つかりません → sparse-checkoutで除外されている可能性があります"
fi

# k1s0.yaml はCLI設定ファイルとして存在する場合にのみ確認する
if [[ -f "${REPO_ROOT}/k1s0.yaml" ]]; then
    ok "ワークスペース: k1s0.yaml が存在します"
else
    warn "ワークスペース: k1s0.yaml が見つかりません → k1s0 init を実行して k1s0.yaml を生成してください"
fi

# =============================================================================
# 総合評価
# =============================================================================
echo ""
echo "============================"

# エラー・警告の件数に基づいて総合評価を表示する
if [[ "$COUNT_ERROR" -eq 0 && "$COUNT_WARN" -eq 0 ]]; then
    echo -e "${COLOR_OK}${COLOR_BOLD}✅ 開発環境は正常です${COLOR_RESET}"
elif [[ "$COUNT_ERROR" -eq 0 ]]; then
    echo -e "${COLOR_WARN}${COLOR_BOLD}⚠️  警告があります (WARN: ${COUNT_WARN}件)${COLOR_RESET}"
    echo "   警告を確認して必要に応じて対応してください。"
else
    echo -e "${COLOR_ERROR}${COLOR_BOLD}❌ 問題があります。ERRORを修正してください${COLOR_RESET}"
    echo "   結果: ERRORが${COUNT_ERROR}件、WARNが${COUNT_WARN}件あります"
    echo ""
    # ERRORが存在する場合は非ゼロ終了コードで終了してCIでの検出を可能にする
    exit 1
fi
echo ""
