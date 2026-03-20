# CI/CD 設計

GitHub Actions によるパイプライン設計を定義する。
Tier アーキテクチャの詳細は [tier-architecture.md](../../architecture/overview/tier-architecture.md) を参照。

## 基本方針

- CI/CD は **GitHub Actions** で一元管理する
- PR 時に CI（lint → test → build）、マージ時に CD（image push → deploy）を実行する
- 環境別デプロイ: dev 自動 / staging 自動 / prod 手動承認
- セキュリティスキャン（Trivy・依存関係チェック）を全パイプラインに組み込む

### ビルド環境の制限事項

| サーバー | 制限 | 対応 |
|---------|------|------|
| master-maintenance | zen-engine → rquickjs-sys が Windows 未対応 | CI は ubuntu-latest、ローカルは WSL2/devcontainer |

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
| デプロイ方式     | `helm upgrade --install --atomic --wait --timeout 5m`（冪等性 + 失敗時自動ロールバック） |
| イメージタグ     | `--set image.tag=${VERSION}-${GITHUB_SHA::7}` で `{version}-{git-sha}` 形式を指定（`:latest` タグは廃止。[Dockerイメージ戦略.md](../docker/Dockerイメージ戦略.md) のタグ規則に準拠） |

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

## CI/CD セキュリティ・信頼性改善

### CI/Deploy 重複解消

`ci.yaml` と `deploy.yaml` に `paths-ignore` を追加し、個別サービスワークフロー（`*-ci.yaml`, `*-deploy.yaml`）がカバーするパスを除外。これにより同一変更に対する CI/Deploy の重複実行を防止する。

### Helm デプロイの安全性強化

全 `helm upgrade --install` に `--atomic --wait --timeout 5m` を追加:
- `--atomic`: デプロイ失敗時に自動ロールバック
- `--wait`: 全 Pod が Ready になるまで待機
- `--timeout 5m`: 最大待機時間を設定

### スモークテスト失敗時のデプロイ中止

`_service-deploy.yaml` と `deploy.yaml` のスモークテストステップから `|| echo "::warning::..."` を削除し、ヘルスチェック失敗時にワークフローを中止するように変更。

### :latest タグ廃止

Docker イメージタグから `:latest` を削除。`{version}` と `{version}-{git-sha}` の2つのタグのみを使用し、デプロイの再現性を確保する。

### Kong diff の self-hosted ランナー化

`kong-sync.yaml` の `diff` ジョブを `runs-on: self-hosted` に変更。Kong Admin API は内部ネットワークにあるため、クラスタ内ランナーでの実行が必要。

### decK CLI バージョン固定・SHA 検証

`kong-sync.yaml` の decK CLI インストールをバージョン固定（v1.40.2）+ SHA256 チェックサム検証に変更。`:latest` の動的取得によるサプライチェーン攻撃リスクを排除。

### pre-commit ワークスペース対応

`.pre-commit-config.yaml` をモノレポのワークスペース構造に対応:
- Rust: `regions/system` と `CLI` のワークスペースを分離して fmt/clippy を実行
- Go: `go.work` 経由の一括実行
- gitleaks フックを追加してシークレットのコミットを防止

### Dependabot 複数ディレクトリ対応

`.github/dependabot.yml` で `directories` フィールドを使用し、モノレポ内の全マニフェストディレクトリ（Go, Rust, npm, Dart）に対応。ルートディレクトリのみの設定から個別ディレクトリ指定に拡張。

## npm スクリプト実行の --if-present 削除方針

### 背景

CI（`ci.yaml`）と justfile の npm スクリプト実行で `--if-present` フラグを使用している箇所がある（`npm test --if-present`, `npm run build --if-present`, `npm run format --if-present` 等）。`--if-present` は該当スクリプトが `package.json` に定義されていない場合にサイレントスキップするフラグだが、以下の問題がある:

- **テストの欠落を隠蔽**: `test` スクリプトが未定義のパッケージでもエラーにならず、テスト未実装が検出されない
- **ビルド漏れの検出遅延**: `build` スクリプトの定義漏れが CI で検出されない

### 対応方針

- 全 TypeScript パッケージに `test`, `build`, `format` スクリプトを明示的に定義する
- 定義を確認後、CI と justfile から `--if-present` を削除する
- `validate-ts-lockfiles` lint ジョブを拡張し、必須スクリプト（`test`, `build`）の存在チェックを追加する

## tier 別 CI ワークフローの拡張

### 背景

統合テスト（`integration-test.yaml`）は当初 system tier のサーバーのみを対象としていたが、service tier（order, payment, inventory）と business tier（domain-master）のサーバーも統合テストの対象とする必要がある。

### 対応方針

- `ci-list-integration-servers.sh` に `--tier` オプションを追加し、tier 別のサーバーリスト取得を可能にする
- 各 tier は独自のワークスペース（`regions/system`, `regions/service/*/server/rust/*`, `regions/business/*/server/rust/*`）を持つため、tier 別の integration-test ジョブを分離する
- `modules.yaml` に `tier` フィールドを追加し、CI スクリプトが tier を自動判定できるようにする

## 技術監査対応の改善事項

### Vault 結合テスト（integration-test.yaml）

`integration-test.yaml` のサービスコンテナに `hashicorp/vault:1.15` を追加し、サービス間認証の結合テストを可能にした。Vault コンテナは dev モード（`VAULT_DEV_ROOT_TOKEN_ID`）で起動し、統合テスト内で AppRole 認証やシークレット取得のフローを検証できる。

```yaml
services:
  vault:
    image: hashicorp/vault:1.15
    env:
      VAULT_DEV_ROOT_TOKEN_ID: dev-token
      VAULT_DEV_LISTEN_ADDRESS: "0.0.0.0:8200"
    ports:
      - 8200:8200
    options: >-
      --health-cmd "vault status"
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5
```

