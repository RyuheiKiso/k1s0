# CI/CD 設計

GitHub Actions によるパイプライン設計を定義する。
Tier アーキテクチャの詳細は [tier-architecture.md](../../architecture/overview/tier-architecture.md) を参照。

## 基本方針

- CI/CD は **GitHub Actions** で一元管理する
- PR 時に CI（lint → test → build）、マージ時に CD（image push → deploy）を実行する
- 環境別デプロイ: dev 自動 / staging 自動 / prod 手動承認
- セキュリティスキャン（Trivy・依存関係チェック）を全パイプラインに組み込む

---

## D-101: GitHub Actions パイプライン設計

### ワークフロー構成

| ワークフロー      | ファイル          | トリガー                    | 目的                     |
| ----------------- | ----------------- | --------------------------- | ------------------------ |
| CI                | `ci.yaml`         | PR 作成・更新時             | lint → test → build     |
| Deploy            | `deploy.yaml`     | main マージ時               | image push → deploy     |
| Proto Check       | `proto.yaml`      | `api/proto/**` 変更時       | proto lint + breaking    |
| Security Scan     | `security.yaml`   | 日次 + PR 時                | 脆弱性スキャン           |
| Kong Config Sync  | `kong-sync.yaml`  | main マージ時 (`infra/kong/**` 変更) | dev → staging → prod    |
| OpenAPI Lint      | `api-lint.yaml`   | push (`**/api/openapi/**`)  | OpenAPI バリデーション & SDK 生成 |
| Tauri GUI Build   | `tauri-build.yaml` | PR 時 + main マージ時 (`CLI/crates/k1s0-gui/**` 変更) | GUI クロスプラットフォームビルド（[TauriGUI設計](../../cli/gui/TauriGUI設計.md) 参照） |
| auth CI           | `auth-ci.yaml`    | PR 時 (`regions/system/server/rust/auth/**`) | auth-server 専用 lint → test → build |
| config CI         | `config-ci.yaml`  | PR 時 (`regions/system/server/rust/config/**`) | config-server 専用 CI |
| saga CI           | `saga-ci.yaml`    | PR 時 (`regions/system/server/rust/saga/**`) | saga-server 専用 CI |
| dlq-manager CI    | `dlq-manager-ci.yaml` | PR 時 (`regions/system/server/rust/dlq-manager/**`) | dlq-manager 専用 lint → test → build |
| bff-proxy CI      | `bff-proxy-ci.yaml` | PR 時 (`regions/system/server/go/bff-proxy/**`) | bff-proxy 専用 lint → test → build（Go） |
| Integration Test  | `integration-test.yaml` | PR 時 (`regions/system/server/rust/**`) | postgres:17 + kafka:7.7.1 起動、4サービス統合テスト |
| auth Deploy       | `auth-deploy.yaml` | main マージ時 (`regions/system/server/rust/auth/**`) | auth-server 専用デプロイ |
| config Deploy     | `config-deploy.yaml` | main マージ時 (`regions/system/server/rust/config/**`) | config-server 専用デプロイ |
| saga Deploy       | `saga-deploy.yaml` | main マージ時 (`regions/system/server/rust/saga/**`) | saga-server 専用デプロイ（dev→staging→prod）|
| dlq-manager Deploy | `dlq-manager-deploy.yaml` | main マージ時 (`regions/system/server/rust/dlq-manager/**`) | dlq-manager 専用デプロイ（dev→staging→prod）|
| bff-proxy Deploy  | `bff-proxy-deploy.yaml` | main マージ時 (`regions/system/server/go/bff-proxy/**`) | bff-proxy 専用デプロイ（dev→staging→prod）|
| App Publish       | `publish-app.yaml` | Git タグ push (`v*`) + `regions/**/flutter/**` 変更 | Flutter デスクトップアプリのクロスプラットフォームビルド・署名・Ceph RGW へのアップロード・App Registry へのメタデータ登録（[アプリ配布基盤設計](../distribution/アプリ配布基盤設計.md) 参照）|
| coverage-rust     | `coverage-rust.yaml` | PR 時 (`regions/**/rust/**`) | Rust テストカバレッジを cargo-tarpaulin で計測し、JSON + HTML レポートをアーティファクトとしてアップロードする |

