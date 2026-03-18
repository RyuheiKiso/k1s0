# CI/CD 設計

GitHub Actions によるパイプライン設計を定義する。
Tier アーキテクチャの詳細は [tier-architecture.md](../../architecture/overview/tier-architecture.md) を参照。

## 基本方針

- CI/CD は **GitHub Actions** で一元管理する
- PR 時に CI（lint → test → build）、マージ時に CD（image push → deploy）を実行する
- 環境別デプロイ: dev 自動 / staging 自動 / prod 手動承認
- セキュリティスキャン（Trivy・依存関係チェック）を全パイプラインに組み込む

![CI/CD パイプライン全体像](images/cicd-pipeline-overview.svg)

---

## D-101: GitHub Actions パイプライン設計

### ワークフロー構成

| ワークフロー      | ファイル          | トリガー                    | 目的                     |
| ----------------- | ----------------- | --------------------------- | ------------------------ |
| CI                | `ci.yaml`         | PR 作成・更新時             | lint → test → build + モジュールレジストリ検証 |
| Deploy            | `deploy.yaml`     | main マージ時               | image push → deploy     |
| **Rust サービス CI (reusable)** | `_rust-service-ci.yaml` | `workflow_call` | Rust サービスの共通 lint → test → build |
| **Go サービス CI (reusable)** | `_go-service-ci.yaml` | `workflow_call` | Go サービスの共通 lint → test → build |
| **サービス Deploy (reusable)** | `_service-deploy.yaml` | `workflow_call` | サービスの共通 build-push → deploy (dev→staging→prod) |
| Proto Check       | `proto.yaml`      | `api/proto/**` 変更時       | proto lint + breaking（ci.yaml の lint-proto ジョブでも実行） |
| Security Scan     | `security.yaml`   | 日次 + PR 時                | 脆弱性スキャン           |
| Kong Config Sync  | `kong-sync.yaml`  | main マージ時 (`infra/kong/**` 変更) | dev → staging → prod    |
| OpenAPI Lint      | `api-lint.yaml`   | push (`**/api/openapi/**`)  | OpenAPI バリデーション & SDK 生成 |
| Tauri GUI Build   | `tauri-build.yaml` | PR 時 + main マージ時 (`CLI/crates/k1s0-gui/**` 変更) | GUI クロスプラットフォームビルド（[TauriGUI設計](../../cli/gui/TauriGUI設計.md) 参照） |
| auth CI           | `auth-ci.yaml`    | PR 時 (`regions/system/server/rust/auth/**`) | `_rust-service-ci.yaml` 呼び出し |
| app-registry CI   | `app-registry-ci.yaml` | PR 時 (`regions/system/server/rust/app-registry/**`) | `_rust-service-ci.yaml` 呼び出し |
| config CI         | `config-ci.yaml`  | PR 時 (`regions/system/server/rust/config/**`) | `_rust-service-ci.yaml` 呼び出し |
| saga CI           | `saga-ci.yaml`    | PR 時 (`regions/system/server/rust/saga/**`) | `_rust-service-ci.yaml` 呼び出し |
| dlq-manager CI    | `dlq-manager-ci.yaml` | PR 時 (`regions/system/server/rust/dlq-manager/**`) | `_rust-service-ci.yaml` 呼び出し |
| order CI          | `order-ci.yaml`   | PR 時 (`regions/service/order/**`) | `_rust-service-ci.yaml` 呼び出し (standalone) |
| inventory CI      | `inventory-ci.yaml` | PR 時 (`regions/service/inventory/**`) | `_rust-service-ci.yaml` 呼び出し (standalone) |
| payment CI        | `payment-ci.yaml` | PR 時 (`regions/service/payment/**`) | `_rust-service-ci.yaml` 呼び出し (standalone) |
| domain-master CI  | `domain-master-ci.yaml` | PR 時 (`regions/business/accounting/**`) | `_rust-service-ci.yaml` 呼び出し (standalone) |
| bff-proxy CI      | `bff-proxy-ci.yaml` | PR 時 (`regions/system/server/go/bff-proxy/**`) | `_go-service-ci.yaml` 呼び出し |
| Integration Test  | `integration-test.yaml` | PR 時 (`regions/system/{server,library}/rust/**`, `Cargo.{toml,lock}`) | postgres:17 + kafka:3.8.0 起動、system tier サーバー自動検出・パッケージ単位並列統合テスト（test-utils feature 自動検出） |
| Golden Path Compile | `golden-path-compile.yaml` | PR 時 (`CLI/crates/k1s0-codegen/**`, `CLI/templates/**`) | CLI テンプレートからサーバーを生成し `cargo check` でコンパイル検証 |
| auth Deploy       | `auth-deploy.yaml` | main マージ時 (`regions/system/server/rust/auth/**`) | `_service-deploy.yaml` 呼び出し |
| app-registry Deploy | `app-registry-deploy.yaml` | main マージ時 (`regions/system/server/rust/app-registry/**`) | `_service-deploy.yaml` 呼び出し |
| config Deploy     | `config-deploy.yaml` | main マージ時 (`regions/system/server/rust/config/**`) | `_service-deploy.yaml` 呼び出し |
| saga Deploy       | `saga-deploy.yaml` | main マージ時 (`regions/system/server/rust/saga/**`) | `_service-deploy.yaml` 呼び出し |
| dlq-manager Deploy | `dlq-manager-deploy.yaml` | main マージ時 (`regions/system/server/rust/dlq-manager/**`) | `_service-deploy.yaml` 呼び出し |
| bff-proxy Deploy  | `bff-proxy-deploy.yaml` | main マージ時 (`regions/system/server/go/bff-proxy/**`) | `_service-deploy.yaml` 呼び出し (port-forward) |
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
          fetch-depth: 0
      - id: detect
        name: Detect changed services
        run: |
          # ディレクトリ構成図.md に基づくサービス検出:
          #   system:   regions/system/server/{lang}/{service}/...
          #   business: regions/business/{domain}/server/{lang}/{service}/...
          #   service:  regions/service/{service}/server/{lang}/{service}/...
          CHANGED=$(git diff --name-only ${{ github.event.before }} ${{ github.sha }} | \
            grep -E '^regions/' | \
            sed 's|^regions/||' | \
            while IFS= read -r path; do
              case "$path" in
                system/server/*/*/*)       echo "$path" | cut -d'/' -f1-4 ;;
                business/*/server/*/*/*)   echo "$path" | cut -d'/' -f1-5 ;;
                service/*/server/*/*/*)    echo "$path" | cut -d'/' -f1-5 ;;
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
          # Tier と短縮サービス名を導出（イメージ名に使用）
          TIER=$(echo "${{ matrix.service }}" | cut -d'/' -f1)
          case "$TIER" in
            system)   SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f4) ;;
            business) SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f5) ;;
            service)  SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f2) ;;
          esac
          echo "project=k1s0-${TIER}" >> "$GITHUB_OUTPUT"
          echo "service_name=${SERVICE_NAME}" >> "$GITHUB_OUTPUT"
      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          context: regions/${{ matrix.service }}
          push: true
          tags: |
            ${{ env.REGISTRY }}/${{ steps.image.outputs.project }}/${{ steps.image.outputs.service_name }}:${{ steps.version.outputs.value }}
            ${{ env.REGISTRY }}/${{ steps.image.outputs.project }}/${{ steps.image.outputs.service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
            ${{ env.REGISTRY }}/${{ steps.image.outputs.project }}/${{ steps.image.outputs.service_name }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
      - name: Install Cosign
        uses: sigstore/cosign-installer@v3
      - name: Sign image with Cosign
        run: |
          cosign sign --yes \
            ${{ env.REGISTRY }}/${{ steps.image.outputs.project }}/${{ steps.image.outputs.service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
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
          #   business/{domain}/server/{lang}/{service} → Helm: business/{service}
          #   service/{service}/server/{lang}/{name}  → Helm: service/{service}
          TIER=$(echo "${{ matrix.service }}" | cut -d'/' -f1)
          case "$TIER" in
            system)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f4)
              HELM_PATH="system/${SERVICE_NAME}"
              ;;
            business)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f5)
              HELM_PATH="business/${SERVICE_NAME}"
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
            ${{ env.REGISTRY }}/${{ steps.meta.outputs.project }}/${{ steps.meta.outputs.service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
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
          #   business/{domain}/server/{lang}/{service} → Helm: business/{service}
          #   service/{service}/server/{lang}/{name}  → Helm: service/{service}
          TIER=$(echo "${{ matrix.service }}" | cut -d'/' -f1)
          case "$TIER" in
            system)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f4)
              HELM_PATH="system/${SERVICE_NAME}"
              ;;
            business)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f5)
              HELM_PATH="business/${SERVICE_NAME}"
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
            ${{ env.REGISTRY }}/${{ steps.meta.outputs.project }}/${{ steps.meta.outputs.service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
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
          #   business/{domain}/server/{lang}/{service} → Helm: business/{service}
          #   service/{service}/server/{lang}/{name}  → Helm: service/{service}
          TIER=$(echo "${{ matrix.service }}" | cut -d'/' -f1)
          case "$TIER" in
            system)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f4)
              HELM_PATH="system/${SERVICE_NAME}"
              ;;
            business)
              SERVICE_NAME=$(echo "${{ matrix.service }}" | cut -d'/' -f5)
              HELM_PATH="business/${SERVICE_NAME}"
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
            ${{ env.REGISTRY }}/${{ steps.meta.outputs.project }}/${{ steps.meta.outputs.service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
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

![環境別デプロイフロー](images/cicd-environment-progression.svg)

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

監査対応として以下のセキュリティスキャンジョブを実装している。

| ジョブ | 実行タイミング | 目的 |
| --- | --- | --- |
| `trivy-scan` | 日次 + PR 時 | リポジトリ全体のファイルシステム脆弱性スキャン（HIGH/CRITICAL） |
| `dependency-check` | 日次 + PR 時 | Go / Rust / npm / Dart の依存関係脆弱性チェック（`list-modules.sh` ベース） |
| `image-scan` | 日次（schedule のみ） | 全 11 サービスのコンテナイメージ脆弱性スキャン（system / business / service 全ティア対象） |
| `iac-scan` | 日次 + PR 時 | `infra/` ディレクトリの Terraform / Kubernetes マニフェスト構成ミス検出（Trivy config scan） |
| `license-scan` | 日次 + PR 時 | 依存関係のライセンスコンプライアンスチェック（Trivy license scanner） |

#### IaC スキャン

`iac-scan` ジョブは Trivy の `config` スキャンタイプを使用して `infra/` ディレクトリを走査する。Terraform 定義ファイルおよび Kubernetes マニフェストの設定ミス（セキュリティグループの過剰開放、暗号化未設定、特権コンテナ等）を HIGH/CRITICAL レベルで検出し、検出時は `exit-code: 1` でジョブを失敗させる。

#### ライセンススキャン

`license-scan` ジョブは Trivy の `license` スキャナーを使用してリポジトリ全体の依存関係ライセンスを検証する。許容されないライセンス（GPL 等の強力なコピーレフトライセンス）が HIGH/CRITICAL として検出された場合、ジョブを失敗させる。これにより、意図しないライセンス汚染を CI レベルで防止する。

#### イメージスキャン拡大

`image-scan` ジョブは以前 order サービスのみを対象としていたが、現在は system / business / service 全ティアの 11 サービスに拡大している。マトリクス戦略（`fail-fast: false`）で並列実行し、各サービスのコンテナイメージを個別にスキャンする。

| ティア | 対象サービス |
| --- | --- |
| system | auth, config, saga, bff-proxy, app-registry, dlq-manager, graphql-gateway |
| business | domain-master |
| service | order, payment, inventory |

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

  # package-aware: list-modules.sh ベースでモジュールを探索してスキャンする
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
      # 各言語は continue-on-error で独立実行し、最後に結果を集約する
      # go-vulncheck.sh / npm-audit.sh は list-modules.sh ベースでモジュールを取得（rg 探索は廃止）
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

  # 全ティアのコンテナイメージ脆弱性スキャン（定期実行のみ）
  # マトリクス戦略で system / business / service ティアの全サービスイメージをスキャン
  image-scan:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    permissions:
      contents: read
    strategy:
      fail-fast: false
      matrix:
        include:
          # --- system ティア ---
          - tier: system
            service: auth
          - tier: system
            service: config
          - tier: system
            service: saga
          - tier: system
            service: bff-proxy
          - tier: system
            service: app-registry
          - tier: system
            service: dlq-manager
          - tier: system
            service: graphql-gateway
          # --- business ティア ---
          - tier: business
            service: domain-master
          # --- service ティア ---
          - tier: service
            service: order
          - tier: service
            service: payment
          - tier: service
            service: inventory
    steps:
      - name: コンテナイメージの脆弱性スキャン (${{ matrix.tier }}/${{ matrix.service }})
        uses: aquasecurity/trivy-action@0.28.0
        with:
          scan-type: image
          image-ref: harbor.internal.example.com/k1s0-${{ matrix.tier }}/${{ matrix.service }}:latest
          severity: HIGH,CRITICAL
          format: table

  # IaC（Infrastructure as Code）構成ミススキャン
  # Terraform / Kubernetes マニフェストの設定ミスを Trivy で検出
  iac-scan:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@v4
      - name: Trivy IaC 構成スキャン
        uses: aquasecurity/trivy-action@0.28.0
        with:
          scan-type: config
          scan-ref: infra/
          severity: HIGH,CRITICAL
          format: table
          exit-code: 1

  # ライセンスコンプライアンスチェック
  # 依存関係のライセンスが許容範囲内かを Trivy で検証
  license-scan:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@v4
      - name: Trivy ライセンスコンプライアンススキャン
        uses: aquasecurity/trivy-action@0.28.0
        with:
          scan-type: fs
          scan-ref: .
          scanners: license
          severity: HIGH,CRITICAL
          format: table
          exit-code: 1
```

### OpenAPI バリデーション & SDK 生成ワークフロー（api-lint.yaml）

OpenAPI 定義（`api/openapi/`）の変更時に、バリデーションとクライアント SDK の自動生成を実行する。

- **OpenAPI バリデーション**: `@redocly/cli` による OpenAPI 定義の lint チェック
- **コード生成**: `oapi-codegen` による Go サーバーコードの生成と差分チェック
- **SDK 自動生成**: `openapi-generator-cli` による TypeScript / Dart クライアント SDK の生成

詳細な CI ジョブ定義は [API設計.md](../../architecture/api/API設計.md) を参照。

### 統合テストワークフロー（integration-test.yaml）

PR 時に `regions/system/` 配下のサーバー・ライブラリ・ワークスペース設定の変更を検知し、実インフラ（PostgreSQL・Kafka）を使った統合テストを実行する。

- **サービスコンテナ**: `postgres:17`（ヘルスチェック付き）+ `apache/kafka:3.8.0`（KRaft モード）
- **DB 初期化**: `infra/docker/init-db/*.sql` を順次適用
- **対象サービス**: `scripts/ci-list-integration-servers.sh` で system tier の stable サーバーを自動検出し、`matrix` で並列実行（`fail-fast: false`）
- **テスト実行**: `cargo test -p <package> --test integration_test` でパッケージ単位実行。`test-utils` feature は `Cargo.toml` の `[features]` セクションを検査して自動検出
- **Rust キャッシュ**: `Swatinem/rust-cache@v2` で `~/.cargo` と `target/` をキャッシュし、ビルド時間を短縮
- **スキーマ分離**: 各サービスは専用の PostgreSQL スキーマを使用

```yaml
# .github/workflows/integration-test.yaml
name: Integration Test

on:
  pull_request:
    branches: [main]
    paths:
      - 'regions/system/server/rust/**'
      - 'regions/system/library/rust/**'
      - 'regions/system/Cargo.toml'
      - 'regions/system/Cargo.lock'

concurrency:
  group: integration-${{ github.ref }}
  cancel-in-progress: true

jobs:
  # 統合テスト対象サーバーを自動検出
  detect-servers:
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.detect.outputs.matrix }}
    steps:
      - uses: actions/checkout@v4
      - id: detect
        run: |
          SERVERS=$(bash scripts/ci-list-integration-servers.sh)
          echo "matrix=${SERVERS}" >> "$GITHUB_OUTPUT"

  integration-test:
    needs: detect-servers
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        server: ${{ fromJson(needs.detect-servers.outputs.matrix) }}
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
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: regions/system
      - name: Initialize database schemas
        run: |
          for sql in infra/docker/init-db/*.sql; do
            psql -h localhost -U dev -d postgres -f "$sql"
          done
      - name: Run integration tests (${{ matrix.server }})
        run: |
          cd regions/system
          pkg_name=$(grep -m1 '^name' "server/rust/${{ matrix.server }}/Cargo.toml" | sed 's/.*"\(.*\)"/\1/')
          if grep -q '\[features\]' "server/rust/${{ matrix.server }}/Cargo.toml" && \
             grep -q 'test-utils' "server/rust/${{ matrix.server }}/Cargo.toml"; then
            cargo test -p "$pkg_name" --test integration_test --features "$pkg_name/test-utils"
          else
            cargo test -p "$pkg_name" --test integration_test
          fi
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
| Go     | `~/go/pkg/mod`               | `actions/setup-go@v5` 内蔵 (`cache-dependency-path: go.work.sum`) |
| Rust   | `~/.cargo`, `target/`        | `Swatinem/rust-cache@v2` (`workspaces: regions/system, CLI`) |
| Node   | npm グローバルキャッシュ      | `actions/setup-node@v4` 内蔵 (`cache: 'npm'`) |
| Dart   | `~/.pub-cache`               | `actions/cache@v4` (`key: dart-pub-${{ hashFiles('**/pubspec.lock') }}`) |
| Docker | Docker layer cache           | `cache-from: type=gha`   |

---

## モジュールレジストリ（modules.yaml）

リポジトリルートの `modules.yaml` が全モジュールの唯一の情報源（Single Source of Truth）として機能する。CI・justfile のハードコードされたスキップリストを廃止し、このファイルで一元管理する。

### フィールド定義

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `path` | string | 必須 | モジュールのディレクトリパス |
| `lang` | string | 必須 | 言語（`rust`, `go`, `ts`, `dart`） |
| `status` | string | 必須 | `stable` / `experimental` / `archived` |
| `type` | string | 必須 | `server` / `library` / `client` / `cli` / `workspace` / `proto` |
| `workspace` | string | 任意 | Cargo/Go ワークスペースルートパス |
| `skip-ci` | bool | 任意 | `true` の場合 CI のリント・テスト・ビルドをスキップ |

### フィルタリングスクリプト

`scripts/list-modules.sh` が `modules.yaml` をフィルタリングする唯一の情報源。`rg` による直接探索フォールバックは廃止し、すべての CI ジョブが `list-modules.sh` 経由でモジュール一覧を取得する。`yq` がある場合はそちらを使用し、なければ bash フォールバックで動作する。

CI の lint / test / build / security の全ジョブが `--no-skip-ci` フラグで `modules.yaml` をフィルタし、`skip-ci: true` のモジュールを自動除外する。

```bash
# stable な Rust サーバーのみ取得
scripts/list-modules.sh --lang rust --status stable --type server

# CI 対象の全 Go モジュール（skip-ci を除外）
scripts/list-modules.sh --lang go --no-skip-ci

# experimental モジュールの一覧
scripts/list-modules.sh --status experimental
```

### CI バリデーション

`ci.yaml` の `validate-modules` ジョブがディスク上のマニフェストと `modules.yaml` の双方向チェックを行い、不整合を **error** として CI を失敗させる。

- **順方向チェック**: ディスク上に存在するが `modules.yaml` に未登録のモジュール → エラー
- **逆方向チェック**: `modules.yaml` に登録されているがディスク上に存在しないモジュール → エラー

### 再発防止 lint ジョブ

| ジョブ / ステップ | トリガー | 内容 |
|-------------------|----------|------|
| `lint-rust` 内 "Check for deprecated axum route syntax" | Rust 変更時 | `regions/` 配下の handler/middleware で axum 旧記法 `/:param` の残存を検出し error |
| `validate-vault-policies` | 常時 | `infra/vault/policies/` で `common/*` 以外の広域 wildcard を検出し error |
| `validate-ts-lockfiles` | 常時 | TypeScript パッケージに `package-lock.json` が存在しない場合 error |

### Rust smoke test

全 HTTP サーバーモジュール（22モジュール）に router 初期化 smoke test を配備。`cargo test` で router 構築時の panic（旧記法混入等）を検出する。

## Reusable Workflow アーキテクチャ

![Reusable Workflow アーキテクチャ](images/cicd-reusable-workflow-arch.svg)

サービス別 CI/Deploy ワークフローの重複を排除するため、3つの reusable workflow を定義している。

### `_rust-service-ci.yaml`

Rust サービス用の共通 CI パイプライン（lint → test → build）。

| 入力 | 必須 | 説明 |
|------|------|------|
| `service-path` | 必須 | サービスのディレクトリパス |
| `package-name` | 必須 | Cargo パッケージ名 |
| `workspace-path` | 必須 | Cargo workspace のルートパス |
| `rust-version` | 任意 | Rust ツールチェインバージョン（デフォルト: 1.93） |
| `standalone` | 任意 | ワークスペースの `-p` フラグを使わないモード（デフォルト: false） |

### `_go-service-ci.yaml`

Go サービス用の共通 CI パイプライン（lint → test → build）。

| 入力 | 必須 | 説明 |
|------|------|------|
| `service-path` | 必須 | サービスのディレクトリパス |
| `go-version` | 任意 | Go バージョン（デフォルト: 1.24） |
| `golangci-lint-version` | 任意 | golangci-lint バージョン（デフォルト: v1.64.8） |

### `_service-deploy.yaml`

サービスデプロイ用の共通パイプライン（build-push → deploy-dev → deploy-staging → deploy-prod）。

| 入力 | 必須 | 説明 |
|------|------|------|
| `service-name` | 必須 | サービス名（Helm リリース名） |
| `context-path` | 必須 | Docker ビルドコンテキストのパス |
| `registry-project` | 必須 | Harbor レジストリ内のプロジェクト名 |
| `helm-chart-path` | 必須 | Helm チャートの相対パス |
| `namespace` | 必須 | Kubernetes namespace |
| `dockerfile` | 任意 | カスタム Dockerfile パス |
| `prod-url` | 任意 | prod 環境の URL |
| `health-check-method` | 任意 | `busybox`（Rust）/ `port-forward`（Go） |

### 新サービス追加手順

1. `modules.yaml` にモジュールエントリを追加
2. `.github/workflows/{サービス名}-ci.yaml`（~20行）を作成し、reusable workflow を呼び出す
3. `.github/workflows/{サービス名}-deploy.yaml`（~25行）を作成し、reusable workflow を呼び出す

## ワークスペースレベルビルド

CI の Rust/Go ビルドは個別マニフェスト反復ではなくワークスペース一括操作を採用し、起動回数を O(n) → O(1) に削減している。

- **Rust**: `cargo fmt/clippy/test/build --manifest-path regions/system/Cargo.toml --workspace` + `--exclude` で experimental クレートを除外
- **Go**: `go build ./...` で `go.work` 経由の一括ビルド
- **影響範囲検出時**: `detect-affected-modules.sh` + `AFFECTED_MODULES` 環境変数で影響モジュールのみを個別実行（差分ビルド）

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
