#!/usr/bin/env bash
# scripts/setup-wsl.sh
# WSL2 ネイティブ開発環境セットアップスクリプト（方法B: 直接インストール）
# WSL2 Ubuntu-24.04 内で実行する: bash scripts/setup-wsl.sh
# 冪等設計: 既にインストール済みのツールはスキップする

set -euo pipefail

# バージョン定数（devcontainer.json / post-create.sh と同期する）
RUST_VERSION="1.93"
GO_VERSION="1.24.0"
NODE_VERSION="22"
BUF_VERSION="1.47.2"
FLUTTER_VERSION="3.24.0"

# 出力ヘルパー
info()  { echo "[INFO]  $*"; }
ok()    { echo "[OK]    $*"; }
warn()  { echo "[WARN]  $*"; }
skip()  { echo "[SKIP]  $* (既にインストール済み)"; }

# コマンドが存在するか確認するヘルパー
has() { command -v "$1" &>/dev/null; }

echo ""
echo "k1s0 WSL2 開発環境セットアップ（方法B: ネイティブインストール）"
echo "========================================================="
echo ""

# ------------------------------------------------------------------
# 前提確認: WSL2 内であることを確認
if [[ -z "${WSL_DISTRO_NAME:-}" ]] && ! grep -qi microsoft /proc/version 2>/dev/null; then
    warn "このスクリプトは WSL2 内での実行を想定しています。"
    warn "WSL2 以外の環境での実行は予期しない動作を引き起こす可能性があります。"
    read -rp "続行しますか? [y/N]: " answer
    [[ "${answer:-N}" =~ ^[Yy]$ ]] || exit 1
fi

# ------------------------------------------------------------------
info "=== 1. 基本パッケージのインストール ==="

sudo apt-get update -qq
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsasl2-dev \
    libz-dev \
    cmake \
    protobuf-compiler \
    curl \
    git \
    unzip \
    patch

ok "基本パッケージをインストールしました"

# ------------------------------------------------------------------
info "=== 2. Docker Engine CE のインストール ==="

if has docker && docker --version &>/dev/null; then
    skip "Docker $(docker --version)"
else
    info "Docker 公式 GPG 鍵と apt リポジトリを設定..."
    sudo apt-get install -y ca-certificates gnupg
    sudo install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg \
        | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    sudo chmod a+r /etc/apt/keyrings/docker.gpg

    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" \
        | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

    sudo apt-get update -qq
    sudo apt-get install -y \
        docker-ce docker-ce-cli containerd.io \
        docker-buildx-plugin docker-compose-plugin

    sudo usermod -aG docker "$USER"
    ok "Docker Engine をインストールしました"
    warn "グループ変更を反映するため WSL を再起動してください: wsl --shutdown"
fi

# systemd が有効な場合は Docker を自動起動に設定
if systemctl is-system-running &>/dev/null 2>&1; then
    sudo systemctl enable docker
    sudo systemctl start docker 2>/dev/null || true
fi

# ------------------------------------------------------------------
info "=== 3. Rust ${RUST_VERSION} のインストール ==="

if has rustc && [[ "$(rustc --version | awk '{print $2}')" == "${RUST_VERSION}"* ]]; then
    skip "Rust $(rustc --version)"
else
    if has rustup; then
        rustup toolchain install "${RUST_VERSION}"
        rustup default "${RUST_VERSION}"
    else
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
            | sh -s -- -y --default-toolchain "${RUST_VERSION}"
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi
    rustup component add clippy rustfmt
    ok "Rust ${RUST_VERSION} をインストールしました"
fi

# ------------------------------------------------------------------
info "=== 4. Go ${GO_VERSION} のインストール ==="

if has go && [[ "$(go version | awk '{print $3}')" == "go${GO_VERSION}" ]]; then
    skip "Go $(go version)"
else
    GO_TARBALL="go${GO_VERSION}.linux-amd64.tar.gz"
    curl -fsSL "https://go.dev/dl/${GO_TARBALL}" -o "/tmp/${GO_TARBALL}"
    sudo rm -rf /usr/local/go
    sudo tar -C /usr/local -xzf "/tmp/${GO_TARBALL}"
    rm "/tmp/${GO_TARBALL}"

    # PATH に追加（未追加の場合のみ）
    if ! grep -q '/usr/local/go/bin' "$HOME/.bashrc"; then
        echo 'export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin' >> "$HOME/.bashrc"
    fi
    export PATH="$PATH:/usr/local/go/bin:$HOME/go/bin"
    ok "Go ${GO_VERSION} をインストールしました"