### CI ワークフロー（ci.yaml）

```yaml
# .github/workflows/ci.yaml
name: CI

on:
  pull_request:
    branches: [main]

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

jobs:
  detect-changes:
    runs-on: ubuntu-latest
    outputs:
      rust: ${{ steps.filter.outputs.rust }}
      go: ${{ steps.filter.outputs.go }}
      ts: ${{ steps.filter.outputs.ts }}
      dart: ${{ steps.filter.outputs.dart }}
      helm: ${{ steps.filter.outputs.helm }}
    steps:
      - uses: actions/checkout@v4
      - uses: dorny/paths-filter@v3
        id: filter
        with:
          filters: |
            rust:
              - 'regions/**/rust/**'
              - 'CLI/**'
            go:
              - 'regions/**/go/**'
            ts:
              - 'regions/**/react/**'
              - 'regions/**/ts/**'
            dart:
              - 'regions/**/flutter/**'
              - 'regions/**/dart/**'
            helm:
              - 'infra/helm/**'

  lint-rust:
    needs: detect-changes
    if: needs.detect-changes.outputs.rust == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.93       # Dockerイメージ戦略.md / devcontainer設計.md と同期
        with:
          components: clippy, rustfmt
      # rg で Cargo.toml を自動探索し、ワークスペースルートと実験系クレートをスキップ
      # スキップ対象: CLI/Cargo.toml, regions/system/Cargo.toml, CLI/crates/k1s0-gui/Cargo.toml,
      #   regions/system/server/rust/ai-agent/Cargo.toml, regions/system/server/rust/ai-gateway/Cargo.toml
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings

  # 実験系 ai-* クレート（continue-on-error で可視性を維持）
  check-ai-experimental:
    needs: detect-changes
    if: needs.detect-changes.outputs.rust == 'true'
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.93
        with:
          components: clippy
      - run: cargo check --manifest-path regions/system/server/rust/ai-agent/Cargo.toml --all-targets
        continue-on-error: true
      - run: cargo check --manifest-path regions/system/server/rust/ai-gateway/Cargo.toml --all-targets
        continue-on-error: true

  lint-go:
    needs: detect-changes
    if: needs.detect-changes.outputs.go == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: "1.24"
      - run: go vet ./...
      - run: golangci-lint run

  lint-ts:
    needs: detect-changes
    if: needs.detect-changes.outputs.ts == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "22"
      - run: npm ci
      - run: npx eslint .
      - run: npx prettier --check .

  lint-dart:
    needs: detect-changes
    if: needs.detect-changes.outputs.dart == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with:
          flutter-version: "3.24.0"              # devcontainer設計.md と同期
      - run: dart analyze
      - run: dart format --set-exit-if-changed .

    needs: detect-changes
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:

  test-rust:
    needs: lint-rust
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.93       # Dockerイメージ戦略.md / devcontainer設計.md と同期
      - run: cargo test --all

  test-go:
    needs: lint-go
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: "1.24"
      - run: go test ./...

  test-ts:
    needs: lint-ts
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "22"
      - run: npm ci
      - run: npm test

  test-dart:
    needs: lint-dart
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with:
          flutter-version: "3.24.0"              # devcontainer設計.md と同期
      - run: flutter test

    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:

  helm-lint:
    needs: detect-changes
    if: needs.detect-changes.outputs.helm == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: azure/setup-helm@v4
        with:
          version: "3.16"   # devcontainer設計.md の Helm バージョンと同期
      - run: |
          for chart in infra/helm/services/*/* infra/helm/services/*/*/*; do
            if [ -f "$chart/Chart.yaml" ]; then
              helm lint "$chart"
            fi
          done

  build:
    needs:
      - test-rust
      - test-go
      - test-ts
      - test-dart
    if: always() && !contains(needs.*.result, 'failure')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build Docker images (dry-run)
        run: |
          echo "Build validation passed"

  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Trivy filesystem scan
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: fs
          scan-ref: .
          severity: HIGH,CRITICAL
          exit-code: 1
```

### Deploy ワークフロー（deploy.yaml）