### SBOM アーティファクト保存（_service-deploy.yaml）

`_service-deploy.yaml` の build-push ジョブで `actions/upload-artifact@v4` により SBOM（Software Bill of Materials）をアーティファクトとして保存する。Trivy の `--format cyclonedx` で生成した SBOM を CI アーティファクトとしてアップロードし、監査時のソフトウェア構成追跡を可能にする。

```yaml
- name: Generate SBOM
  uses: aquasecurity/trivy-action@0.28.0
  with:
    scan-type: image
    image-ref: ${{ env.IMAGE_REF }}
    format: cyclonedx
    output: sbom.json

- name: Upload SBOM artifact
  uses: actions/upload-artifact@v4
  with:
    name: sbom-${{ inputs.service-name }}
    path: sbom.json
    retention-days: 90
```

### npm キャッシュ確認済み（ci.yaml）

`ci.yaml` の `lint-ts` / `test-ts` ジョブで `actions/setup-node@v4` に `cache: 'npm'` を設定済みであることを確認。npm のグローバルキャッシュを活用し、依存関係のインストール時間を短縮している。キャッシュ戦略テーブル（本ドキュメント「キャッシュ戦略」セクション）にも反映済み。

### KAFKA_CLUSTER_ID 共通化（integration-test.yaml）

`integration-test.yaml` で Kafka サービスコンテナの `CLUSTER_ID` をワークフローレベルの `env` に定義し、DRY 化した。複数ジョブ間で同一の値を使い回す場合に定義が分散しないようにする。

```yaml
# ワークフローレベル env で共通定義
env:
  KAFKA_CLUSTER_ID: "5L6g3nShT-eMCtK--X86sw"
```

### デプロイヘルスチェックタイムアウト（deploy.yaml）

`deploy.yaml` のスモークテスト（ヘルスチェック）で使用する `wget` に `--timeout=5` を追加し、応答のないサービスに対してタイムアウトを設定した。これにより、ヘルスチェックがハングしてワークフロー全体の実行時間を浪費することを防止する。

```yaml
- name: Smoke test
  run: |
    wget --timeout=5 --tries=3 -qO- http://localhost:${{ steps.port.outputs.value }}/health
```

### カバレッジ閾値ロードマップ（LOW-08）

テストカバレッジの段階的引き上げ計画。`coverage-rust.yaml` で計測した結果を基に、閾値を徐々に引き上げる。

| フェーズ | ライブラリ閾値 | サーバー閾値 | 目標時期 | 内容 |
|----------|---------------|-------------|----------|------|
| Phase 1 | - | - | 現在 | ベースライン測定（現状の閾値を記録、CI は計測のみで失敗させない） |
| Phase 2 | 60% | 40% | Phase 1 + 2ヶ月 | 最低限の閾値を設定し、CI でカバレッジ低下を検出 |
| Phase 3 | 80% | 60% | Phase 2 + 3ヶ月 | 本番品質の閾値を適用、新規コードは必ずテスト付き |

- **Phase 1（現在）**: `coverage-rust.yaml` でカバレッジレポートを生成・アップロードするが、閾値チェックは行わない。各サービス・ライブラリのベースラインを測定する
- **Phase 2**: `cargo-tarpaulin` の `--fail-under` オプションでライブラリ 60% / サーバー 40% を設定。閾値未達の場合 CI を警告（`continue-on-error: true`）とする
- **Phase 3**: 閾値をライブラリ 80% / サーバー 60% に引き上げ、`continue-on-error` を外して CI を失敗させる。新規コードのマージには必ずテストカバレッジの維持が求められる

---

## Docker ダイジェスト固定の自動化

### 背景

Dockerfile および GitHub Actions ワークフローのベースイメージ・アクションは SHA ダイジェストでピン留めし、サプライチェーン攻撃を防止する。ダイジェストの手動管理は運用負荷が高いため、自動化ツールで更新を管理する。

### 自動更新ツール: Renovate / Dependabot

| ツール | 設定ファイル | 対象 |
| --- | --- | --- |
| Dependabot | `.github/dependabot.yml` | GitHub Actions のアクションバージョン |
| Renovate | `renovate.json`（導入時） | Dockerfile のベースイメージダイジェスト |

#### Dependabot による GitHub Actions 自動更新

`.github/dependabot.yml` で `package-ecosystem: github-actions` を設定し、アクションの SHA ピン留めを自動更新する。

```yaml
# .github/dependabot.yml（抜粋）
- package-ecosystem: "github-actions"
  directory: "/"
  schedule:
    interval: "weekly"
```

#### Renovate による Dockerfile ダイジェスト自動更新（導入検討中）

Renovate は Dockerfile 内のベースイメージタグをダイジェスト付きに自動変換・更新する。

```json
{
  "extends": ["config:recommended"],
  "dockerfile": {
    "pinDigests": true
  }
}
```

### CI パイプラインでのダイジェスト検証

`security.yaml` の `image-scan` ジョブで Harbor レジストリ上のイメージを Trivy スキャンし、既知の脆弱性を検出する。ダイジェスト固定により、スキャン済みイメージと実際にデプロイされるイメージの一致を保証する。

### 手動更新手順

自動化ツールで対応できない場合の手動更新手順:

1. 対象イメージの最新ダイジェストを取得する
   ```bash
   # Docker Hub のイメージダイジェスト取得
   docker pull rust:1.93-bookworm
   docker inspect --format='{{index .RepoDigests 0}}' rust:1.93-bookworm
   ```
2. Dockerfile または workflow YAML の該当行を更新する
3. `buf breaking` / `cargo check` / CI でビルド検証する
4. PR を作成し、セキュリティチームのレビューを受ける

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
