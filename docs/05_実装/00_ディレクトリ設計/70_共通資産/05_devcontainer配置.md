# 05. devcontainer 配置

本ファイルは `.devcontainer/` と `tools/devcontainer/profiles/` の配置を確定する。VS Code Dev Container で役割別の再現可能な開発環境を即座に起動できる仕組みを規定する。

## 配置の二重構造

```
.devcontainer/
├── devcontainer.json               # デフォルト（docs-writer 最軽量）
├── post-create.sh
└── README.md
tools/devcontainer/profiles/
├── tier1-rust-dev/
│   ├── devcontainer.json
│   ├── Dockerfile
│   └── post-create.sh
├── tier1-go-dev/
├── tier2-dev/
├── tier3-web-dev/
├── tier3-native-dev/
├── platform-cli-dev/
├── infra-ops/
└── docs-writer/
```

ルート `.devcontainer/devcontainer.json` は最軽量の docs-writer を採用し、初回クローン時に VS Code が即座に起動できることを優先する。開発者は `Reopen in Container` で `tools/devcontainer/profiles/<role>/devcontainer.json` を選択し、役割に対応した環境に切替える。

## 役割別 Dev Container の特徴

### tier1-rust-dev

```json
// tools/devcontainer/profiles/tier1-rust-dev/devcontainer.json
{
  "name": "k1s0 tier1-rust-dev",
  "build": {
    "dockerfile": "Dockerfile"
  },
  "features": {
    "ghcr.io/devcontainers/features/git:1": {},
    "ghcr.io/devcontainers/features/docker-outside-of-docker:1": {}
  },
  "customizations": {
    "vscode": {
      "extensions": [
        "rust-lang.rust-analyzer",
        "vadimcn.vscode-lldb",
        "tamasfe.even-better-toml",
        "bufbuild.vscode-buf",
        "redhat.vscode-yaml"
      ],
      "settings": {
        "rust-analyzer.cargo.features": "all",
        "editor.formatOnSave": true
      }
    }
  },
  "postCreateCommand": "bash .devcontainer/post-create.sh",
  "mounts": [
    "source=${localWorkspaceFolder}/target,target=/workspaces/k1s0/target,type=bind,consistency=cached"
  ],
  "remoteUser": "vscode"
}
```

```dockerfile
# tools/devcontainer/profiles/tier1-rust-dev/Dockerfile
FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

# buf / protoc
RUN curl -sSL https://github.com/bufbuild/buf/releases/latest/download/buf-Linux-x86_64 -o /usr/local/bin/buf \
    && chmod +x /usr/local/bin/buf

# cargo tools
RUN cargo install cargo-nextest cargo-audit cargo-deny cargo-fuzz

# protoc
RUN apt-get update && apt-get install -y --no-install-recommends protobuf-compiler
```

post-create.sh で以下を実行:

- `git sparse-checkout init --cone && git sparse-checkout set --stdin < .sparse-checkout/roles/tier1-rust-dev.txt`
- `cargo fetch`
- `buf dep update`

### tier1-go-dev

Go 1.22 + Dapr SDK + buf + delve debugger。

### tier2-dev

.NET 8 SDK + Roslyn analyzer + Go 1.22 SDK（tier2 は dotnet / go 混在）。

### tier3-web-dev

Node.js 20 + pnpm 9 + Playwright + Vite + React。

### tier3-native-dev

.NET MAUI Workload + Android SDK + Xcode は macOS ホスト委譲（Windows container 環境で Xcode は動かないため）。

### platform-cli-dev

Rust + cargo + handlebars + buf。

### infra-ops

kubectl + helm + kustomize + kind + tofu + ansible + argocd-cli + gh。

### docs-writer

最軽量。git + markdown linter + mermaid-cli + drawio-desktop-cli。

## post-create.sh の共通責務

全 profile が post-create.sh で以下を実行:

1. `tools/sparse/checkout-role.sh <profile-name>` でスパースチェックアウト適用
2. `.sparse-checkout/roles/<role>.txt` に基づきワーキングツリーを絞る
3. 言語固有の依存解決（cargo fetch / go mod download / pnpm install / dotnet restore）
4. buf / OpenAPI 等の code generation（必要時）
5. 開発用 Kubernetes cluster のブートストラップ（infra-ops のみ）

## mount 最適化

`target/`（Rust ビルド成果物）、`node_modules/`、`bin/`（Go build cache）は named volume にバインドして、コンテナ再作成時もキャッシュを保持。

```json
"mounts": [
  "source=k1s0-rust-target,target=/workspaces/k1s0/target,type=volume",
  "source=k1s0-go-cache,target=/home/vscode/go/pkg,type=volume",
  "source=k1s0-pnpm-store,target=/home/vscode/.local/share/pnpm/store,type=volume"
]
```

## CI における Dev Container 利用

GitHub Actions で `devcontainers/ci@v0.3` アクションを使い、本番 CI でも Dev Container を利用する。これにより「ローカルで動くが CI で動かない」現象を排除する。

```yaml
# .github/workflows/ci-tier1-rust.yml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: devcontainers/ci@v0.3
        with:
          configFile: tools/devcontainer/profiles/tier1-rust-dev/devcontainer.json
          runCmd: |
            cargo nextest run
            cargo audit
            cargo deny check
```

## 対応 IMP-DIR ID

- IMP-DIR-COMM-115（devcontainer 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-DEVEX-002（Dev Container 標準化）
- DX-GP-\* / NFR-D-MIG-\*（環境再現性）