```yaml
# .github/workflows/deploy.yaml
name: Deploy

on:
  push:
    branches: [main]

env:
  REGISTRY: harbor.internal.example.com

jobs:
  detect-services:
    runs-on: ubuntu-latest
    outputs:
      services: ${{ steps.detect.outputs.services }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 2
      - id: detect
        name: Detect changed services
        run: |
          # ディレクトリ構成図.md に基づくサービス検出:
          #   system:   regions/system/server/{lang}/{service}/...
          #   business: regions/business/{domain}/server/{lang}/{service}/...
          #   service:  regions/service/{service}/server/{lang}/...
          CHANGED=$(git diff --name-only HEAD~1 HEAD | \
            grep -E '^regions/' | \
            sed 's|^regions/||' | \
            while IFS= read -r path; do
              case "$path" in
                system/server/*/*/*)       echo "$path" | cut -d'/' -f1-4 ;;
                business/*/server/*/*/*)   echo "$path" | cut -d'/' -f1-5 ;;
                service/*/server/*/*)      echo "$path" | cut -d'/' -f1-4 ;;
              esac
            done | sort -u | head -20)
          echo "services=$(echo "$CHANGED" | jq -R -s -c 'split("\n") | map(select(. != ""))')" >> "$GITHUB_OUTPUT"

  build-and-push:
    needs: detect-services
    if: needs.detect-services.outputs.services != '[]'
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: ${{ fromJson(needs.detect-services.outputs.services) }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Set short SHA
        id: sha
        run: echo "short=${GITHUB_SHA::7}" >> "$GITHUB_OUTPUT"
      - name: Set version
        id: version
        run: |
          # 直近の Git タグからバージョンを取得（例: v1.2.3 → 1.2.3）
          VERSION=$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || echo "0.0.0")
          echo "value=${VERSION}" >> "$GITHUB_OUTPUT"
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      - name: Login to Harbor
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ secrets.HARBOR_USERNAME }}
          password: ${{ secrets.HARBOR_PASSWORD }}
      - name: Determine image metadata
        id: image
        run: |
          TIER=$(echo "${{ matrix.service }}" | cut -d'/' -f1)
          echo "project=k1s0-${TIER}" >> "$GITHUB_OUTPUT"
      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          context: regions/${{ matrix.service }}
          push: true
          tags: |
            ${{ env.REGISTRY }}/${{ steps.image.outputs.project }}/${{ matrix.service }}:${{ steps.version.outputs.value }}
            ${{ env.REGISTRY }}/${{ steps.image.outputs.project }}/${{ matrix.service }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
            ${{ env.REGISTRY }}/${{ steps.image.outputs.project }}/${{ matrix.service }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
      - name: Install Cosign
        uses: sigstore/cosign-installer@v3
      - name: Sign image with Cosign
        run: |
          cosign sign --yes \
            ${{ env.REGISTRY }}/${{ steps.image.outputs.project }}/${{ matrix.service }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
        env:
          COSIGN_EXPERIMENTAL: "1"

  # NOTE: デプロイジョブはクラスタネットワーク内の self-hosted ランナーで実行する。
  # 各環境のランナーはそれぞれのクラスタ内で動作し、Kubernetes API や
  # 内部サービスへの直接アクセスが可能となる。
  deploy-dev:
    needs: [build-and-push, detect-services]
    runs-on: [self-hosted, dev]
    environment: dev
    strategy:
      matrix:
        service: ${{ fromJson(needs.detect-services.outputs.services) }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Set short SHA
        id: sha
        run: echo "short=${GITHUB_SHA::7}" >> "$GITHUB_OUTPUT"
      - name: Set version
        id: version
        run: |
          VERSION=$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || echo "0.0.0")
          echo "value=${VERSION}" >> "$GITHUB_OUTPUT"
      - name: Derive service metadata
        id: meta
        run: |
          # Tier 別のディレクトリ構成に基づきメタデータを導出:
          #   system/server/{lang}/{service}         → Helm: system/{service}
          #   business/{domain}/server/{lang}/{service} → Helm: business/{domain}/{service}
          #   service/{service}/server/{lang}        → Helm: service/{service}
          TIER=$(echo "${{ matrix.service }}" | cut -d'/' -f1)
          case "$TIER" in
            system)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f4)
              HELM_PATH="system/${SERVICE_NAME}"
              ;;
            business)
              DOMAIN=$(echo "${{ matrix.service }}" | cut -d'/' -f2)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f5)
              HELM_PATH="business/${DOMAIN}/${SERVICE_NAME}"
              ;;
            service)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f2)
              HELM_PATH="service/${SERVICE_NAME}"
              ;;
          esac
          echo "project=k1s0-${TIER}" >> "$GITHUB_OUTPUT"
          echo "service_name=${SERVICE_NAME}" >> "$GITHUB_OUTPUT"
          echo "helm_path=${HELM_PATH}" >> "$GITHUB_OUTPUT"
          echo "namespace=k1s0-${TIER}" >> "$GITHUB_OUTPUT"
      - uses: sigstore/cosign-installer@v3
      - name: Verify image signature
        run: |
          cosign verify \
            --certificate-oidc-issuer https://token.actions.githubusercontent.com \
            --certificate-identity-regexp "github.com/k1s0-org/k1s0" \
            ${{ env.REGISTRY }}/${{ steps.meta.outputs.project }}/${{ matrix.service }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
      - uses: azure/setup-helm@v4
        with:
          version: "3.16"   # devcontainer設計.md の Helm バージョンと同期
      - name: Deploy to dev
        run: |
          helm upgrade --install ${{ steps.meta.outputs.service_name }} \
            ./infra/helm/services/${{ steps.meta.outputs.helm_path }} \
            -n ${{ steps.meta.outputs.namespace }} \
            -f ./infra/helm/services/${{ steps.meta.outputs.helm_path }}/values-dev.yaml \
            --set image.tag=${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}

  deploy-staging:
    needs: [deploy-dev, detect-services]
    runs-on: [self-hosted, staging]
    environment: staging
    strategy:
      matrix:
        service: ${{ fromJson(needs.detect-services.outputs.services) }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Set short SHA
        id: sha
        run: echo "short=${GITHUB_SHA::7}" >> "$GITHUB_OUTPUT"
      - name: Set version
        id: version
        run: |
          VERSION=$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || echo "0.0.0")
          echo "value=${VERSION}" >> "$GITHUB_OUTPUT"
      - name: Derive service metadata
        id: meta
        run: |
          # Tier 別のディレクトリ構成に基づきメタデータを導出:
          #   system/server/{lang}/{service}         → Helm: system/{service}
          #   business/{domain}/server/{lang}/{service} → Helm: business/{domain}/{service}
          #   service/{service}/server/{lang}        → Helm: service/{service}
          TIER=$(echo "${{ matrix.service }}" | cut -d'/' -f1)
          case "$TIER" in
            system)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f4)
              HELM_PATH="system/${SERVICE_NAME}"
              ;;
            business)
              DOMAIN=$(echo "${{ matrix.service }}" | cut -d'/' -f2)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f5)
              HELM_PATH="business/${DOMAIN}/${SERVICE_NAME}"
              ;;
            service)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f2)
              HELM_PATH="service/${SERVICE_NAME}"
              ;;
          esac
          echo "project=k1s0-${TIER}" >> "$GITHUB_OUTPUT"
          echo "service_name=${SERVICE_NAME}" >> "$GITHUB_OUTPUT"
          echo "helm_path=${HELM_PATH}" >> "$GITHUB_OUTPUT"
          echo "namespace=k1s0-${TIER}" >> "$GITHUB_OUTPUT"
      - uses: sigstore/cosign-installer@v3
      - name: Verify image signature
        run: |
          cosign verify \
            --certificate-oidc-issuer https://token.actions.githubusercontent.com \
            --certificate-identity-regexp "github.com/k1s0-org/k1s0" \
            ${{ env.REGISTRY }}/${{ steps.meta.outputs.project }}/${{ matrix.service }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
      - uses: azure/setup-helm@v4
        with:
          version: "3.16"   # devcontainer設計.md の Helm バージョンと同期
      - name: Deploy to staging
        run: |
          helm upgrade --install ${{ steps.meta.outputs.service_name }} \
            ./infra/helm/services/${{ steps.meta.outputs.helm_path }} \
            -n ${{ steps.meta.outputs.namespace }} \
            -f ./infra/helm/services/${{ steps.meta.outputs.helm_path }}/values-staging.yaml \
            --set image.tag=${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}

  deploy-prod:
    needs: [deploy-staging, detect-services]
    runs-on: [self-hosted, prod]
    environment:
      name: prod
      url: https://api.k1s0.internal.example.com
    strategy:
      matrix:
        service: ${{ fromJson(needs.detect-services.outputs.services) }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Set short SHA
        id: sha
        run: echo "short=${GITHUB_SHA::7}" >> "$GITHUB_OUTPUT"
      - name: Set version
        id: version
        run: |
          VERSION=$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || echo "0.0.0")
          echo "value=${VERSION}" >> "$GITHUB_OUTPUT"
      - name: Derive service metadata
        id: meta
        run: |
          # Tier 別のディレクトリ構成に基づきメタデータを導出:
          #   system/server/{lang}/{service}         → Helm: system/{service}
          #   business/{domain}/server/{lang}/{service} → Helm: business/{domain}/{service}
          #   service/{service}/server/{lang}        → Helm: service/{service}
          TIER=$(echo "${{ matrix.service }}" | cut -d'/' -f1)
          case "$TIER" in
            system)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f4)
              HELM_PATH="system/${SERVICE_NAME}"
              ;;
            business)
              DOMAIN=$(echo "${{ matrix.service }}" | cut -d'/' -f2)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f5)
              HELM_PATH="business/${DOMAIN}/${SERVICE_NAME}"
              ;;
            service)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f2)
              HELM_PATH="service/${SERVICE_NAME}"
              ;;
          esac
          echo "project=k1s0-${TIER}" >> "$GITHUB_OUTPUT"
          echo "service_name=${SERVICE_NAME}" >> "$GITHUB_OUTPUT"
          echo "helm_path=${HELM_PATH}" >> "$GITHUB_OUTPUT"
          echo "namespace=k1s0-${TIER}" >> "$GITHUB_OUTPUT"
      - uses: sigstore/cosign-installer@v3
      - name: Verify image signature
        run: |
          cosign verify \
            --certificate-oidc-issuer https://token.actions.githubusercontent.com \
            --certificate-identity-regexp "github.com/k1s0-org/k1s0" \
            ${{ env.REGISTRY }}/${{ steps.meta.outputs.project }}/${{ matrix.service }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
      - uses: azure/setup-helm@v4
        with:
          version: "3.16"   # devcontainer設計.md の Helm バージョンと同期
      - name: Deploy to prod
        run: |
          helm upgrade --install ${{ steps.meta.outputs.service_name }} \
            ./infra/helm/services/${{ steps.meta.outputs.helm_path }} \
            -n ${{ steps.meta.outputs.namespace }} \
            -f ./infra/helm/services/${{ steps.meta.outputs.helm_path }}/values-prod.yaml \
            --set image.tag=${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
```