fi

# ------------------------------------------------------------------
info "=== 5. Node.js ${NODE_VERSION} のインストール ==="

if has node && node --version | grep -q "^v${NODE_VERSION}"; then
    skip "Node.js $(node --version)"
else
    curl -fsSL "https://deb.nodesource.com/setup_${NODE_VERSION}.x" | sudo -E bash -
    sudo apt-get install -y nodejs
    sudo corepack enable pnpm
    ok "Node.js $(node --version) をインストールしました"
fi

# ------------------------------------------------------------------
info "=== 6. Go 開発ツールのインストール ==="

go install golang.org/x/tools/cmd/goimports@v0.31.0
go install github.com/golangci/golangci-lint/cmd/golangci-lint@v1.64.8
go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.36.3
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.5.1
go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@v2.4.1
ok "Go ツールをインストールしました"

# ------------------------------------------------------------------
info "=== 7. buf のインストール ==="

if has buf && [[ "$(buf --version 2>/dev/null)" == "${BUF_VERSION}" ]]; then
    skip "buf ${BUF_VERSION}"
else
    curl -sSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-$(uname -s)-$(uname -m)" \
        -o /tmp/buf
    sudo mv /tmp/buf /usr/local/bin/buf
    sudo chmod +x /usr/local/bin/buf
    ok "buf ${BUF_VERSION} をインストールしました"
fi

# ------------------------------------------------------------------
info "=== 8. just のインストール ==="

if has just; then
    skip "just $(just --version)"
else
    curl -sSL https://just.systems/install.sh | sudo bash -s -- --to /usr/local/bin
    ok "just $(just --version) をインストールしました"
fi

# ------------------------------------------------------------------
info "=== 9. sqlx-cli のインストール ==="

if has sqlx; then
    skip "sqlx $(sqlx --version)"
else
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env" 2>/dev/null || true
    cargo install sqlx-cli --no-default-features --features postgres
    ok "sqlx-cli をインストールしました"
fi

# ------------------------------------------------------------------
info "=== 10. Flutter SDK ${FLUTTER_VERSION} のインストール ==="

if [[ -d "/opt/flutter" ]] && has flutter && [[ "$(flutter --version 2>&1 | awk 'NR==1{print $2}')" == "${FLUTTER_VERSION}"* ]]; then
    skip "Flutter $(flutter --version 2>&1 | head -1)"
else
    info "Flutter SDK を /opt/flutter にクローン中..."
    sudo git clone --depth 1 -b "${FLUTTER_VERSION}" \
        https://github.com/flutter/flutter.git /opt/flutter
    # flutter バイナリをすべてのユーザーが実行できるようにパーミッションを設定
    sudo chown -R "$USER" /opt/flutter

    # PATH に追加（未追加の場合のみ）
    if ! grep -q '/opt/flutter/bin' "$HOME/.bashrc"; then
        echo 'export PATH=$PATH:/opt/flutter/bin' >> "$HOME/.bashrc"
    fi
    export PATH="$PATH:/opt/flutter/bin"

    # キャッシュ初期化・アナリティクス無効化
    flutter precache --web
    flutter config --no-analytics
    ok "Flutter ${FLUTTER_VERSION} をインストールしました"
fi

# ------------------------------------------------------------------
echo ""
echo "========================================================="
echo "セットアップ完了"
echo "========================================================="
echo ""
echo "インストール済みツール:"
has rustc   && echo "  Rust:      $(rustc --version)"
has go      && echo "  Go:        $(go version)"
has node    && echo "  Node.js:   $(node --version)"
has docker  && echo "  Docker:    $(docker --version 2>/dev/null || echo '(デーモン未起動)')"
has buf     && echo "  buf:       $(buf --version)"
has just    && echo "  just:      $(just --version)"
has sqlx    && echo "  sqlx:      $(sqlx --version)"
has flutter && echo "  Flutter:   $(flutter --version 2>&1 | head -1)"
echo ""
echo "次のステップ:"
echo "  1. WSL を再起動してグループ変更を反映: wsl --shutdown"
echo "  2. VS Code から WSL に接続して開発を開始"
echo "  3. インフラを起動: just local-up"
echo ""
