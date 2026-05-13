---
id: env.security.security_oss_install
axis: security
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.security.security_repository_acquisition
covered_by:
  defense_in_depth_layers: [B, E]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- Trivy / Cosign / SPIRE agent / Falco (docker) / OWASP ZAP (docker) の 5 ツールを導入し、全て動作確認できることを主要 OSS 導入の検収条件とする。

## Trivy のインストール（apt 経由）

**前提**: [03_必須ランタイム](03_必須ランタイム.md) の Trivy セクションで Aqua Security apt リポジトリの追加と `apt install trivy` を完了してからこのページを参照すること。apt リポジトリを追加せずに `apt install trivy` を実行しても "package not found" になる。

インストール済み確認:

```bash
trivy --version
# 0.50 以上であること
```

03_必須ランタイムの手順を実施せずにインストールする場合（バイナリ直接）:

```bash
curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin
trivy --version
```

## Cosign のインストール（go install 経由）

Go 環境がある場合は go install でインストールする。

```bash
# Go がある場合
go install github.com/sigstore/cosign/v2/cmd/cosign@latest
# Go がない場合はバイナリダウンロード（03_必須ランタイムの手順参照）
cosign version
```

## SPIRE agent のセットアップ

```bash
# SPIRE server 設定ファイルの作成（ローカルテスト用）
mkdir -p /tmp/spire-config
cat <<'EOF' > /tmp/spire-config/server.conf
server {
    bind_address = "0.0.0.0"
    bind_port = "8081"
    trust_domain = "k1s0.local"
    data_dir = "/tmp/spire-data/server"
    log_level = "DEBUG"
    jwt_issuer = "https://k1s0.local"

    ca_subject {
        country = ["JP"]
        organization = ["k1s0"]
        common_name = "k1s0-local"
    }
}

plugins {
    DataStore "sql" {
        plugin_data {
            database_type = "sqlite3"
            connection_string = "/tmp/spire-data/server.db"
        }
    }
    NodeAttestor "join_token" {
        plugin_data {}
    }
    KeyManager "memory" {
        plugin_data {}
    }
}
EOF
mkdir -p /tmp/spire-data/server
spire-server --version
```

## Falco (Docker 経由)

```bash
# Falco の動作確認（no-driver モード）
docker run --rm falcosecurity/falco-no-driver:latest falco --version
```

## OWASP ZAP (Docker 経由)

```bash
# OWASP ZAP の起動（daemon モード）
docker pull ghcr.io/zaproxy/zaproxy:stable
docker run --rm ghcr.io/zaproxy/zaproxy:stable zap.sh -version
```

## Semgrep のインストール

```bash
source .venv/bin/activate
pip install semgrep
semgrep --version
```

## checkov のインストール

```bash
source .venv/bin/activate
pip install checkov
checkov --version
```

## 検収コマンド

```bash
trivy --version
cosign version
spire-agent --version
docker run --rm falcosecurity/falco-no-driver:latest falco --version 2>/dev/null | head -1
docker run --rm ghcr.io/zaproxy/zaproxy:stable zap.sh -version 2>/dev/null | head -1
source .venv/bin/activate && semgrep --version
source .venv/bin/activate && checkov --version
```

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [07_テスト検証環境](07_テスト検証環境.md)