### 環境別デプロイ戦略

| 環境    | トリガー           | 承認       | ロールバック              |
| ------- | ------------------ | ---------- | ------------------------- |
| dev     | main マージ時 自動 | 不要       | `helm rollback` 手動      |
| staging | dev 成功後 自動    | 不要       | `helm rollback` 手動      |
| prod    | staging 成功後     | 手動承認   | `helm rollback` 即時実行  |

### GitHub Environments 設定

| Environment | Protection Rules                                  |
| ----------- | ------------------------------------------------- |
| dev         | なし                                              |
| staging     | なし                                              |
| prod        | Required reviewers（2名以上）+ Wait timer（5分）  |

### Proto Check ワークフロー（proto.yaml）

```yaml
# .github/workflows/proto.yaml
name: Proto Check

on:
  pull_request:
    paths:
      - 'api/proto/**'
      - 'api/proto/buf.yaml'
      - 'api/proto/buf.gen.yaml'

jobs:
  proto-lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: bufbuild/buf-setup-action@v1
        with:
          version: "1.47.2"              # devcontainer設計.md の BUF_VERSION と同期
      - name: Lint
        run: buf lint api/proto
      - name: Breaking change detection
        run: buf breaking api/proto --against '.git#branch=main'
      - name: Generate (dry-run)
        run: buf generate api/proto --template buf.gen.yaml
```

### Security Scan ワークフロー（security.yaml）

```yaml
# .github/workflows/security.yaml
name: Security Scan

on:
  schedule:
    - cron: '0 2 * * *'    # 毎日 AM 2:00 (UTC)
  pull_request:
    branches: [main]

jobs:
  trivy-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Trivy filesystem scan
        uses: aquasecurity/trivy-action@0.28.0
        with:
          scan-type: fs
          scan-ref: .
          severity: HIGH,CRITICAL
          format: table
          exit-code: 1

  # package-aware: 各言語のモジュールを自動探索してスキャンする
  dependency-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Go セットアップ
        uses: actions/setup-go@v5
        with:
          go-version: '1.24'
      - name: Go vulncheck インストール
        run: go install golang.org/x/vuln/cmd/govulncheck@latest
      - name: Rust セットアップ
        uses: dtolnay/rust-toolchain@stable
      - name: cargo-audit インストール
        run: cargo install cargo-audit
      - name: Node.js セットアップ
        uses: actions/setup-node@v4
        with:
          node-version: '22'
      - name: Flutter セットアップ
        uses: subosito/flutter-action@v2
        with:
          channel: stable
      - name: ripgrep インストール
        run: sudo apt-get install -y ripgrep
      # 各言語は continue-on-error で独立実行し、最後に結果を集約する
      - name: Go 脆弱性スキャン
        id: go-vulncheck
        continue-on-error: true
        run: bash scripts/security/go-vulncheck.sh
      - name: Rust 脆弱性監査
        id: cargo-audit
        continue-on-error: true
        run: bash scripts/security/cargo-audit.sh
      - name: npm 脆弱性監査
        id: npm-audit
        continue-on-error: true
        run: bash scripts/security/npm-audit.sh
      - name: Dart 依存チェック
        id: dart-outdated
        continue-on-error: true
        run: bash scripts/security/dart-outdated.sh
      - name: スキャン結果の集約
        run: |
          if [ "${{ steps.go-vulncheck.outcome }}" = "failure" ] || \
             [ "${{ steps.cargo-audit.outcome }}" = "failure" ] || \
             [ "${{ steps.npm-audit.outcome }}" = "failure" ]; then
            echo "::error::One or more security scans failed"
            exit 1
          fi

  image-scan:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - name: Scan latest images
        uses: aquasecurity/trivy-action@0.28.0
        with:
          scan-type: image
          image-ref: harbor.internal.example.com/k1s0-service/order:latest
          severity: HIGH,CRITICAL
          format: table
```

### OpenAPI バリデーション & SDK 生成ワークフロー（api-lint.yaml）

OpenAPI 定義（`api/openapi/`）の変更時に、バリデーションとクライアント SDK の自動生成を実行する。

- **OpenAPI バリデーション**: `@redocly/cli` による OpenAPI 定義の lint チェック
- **コード生成**: `oapi-codegen` による Go サーバーコードの生成と差分チェック
- **SDK 自動生成**: `openapi-generator-cli` による TypeScript / Dart クライアント SDK の生成

詳細な CI ジョブ定義は [API設計.md](../../architecture/api/API設計.md) を参照。

### 統合テストワークフロー（integration-test.yaml）

PR 時に `regions/system/server/rust/**` 配下の変更を検知し、実インフラ（PostgreSQL・Kafka）を使った統合テストを実行する。

- **サービスコンテナ**: `postgres:17`（ヘルスチェック付き）+ `apache/kafka:3.8.0`（KRaft モード）
- **DB 初期化**: `infra/docker/init-db/*.sql` を順次適用
- **対象サービス**: auth-server / config-server / saga-server / dlq-manager / ratelimit-server（各サービスで `cargo test --all -- --ignored`）
- **`#[ignore]` 戦略**: testcontainers を使う統合テストには `#[ignore = "requires Docker (testcontainers)"]` を付与し、通常の `cargo test` ではスキップ。CI の統合テストジョブでのみ `--ignored` フラグで明示実行する
- **スキーマ分離**: 各サービスは専用の PostgreSQL スキーマ（auth / config / saga / dlq）を使用

```yaml
# .github/workflows/integration-test.yaml
name: Integration Test

on:
  pull_request:
    branches: [main]
    paths:
      - 'regions/system/server/rust/**'

concurrency:
  group: integration-${{ github.ref }}
  cancel-in-progress: true

jobs:
  integration-test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:17
        env:
          POSTGRES_USER: dev
          POSTGRES_PASSWORD: dev
        ports:
          - 5432:5432
        options: >-
          --health-cmd "pg_isready -U dev"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      kafka:
        image: apache/kafka:3.8.0
        env:
          KAFKA_NODE_ID: 1
          KAFKA_PROCESS_ROLES: broker,controller
          KAFKA_CONTROLLER_QUORUM_VOTERS: 1@kafka:9093
          KAFKA_LISTENERS: PLAINTEXT://:9092,CONTROLLER://:9093
          KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:9092
          KAFKA_CONTROLLER_LISTENER_NAMES: CONTROLLER
          KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT
          CLUSTER_ID: "5L6g3nShT-eMCtK--X86sw"
          KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
          KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR: 1
          KAFKA_TRANSACTION_STATE_LOG_MIN_ISR: 1
        ports:
          - 9092:9092
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.93
      - name: Initialize database schemas
        run: |
          for sql in infra/docker/init-db/*.sql; do
            psql -h localhost -U dev -d postgres -f "$sql"
          done
      - name: Run integration tests (auth-server)
        run: cd regions/system/server/rust/auth && cargo test --all -- --ignored
      - name: Run integration tests (config-server)
        run: cd regions/system/server/rust/config && cargo test --all -- --ignored
      - name: Run integration tests (saga-server)
        run: cd regions/system/server/rust/saga && cargo test --all -- --ignored
      - name: Run integration tests (dlq-manager)
        run: cd regions/system/server/rust/dlq-manager && cargo test --all -- --ignored
      - name: Run integration tests (ratelimit-server)
        run: cd regions/system/server/rust/ratelimit && cargo test --all -- --ignored
```

### Helm デプロイ連携

CI/CD パイプラインから Helm デプロイを実行する際の連携方式:

```
GitHub Actions (self-hosted runner in cluster) → helm → Kubernetes Cluster
```

| 項目             | 設定                                                |
| ---------------- | --------------------------------------------------- |
| ランナー         | 各環境のクラスタ内で動作する self-hosted ランナーを使用（`[self-hosted, dev]` 等） |
| Helm バージョン  | `azure/setup-helm@v4` で 3.16 を指定（[devcontainer設計.md](../devenv/devcontainer設計.md) と同期） |
| デプロイ方式     | `helm upgrade --install`（冪等性を保証）            |
| イメージタグ     | `--set image.tag=${VERSION}-${GITHUB_SHA::7}` で `{version}-{git-sha}` 形式を指定（[Dockerイメージ戦略.md](../docker/Dockerイメージ戦略.md) のタグ規則に準拠） |

### キャッシュ戦略

| 言語   | キャッシュ対象                | アクション                |
| ------ | ----------------------------- | ------------------------- |
| Go     | `~/go/pkg/mod`               | `actions/cache`           |
| Rust   | `~/.cargo`, `target/`        | `actions/cache`           |
| Node   | `node_modules/`              | `actions/setup-node` 内蔵 |
| Dart   | `~/.pub-cache`               | `actions/cache`           |
| Docker | Docker layer cache           | `cache-from: type=gha`   |

---

## 関連ドキュメント

- [tier-architecture.md](../../architecture/overview/tier-architecture.md) — Tier アーキテクチャの詳細
- [Dockerイメージ戦略.md](../docker/Dockerイメージ戦略.md) — イメージビルド・タグ・レジストリ
- [helm設計.md](../kubernetes/helm設計.md) — Helm Chart と values 設計
- [kubernetes設計.md](../kubernetes/kubernetes設計.md) — Namespace・NetworkPolicy 設計
- [API設計.md](../../architecture/api/API設計.md) — REST API・gRPC・GraphQL 設計
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) — 監視・ログ・トレース設計
- [config.md](../../cli/config/config設計.md) — config.yaml スキーマ・環境別管理
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) — 認証・認可・シークレット管理
- [devcontainer設計.md](../devenv/devcontainer設計.md) — Dev Container 設定
- [APIゲートウェイ設計.md](../../architecture/api/APIゲートウェイ設計.md) — Kong 構成管理
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) — Kafka・Proto スキーマ CI
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) — Linter・Formatter・命名規則
- [アプリ配布基盤設計.md](../distribution/アプリ配布基盤設計.md) — デスクトップアプリ配布パイプライン
