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
| CI                | `ci.yaml`         | PR 作成・更新時 + **毎週月曜 03:00 UTC（schedule）** | lint → test → build + モジュールレジストリ検証 + ティア間依存検証。schedule 時は変更検出をバイパスし全テストを実行する |
| Deploy            | `deploy.yaml`     | main マージ時               | image push → deploy     |
| **Rust サービス CI (reusable)** | `_rust-service-ci.yaml` | `workflow_call` | Rust サービスの共通 lint → test → build |
| **Go サービス CI (reusable)** | `_go-service-ci.yaml` | `workflow_call` | Go サービスの共通 lint → test → build |
| **サービス Deploy (reusable)** | `_service-deploy.yaml` | `workflow_call` | サービスの共通 build-push → deploy (dev→staging→prod) |
| Proto Check       | `proto.yaml`      | `api/proto/**` 変更時       | proto lint + breaking（ci.yaml の lint-proto ジョブでも実行） |
| Security Scan     | `security.yaml`   | 日次 + PR 時 + main マージ後 | 脆弱性スキャン。image-scan は schedule/push 時のみ実行（PR 時はイメージ未存在）。全ティアのサービスイメージをマトリクスでスキャン |
| Kong Config Sync  | `kong-sync.yaml`  | main マージ時 (`infra/kong/**` 変更) | dev → staging → prod    |
| OpenAPI Lint      | `api-lint.yaml`   | push (`**/api/openapi/**`)  | OpenAPI バリデーション & SDK 生成 |
| Tauri GUI Build   | `tauri-build.yaml` | PR 時 + main マージ時 (`CLI/crates/k1s0-gui/**` 変更) | GUI クロスプラットフォームビルド（[TauriGUI設計](../../cli/gui/TauriGUI設計.md) 参照） |
| auth CI           | `auth-ci.yaml`    | PR 時 (`regions/system/server/rust/auth/**`) | `_rust-service-ci.yaml` 呼び出し |
| app-registry CI   | `app-registry-ci.yaml` | PR 時 (`regions/system/server/rust/app-registry/**`) | `_rust-service-ci.yaml` 呼び出し |
| config CI         | `config-ci.yaml`  | PR 時 (`regions/system/server/rust/config/**`) | `_rust-service-ci.yaml` 呼び出し |
| saga CI           | `saga-ci.yaml`    | PR 時 (`regions/system/server/rust/saga/**`) | `_rust-service-ci.yaml` 呼び出し |
| dlq-manager CI    | `dlq-manager-ci.yaml` | PR 時 (`regions/system/server/rust/dlq-manager/**`) | `_rust-service-ci.yaml` 呼び出し |
| task CI           | `task-ci.yaml`    | PR 時 (`regions/service/task/server/**`, `regions/service/task/client/**`) | `_rust-service-ci.yaml` 呼び出し (standalone)。クライアント変更時もサーバーCI起動し契約整合性を確認する |
| board CI          | `board-ci.yaml`   | PR 時 (`regions/service/board/server/**`, `regions/service/board/client/**`) | `_rust-service-ci.yaml` 呼び出し (standalone) |
| activity CI       | `activity-ci.yaml` | PR 時 (`regions/service/activity/server/**`, `regions/service/activity/client/**`) | `_rust-service-ci.yaml` 呼び出し (standalone) |
| project-master CI | `project-master-ci.yaml` | PR 時 (`regions/business/taskmanagement/**`) | `_rust-service-ci.yaml` 呼び出し (standalone) |
| bff-proxy CI      | `bff-proxy-ci.yaml` | PR 時 (`regions/system/server/go/bff-proxy/**`) | `_go-service-ci.yaml` 呼び出し |
| Integration Test  | `integration-test.yaml` | PR 時 (`regions/system/{server,library}/rust/**`, `regions/business/*/server/rust/**`, `regions/service/*/server/rust/**`, `Cargo.{toml,lock}`, `regions/**/database/postgres/migrations/**`, `infra/docker/init-db/**`, `scripts/ci-list-integration-servers.sh`, `scripts/list-modules.sh`) | postgres:17 + kafka:3.8.0 起動、system/business/service 全ティア対応。`ci-list-integration-servers.sh [system\|business\|service]` でティア別サーバー自動検出・パッケージ単位並列統合テスト（test-utils feature 自動検出）。DB migration 変更・`infra/docker/init-db/` 変更（テスト環境スキーマに影響）・CI スクリプト変更時も起動する |
| Golden Path Compile | `golden-path-compile.yaml` | PR 時 (`CLI/crates/k1s0-codegen/**`, `CLI/templates/**`) | CLI テンプレートからサーバーを生成し `cargo check` でコンパイル検証 |
| auth Deploy       | `auth-deploy.yaml` | main マージ時 (`regions/system/server/rust/auth/**`) | `_service-deploy.yaml` 呼び出し |
| app-registry Deploy | `app-registry-deploy.yaml` | main マージ時 (`regions/system/server/rust/app-registry/**`) | `_service-deploy.yaml` 呼び出し |
| config Deploy     | `config-deploy.yaml` | main マージ時 (`regions/system/server/rust/config/**`) | `_service-deploy.yaml` 呼び出し |
| saga Deploy       | `saga-deploy.yaml` | main マージ時 (`regions/system/server/rust/saga/**`) | `_service-deploy.yaml` 呼び出し |
| dlq-manager Deploy | `dlq-manager-deploy.yaml` | main マージ時 (`regions/system/server/rust/dlq-manager/**`) | `_service-deploy.yaml` 呼び出し |
| bff-proxy Deploy  | `bff-proxy-deploy.yaml` | main マージ時 (`regions/system/server/go/bff-proxy/**`) | `_service-deploy.yaml` 呼び出し (port-forward) |
| App Publish       | `publish-app.yaml` | Git タグ push (`v*`) + `regions/**/flutter/**` 変更 | Flutter デスクトップアプリのクロスプラットフォームビルド・署名・PV へのアップロード・App Registry へのメタデータ登録（[アプリ配布基盤設計](../distribution/アプリ配布基盤設計.md) 参照）|
| coverage-rust     | `coverage-rust.yaml` | PR 時 (`regions/**/rust/**`) | Rust テストカバレッジを cargo-tarpaulin で計測し、JSON + HTML レポートをアーティファクトとしてアップロードする |

### ci.yaml 追加ジョブ（セキュリティ監査対応）

| ジョブ名 | 目的 |
| --- | --- |
| `check-tier-deps` | H5対応: system→business→service のティア依存方向を強制し、逆方向依存（system が business/service に依存する等）を CI で検出・失敗させる |
| `build-gui-windows` | GUI フロントエンド（CLI/crates/k1s0-gui/ui）の Windows 環境でのビルド検証。Rolldown/Vite の Windows 互換性を可視化する。既知の Rolldown panic で失敗する可能性があるため `continue-on-error: true` を設定。失敗時は `::warning::` アノテーションで既知課題である旨を通知する |
| `iac-validation` | IaC 検証: `infra/terraform/` の `terraform fmt -check` および dev/prod 環境の `terraform validate`（バックエンド初期化なし）を実行し、Terraform 構文・フォーマット不整合を CI で検出する |

### TypeScript パッケージマネージャー（pnpm 採用）

TypeScript/React パッケージのインストールは `npm ci` から `pnpm install --frozen-lockfile` へ移行した。

**理由:** `pnpm-workspace.yaml` で `workspace:*` 依存（モノリポ内パッケージ間参照）を管理しており、`npm ci` ではこの依存が解決できない。`pnpm` を使用することで `workspace:*` 参照が正しく解決される。

**影響範囲:**
- `.github/workflows/ci.yaml`: `lint-ts`, `test-ts`, `coverage-ts`, `build-ts` の各ジョブで `pnpm/action-setup@v4` を追加し、`cache: 'npm'` → `cache: 'pnpm'` に変更
- `justfile`: `lint-ts`, `test-ts`, `fmt-ts`, `build-ts` の各レシピで `npm ci` → `pnpm install --frozen-lockfile`、`npm run` → `pnpm run` に変更

> **注意:** `build-gui-windows` ジョブは Windows ネイティブ環境のため `npm ci` を維持する（`CLI/crates/k1s0-gui/ui` は `pnpm-workspace.yaml` のスコープ外）。

### Windows GUI Build の Known Issue

`build-gui-windows` ジョブは Rolldown の Windows 対応 panic が発生する可能性がある既知の問題を抱えている。

- `continue-on-error: true` を設定し、失敗してもパイプライン全体はブロックしない
- 失敗時はジョブの末尾ステップで `::warning::` アノテーションを出力し、既知課題として追跡可能にする
- 解消されたら `continue-on-error: true` を削除し、通常の必須ジョブに昇格させること

### security.yaml 変更点（セキュリティ監査対応）

| 変更点 | 内容 |
| --- | --- |
| H7対応: `dart-outdated.outcome` チェック追加 | `スキャン結果の集約` ステップに `dart-outdated.outcome` の failure 判定を追加し、Dart 依存チェック失敗時も CI 全体が失敗するよう修正 |

### 定期実行スケジュール（週次全テスト）

`ci.yaml` には `schedule` トリガーが設定されており、毎週月曜 03:00 UTC に全テストを自動実行する。

| 目的 | 内容 |
| --- | --- |
| 回帰テスト検知 | PR がない期間でも依存ライブラリの更新・外部 API の変更による回帰を週次で検知する |
| 変更検出バイパス | `schedule` トリガー時は `detect-changes` が全言語フラグを `'true'` に設定し、`paths-filter` をスキップして全テストを実行する |
| 全モジュールテスト | `go_modules` 等を空文字列にすることで `_test.yaml` が `list-modules.sh` 経由で全モジュールをテストする |

**設定**:
```yaml
schedule:
  # 毎週月曜 03:00 UTC に全テストを定期実行し、回帰テスト失敗を検知する
  - cron: '0 3 * * 1'
```

### CI ワークフロー（ci.yaml）

```yaml
# .github/workflows/ci.yaml
name: CI

on:
  pull_request:
    branches: [main]
  schedule:
    # 毎週月曜 03:00 UTC に全テストを定期実行し、回帰テスト失敗を検知する
    # schedule 時は detect-changes が全言語を 'true' に設定して変更検出をバイパスする
    - cron: '0 3 * * 1'

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

  # H-8対応: paths-filter 条件を廃止し、全 PR で常に Helm lint を実行する。
  # Library Chart (k1s0-common) の変更はコンシューマーチャートに波及するため、
  # Helm ファイル変更がない PR でも継続的にバリデーションを実施する。
  helm-lint:
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
        uses: aquasecurity/trivy-action@76071ef0d7ec797419534a183b498b4d6a132a02 # 0.29.0
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
          # infra/helm/services/ 変更時は対応するサービスを逆引きしてデプロイ対象に含める
          CHANGED=$(git diff --name-only ${{ github.event.before }} ${{ github.sha }} | \
            (grep -E '^(regions/|infra/helm/services/)' || true) | \
            sed 's|^infra/helm/services/||; s|^regions/||' | \
            while IFS= read -r path; do
              case "$path" in
                system/server/*/*/*)       echo "$path" | cut -d'/' -f1-4 ;;
                business/*/server/*/*/*)   echo "$path" | cut -d'/' -f1-5 ;;
                service/*/server/*/*/*)    echo "$path" | cut -d'/' -f1-5 ;;
                system/*/Chart.yaml|system/*/*.yaml)   echo "system/server/$(echo "$path" | cut -d'/' -f2)" ;;
                business/*/*/Chart.yaml|business/*/*.yaml) echo "business/$(echo "$path" | cut -d'/' -f2)/server" ;;
                service/*/Chart.yaml|service/*/*.yaml) echo "service/$(echo "$path" | cut -d'/' -f2)/server" ;;
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
            --certificate-identity-regexp "^https://github\\.com/k1s0-org/k1s0/\\.github/workflows/.*" \
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
            --certificate-identity-regexp "^https://github\\.com/k1s0-org/k1s0/\\.github/workflows/.*" \
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
            --certificate-identity-regexp "^https://github\\.com/k1s0-org/k1s0/\\.github/workflows/.*" \
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
| `trivy-scan` | 日次 + PR 時 | リポジトリ全体のファイルシステム脆弱性スキャン（MEDIUM/HIGH/CRITICAL） |
| `dependency-check` | 日次 + PR 時 | Go / Rust / npm / Dart の依存関係脆弱性チェック（`list-modules.sh` ベース） |
| `image-scan` | 日次 + main マージ後 | 全サービスのコンテナイメージ脆弱性スキャン（system / business / service 全ティア対象）。MEDIUM/HIGH/CRITICAL 検出時は `exit-code: 1` でジョブ失敗 |
| `iac-scan` | 日次 + PR 時 | `infra/` ディレクトリの Terraform / Kubernetes マニフェスト構成ミス検出（Trivy config scan） |
| `license-scan` | 日次 + PR 時 | 依存関係のライセンスコンプライアンスチェック（Trivy license scanner） |
| `sast` | 日次 + PR 時 | Go (gosec) + Rust (clippy security lints) による SAST スキャン |

#### IaC スキャン

`iac-scan` ジョブは Trivy の `config` スキャンタイプを使用して `infra/` ディレクトリを走査する。Terraform 定義ファイルおよび Kubernetes マニフェストの設定ミス（セキュリティグループの過剰開放、暗号化未設定、特権コンテナ等）を MEDIUM/HIGH/CRITICAL レベルで検出し、検出時は `exit-code: 1` でジョブを失敗させる。

#### ライセンススキャン

`license-scan` ジョブは Trivy の `license` スキャナーを使用してリポジトリ全体の依存関係ライセンスを検証する。許容されないライセンス（GPL 等の強力なコピーレフトライセンス）が MEDIUM/HIGH/CRITICAL として検出された場合、ジョブを失敗させる。これにより、意図しないライセンス汚染を CI レベルで防止する。

#### イメージスキャン拡大

`image-scan` ジョブは以前 order サービスのみを対象としていたが、現在は system / business / service 全ティアの全サービスに拡大している。マトリクス戦略（`fail-fast: false`）で並列実行し、各サービスのコンテナイメージを個別にスキャンする。

**実行タイミングの拡大（2026-03-21）:** PR 時はイメージが存在しないため従来通りスキップするが、`schedule`（日次）に加えて `push`（main マージ後）でも実行するよう変更した。これにより、main マージ直後のイメージも自動的にスキャンされる。

**exit-code: 1 追加（2026-03-21）:** HIGH/CRITICAL 脆弱性検出時にジョブを失敗させる `exit-code: 1` を追加した。他のスキャンジョブ（`trivy-scan`, `iac-scan`, `license-scan`）と同様の挙動に統一した。

**イメージバージョン固定（2026-03-21）:** `:latest` タグへの依存を廃止し、直近の Git タグから動的にバージョンを取得して `harbor.internal.example.com/k1s0-{tier}/{service}:{version}` 形式でスキャンする。

| ティア | 対象サービス |
| --- | --- |
| system | auth, config, saga, bff-proxy, app-registry, dlq-manager, graphql-gateway |
| business | project-master |
| service | task, board, activity |

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
        uses: aquasecurity/trivy-action@76071ef0d7ec797419534a183b498b4d6a132a02 # 0.29.0
        with:
          scan-type: fs
          scan-ref: .
          # MEDIUM も含めてスキャン（M-23対応）
          severity: MEDIUM,HIGH,CRITICAL
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

  # 全ティアのコンテナイメージ脆弱性スキャン（定期実行 + main マージ後）
  # マトリクス戦略で system / business / service ティアの全サービスイメージをスキャン
  image-scan:
    runs-on: ubuntu-latest
    # PR 時はイメージが存在しないためスキップ。スケジュール実行と main マージ後のみ実行する
    if: github.event_name == 'schedule' || github.event_name == 'push'
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
            service: project-master
          # --- service ティア ---
          - tier: service
            service: task
          - tier: service
            service: board
          - tier: service
            service: activity
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      # 直近の Git タグからバージョンを取得（:latest タグに依存しないため）
      - name: Set version
        id: version
        run: |
          VERSION=$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || echo "0.0.0")
          echo "value=${VERSION}" >> "$GITHUB_OUTPUT"
      - name: コンテナイメージの脆弱性スキャン (${{ matrix.tier }}/${{ matrix.service }})
        uses: aquasecurity/trivy-action@76071ef0d7ec797419534a183b498b4d6a132a02 # 0.29.0
        with:
          scan-type: image
          # 最新のバージョンタグを動的に取得してスキャン（:latest は使用しない）
          image-ref: harbor.internal.example.com/k1s0-${{ matrix.tier }}/${{ matrix.service }}:${{ steps.version.outputs.value }}
          # MEDIUM も含めてスキャン（M-23対応）
          severity: MEDIUM,HIGH,CRITICAL
          format: table
          # MEDIUM/HIGH/CRITICAL 脆弱性検出時にジョブを失敗させる（他のスキャンと統一）
          exit-code: 1

  # IaC（Infrastructure as Code）構成ミススキャン
  # Terraform / Kubernetes マニフェストの設定ミスを Trivy で検出
  iac-scan:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@v4
      - name: Trivy IaC 構成スキャン
        uses: aquasecurity/trivy-action@76071ef0d7ec797419534a183b498b4d6a132a02 # 0.29.0
        with:
          scan-type: config
          scan-ref: infra/
          # MEDIUM も含めてスキャン（M-23対応）
          severity: MEDIUM,HIGH,CRITICAL
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
        uses: aquasecurity/trivy-action@76071ef0d7ec797419534a183b498b4d6a132a02 # 0.29.0
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
- **注意（H-05 対応）**: 本番環境の Kafka クラスター（`infra/messaging/kafka/kafka-cluster.yaml`）では
  PLAINTEXT リスナー（port 9092）を削除し、TLS リスナー（port 9093）のみを使用する。
  CI テスト用のサービスコンテナは GitHub Actions 内のネットワーク分離環境であるため、
  `PLAINTEXT://:9092` を使用しているが、本番環境では全通信を TLS で暗号化することが必須。
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
| イメージタグ     | `--set image.tag=${VERSION}-${GITHUB_SHA::12}` で `{version}-{git-sha}` 形式を指定（`:latest` タグは廃止。[Dockerイメージ戦略.md](../docker/Dockerイメージ戦略.md) のタグ規則に準拠） |

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
| `type` | string | 必須 | `server` / `library` / `client` / `cli` / `workspace` / `proto` / `database` / `infra` |
| `workspace` | string | 任意 | Cargo/Go ワークスペースルートパス |
| `skip-ci` | bool | 任意 | `true` の場合 CI のリント・テスト・ビルドをスキップ |

### database / infra モジュールタイプの追加

技術監査対応として、`modules.yaml` に `database` および `infra` タイプのモジュールを追加した。

**`database` タイプ**: system tier の全データベースクレート（`regions/system/database/*-db`）を登録。23 個の DB クレートが対象。`workspace: regions/system` を指定し、Cargo ワークスペースに属する。

**`infra` タイプ**: `infra/` 配下のインフラストラクチャ設定（Ansible, Docker, Helm, Istio, Keycloak, Kong, Kubernetes, Terraform, Vault 等）を登録。全モジュールに `skip-ci: true` を設定し、CI のリント・テスト・ビルドの対象外とする（インフラ設定は言語固有の CI ジョブでは検証しないため）。

なお `validate-modules` ジョブの双方向チェック（ディスク上のマニフェストと `modules.yaml` の整合性検証）は、マニフェストファイルを持つエントリのみを比較対象とする。`type: database`（SQL ディレクトリ）と `lang: yaml`（infra 設定）はマニフェストファイルを持たないため比較対象から除外し、誤検知（false positive）を防ぐ。

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
- **比較対象**: `lang: rust`（`type: database` 除く）・`lang: go`・`lang: typescript`・`lang: dart` のエントリのみ。`type: database` は SQL マイグレーションファイルのみのディレクトリ（Cargo.toml なし）、`lang: yaml` は infra 設定のため除外。

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

統合テスト（`integration-test.yaml`）は当初 system tier のサーバーのみを対象としていたが、service tier（task, board, activity）と business tier（project-master）のサーバーも統合テストの対象とする必要がある。

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

### Trivy バージョン統一・SARIF レポートアップロード

技術監査対応として、全ワークフロー（`ci.yaml`, `deploy.yaml`, `security.yaml`, `publish-app.yaml`, `_service-deploy.yaml`）の `aquasecurity/trivy-action` を **0.29.0**（SHA ピン留め: `76071ef0d7ec797419534a183b498b4d6a132a02`）に統一した。従来は `@master` や `@0.28.0` が混在しており、以下の問題があった:

- `@master` 参照によるサプライチェーン攻撃リスク
- バージョン不一致による脆弱性データベースの差異

**deploy.yaml への SARIF レポート追加**: `deploy.yaml` の `build-and-push` ジョブに Trivy SARIF レポートの生成・アップロードステップを追加した。ビルド済みイメージに対して CRITICAL/HIGH の脆弱性スキャンを実行し、結果を SARIF 形式でアーティファクトに保存する。

```yaml
# deploy.yaml build-and-push ジョブ内（ビルド・プッシュ後）
- name: Run Trivy vulnerability scanner
  uses: aquasecurity/trivy-action@76071ef0d7ec797419534a183b498b4d6a132a02 # 0.29.0
  with:
    image-ref: ${{ env.REGISTRY }}/${{ steps.image.outputs.project }}/${{ steps.image.outputs.service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
    format: 'sarif'
    output: 'trivy-results.sarif'
    severity: 'CRITICAL,HIGH'
- name: Upload Trivy SARIF report
  uses: actions/upload-artifact@v4
  if: always()
  with:
    name: trivy-sarif-${{ steps.image.outputs.service_name }}
    path: trivy-results.sarif
    if-no-files-found: warn
```

### SBOM アーティファクト保存（_service-deploy.yaml）

`_service-deploy.yaml` の build-push ジョブで `actions/upload-artifact@v4` により SBOM（Software Bill of Materials）をアーティファクトとして保存する。Trivy の `--format cyclonedx` で生成した SBOM を CI アーティファクトとしてアップロードし、監査時のソフトウェア構成追跡を可能にする。

```yaml
- name: Generate SBOM
  uses: aquasecurity/trivy-action@76071ef0d7ec797419534a183b498b4d6a132a02 # 0.29.0
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

### Dart カバレッジ計測方式

`ci.yaml` の `test-dart` ジョブでは以下の方式で Dart/Flutter パッケージのカバレッジを計測する。

- **Flutter パッケージ**: `flutter test --coverage` で `coverage/lcov.info` を生成し、`lcov --summary` で行カバレッジ率を算出する
- **非Flutter Dart パッケージ**: `dart test --coverage=coverage` でカバレッジデータを収集した後、`dart pub global run coverage:format_coverage` で lcov 形式に変換する
- テスト失敗時は `|| { failed=1; }` パターンで記録し、全パッケージのテスト完了後にまとめて終了コード 1 で失敗させる（`|| true` による偽陽性を排除）

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

### Dockerfileダイジェスト固定自動化

`.github/workflows/pin-docker-digests.yaml` ワークフローと `scripts/pin-docker-digests.sh` スクリプトにより、リポジトリ内すべての Dockerfile の FROM 行を `@sha256:` 形式に自動更新する仕組みを整備している。

#### 自動実行スケジュール

| 項目 | 内容 |
| --- | --- |
| スケジュール | 毎週月曜日 09:00 UTC（日本時間 18:00） |
| cron 設定 | `0 9 * * 1` |
| 手動実行 | GitHub Actions の "Run workflow"（`workflow_dispatch`）から手動トリガーも可能 |

#### 動作フロー

1. `scripts/pin-docker-digests.sh` がリポジトリ内の全 Dockerfile を走査し、各 FROM 行のベースイメージを最新の `@sha256:` ダイジェスト付き形式に更新する
2. 変更がある場合のみ PR を自動作成する（変更がなければジョブは正常終了し PR は作成されない）
3. PR には `security` / `automated` ラベルが付与され、変更内容と Trivy スキャン確認チェックリストが記載される

#### 運用担当者の対応

運用担当者は週次で自動生成された PR をレビュー・マージすることで、イメージのサプライチェーンセキュリティを継続的に維持できる。

| アクション | 手順 |
| --- | --- |
| 週次 PR レビュー | CI の Trivy イメージスキャンが PASS していることを確認してマージ |
| 手動トリガー | GitHub Actions の "Pin Docker Digests" ワークフローページから "Run workflow" をクリック |
| 緊急対応 | `workflow_dispatch` で手動実行し、特定イメージの脆弱性対応を即座に反映 |

#### ワークフロー設定（抜粋）

```yaml
# .github/workflows/pin-docker-digests.yaml（抜粋）
name: Pin Docker Digests

on:
  schedule:
    # 毎週月曜日 09:00 UTC（日本時間 18:00）に実行する
    - cron: '0 9 * * 1'
  workflow_dispatch:
    # 手動実行も可能にする（テスト・緊急対応用）

jobs:
  pin-digests:
    permissions:
      contents: write
      pull-requests: write
    steps:
      - name: Docker ダイジェストを固定する
        run: scripts/pin-docker-digests.sh
      - name: PR を作成する
        uses: peter-evans/create-pull-request@... # v7
        with:
          branch: chore/pin-docker-digests
          labels: |
            security
            automated
```

### 自動更新ツール: Renovate / Dependabot

| ツール | 設定ファイル | 対象 |
| --- | --- | --- |
| Dependabot | `.github/dependabot.yml` | GitHub Actions のアクションバージョン |
| Renovate | `renovate.json`（導入時） | Dockerfile のベースイメージダイジェスト |
| `pin-docker-digests.yaml` | `.github/workflows/pin-docker-digests.yaml` | Dockerfile の FROM 行 `@sha256:` ダイジェスト（週次自動 PR） |

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

#### SAST スキャン（2026-03-21 追加）

技術品質監査の指摘（High 4-3）に対応し、`sast` ジョブを追加した。

| 対象言語 | ツール | スクリプト |
| --- | --- | --- |
| Go | gosec | `scripts/security/gosec-scan.sh` |
| Rust | clippy security lints | `scripts/security/clippy-security.sh` |

**gosec** は Go のソースコードを解析し、SQL インジェクション・ハードコードされた認証情報・
不安全な暗号化・ファイルパーミッション問題等の一般的なセキュリティ脆弱性を検出する。

**clippy security lints** は Rust の `suspicious` と `correctness` lint カテゴリを有効化し、
整数演算の問題・パニック誘発コード等を検出する。

各スキャンは `continue-on-error: true` で独立実行し、最後の集約ステップで一括して
ジョブ全体の成否を判定する（`dependency-check` ジョブと同パターン）。

---

## Doc Sync (2026-03-21) Phase 2 対応

### P2-35: security-gate ジョブの追加（security.yaml）

`security.yaml` に全セキュリティジョブを集約する `security-gate` ジョブを追加した。

```yaml
# security.yaml 末尾に追加
# 全セキュリティスキャンジョブの結果を集約し、ブランチ保護ルールで参照される単一のゲートジョブ
security-gate:
  needs:
    - trivy-scan
    - dependency-check
    - iac-scan
    - license-scan
    - sast
  runs-on: ubuntu-latest
  if: always()
  steps:
    - name: 全スキャン結果の集約チェック
      run: |
        if [ "${{ needs.trivy-scan.result }}" = "failure" ] || \
           [ "${{ needs.dependency-check.result }}" = "failure" ] || \
           [ "${{ needs.iac-scan.result }}" = "failure" ] || \
           [ "${{ needs.license-scan.result }}" = "failure" ] || \
           [ "${{ needs.sast.result }}" = "failure" ]; then
          echo "::error::One or more security scans failed"
          exit 1
        fi
        echo "All security scans passed"
```

**ブランチ保護ルールとの連携**: リポジトリのブランチ保護ルール（Settings → Branches → Required status checks）で `security-gate` のみを必須ジョブとして指定する。各スキャンジョブを個別に必須化する必要がなくなり、スキャンジョブの追加・削除に際してブランチ保護ルールの変更が不要になる。

| 変更点 | 内容 |
| --- | --- |
| `security-gate` ジョブ追加 | 全スキャン結果（trivy-scan / dependency-check / iac-scan / license-scan / sast）を `needs` で集約し、いずれかが failure の場合にジョブ全体を失敗させる |
| ブランチ保護ルール簡略化 | Required status checks に `security-gate` のみを登録すれば全スキャン結果を一元管理できる |

---

### P2-36: modules.yaml ベースの CI モジュール検出強化

#### detect-affected-modules.sh の更新

`detect-affected-modules.sh` が `modules.yaml` を読み込み、以下の条件に合致するモジュールを自動除外するように更新した。

| 除外条件 | フィールド | 値 |
| --- | --- | --- |
| CI スキップ指定 | `skip-ci` | `true` |
| アーカイブ済み | `status` | `archived` |

これにより、廃止・アーカイブ済みのモジュールが影響範囲検出の結果に誤って含まれることを防止する。`list-modules.sh` の `--no-skip-ci` フラグと同様のフィルタリングロジックを `detect-affected-modules.sh` にも適用している。

#### scripts/check-modules-consistency.sh の追加

`modules.yaml` と CI 設定（ワークフロー YAML）の整合性を検証するスクリプト `scripts/check-modules-consistency.sh` を追加した。

| チェック項目 | 内容 |
| --- | --- |
| モジュール登録漏れ | ディスク上に存在するモジュールが `modules.yaml` に未登録 → エラー |
| ゴースト登録 | `modules.yaml` に登録されているがディスク上に存在しない → エラー |
| CI ワークフロー対応漏れ | `status: stable` かつ `type: server` のモジュールに対応する `*-ci.yaml` が存在しない → 警告 |
| skip-ci/archived 整合性 | `skip-ci: true` のモジュールが CI ワークフローで参照されていないか確認 |

```bash
# 使い方
bash scripts/check-modules-consistency.sh

# CI での実行例（validate-modules ジョブに追加）
- name: modules.yaml 整合性チェック
  run: bash scripts/check-modules-consistency.sh
```

---

### P2-30: modules.yaml library_parity 3分類の改訂

`modules.yaml` の `library_parity` セクションの分類体系を以下の通り改訂した。

#### 変更前（旧分類）

| カテゴリ | 対象 |
| --- | --- |
| `all` | 全言語（Go / Rust / TypeScript / Dart）で実装 |
| `server_only` | サーバーサイドのみ（Go / Rust） |
| `rust_only` | Rust 専用 |
| `go_only` | Go 専用 |

#### 変更後（新分類）

| カテゴリ | 対象言語 | 説明 |
| --- | --- | --- |
| `core` | Go / Rust / TypeScript / Dart（4言語） | 全言語で同等の実装が必要なコアライブラリ（認証トークン処理・設定読み込み・エラー型等） |
| `server` | Go / Rust（2言語） | サーバーサイドのみで使用するライブラリ（DB クライアント・gRPC サーバーヘルパー等） |
| `client` | TypeScript / Dart（2言語） | クライアントサイドのみで使用するライブラリ（UI コンポーネント・ネイティブ API ラッパー等） |
| `lang_specific` | 単一言語 | 特定言語の固有機能を提供するライブラリ（Rust の unsafe 実装・Go の reflect 利用等） |

**改訂の背景**: 旧分類では `rust_only` / `go_only` のように言語名で分類していたため、新言語追加時に分類名の変更が必要だった。新分類では用途（`core` / `server` / `client` / `lang_specific`）で分類することで、言語体系の変更に依存しない拡張性を確保する。

---

## Doc Sync (2026-03-21)

### service tier サービス CI への実 DB 統合テストジョブ追加 [技術品質監査 T-01/T-02]

`board-ci.yaml`・`task-ci.yaml`・`activity-ci.yaml` に `integration-test` ジョブを追加した。
`ci`（`_rust-service-ci.yaml` 呼び出し）の完了後に実行され、PostgreSQL 16 サービスコンテナを起動して
`#[ignore]` 付き統合テストを `-- --include-ignored` で実行する。

| ワークフロー | 追加ジョブ | PostgreSQL サービス |
| --- | --- | --- |
| `board-ci.yaml` | `integration-test` | `postgres:16` (localhost:5432) |
| `task-ci.yaml` | `integration-test` | `postgres:16` (localhost:5432) |
| `activity-ci.yaml` | `integration-test` | `postgres:16` (localhost:5432) |

テストファイルは各サービスの `tests/integration_db_test.rs` に `#[ignore]` 属性付きで配置する。
`DATABASE_URL` 環境変数（`postgres://postgres:postgres@localhost:5432/test_db`）で接続先を設定する。

---

## Doc Sync (2026-03-21) M-009 対応

### Git SHA タグ長を 7 → 12 桁に変更 [技術品質監査 M-009]

`deploy.yaml` および `_service-deploy.yaml` の `Set short SHA` ステップで使用する `${GITHUB_SHA::7}` を
`${GITHUB_SHA::12}` に変更した。

**背景**: 7桁では約268億通りの衝突確率があり、大規模リポジトリでは SHA1 衝突が現実的なリスクとなる。
12桁は約1.6兆通りのエントロピーを持ち、実用上の衝突リスクをほぼゼロにできる。

| 変更前 | 変更後 | 影響箇所 |
| --- | --- | --- |
| `${GITHUB_SHA::7}` | `${GITHUB_SHA::12}` | `deploy.yaml`（4箇所）、`_service-deploy.yaml`（全箇所） |

イメージタグ形式: `{version}-{12桁git-sha}`（例: `1.2.3-a1b2c3d4e5f6`）

---

## Doc Sync (2026-03-22)

### codecov fail_ci_if_error を true に変更（M-16対応）

`_test.yaml` の全カバレッジアップロードジョブ（`test-go` / `coverage-rust` / `coverage-ts` / `test-dart`）で
`codecov/codecov-action` の `fail_ci_if_error` を `true` に変更した。

**背景（M-16指摘）**: 従来はデフォルト値（`false`）のままであり、Codecov への
アップロードが失敗しても CI はパスし続けていた。サイレントなカバレッジ計測欠落を
検知できないため、アップロード失敗時は CI を失敗させる。

```yaml
# _test.yaml（test-go / coverage-rust / coverage-ts / test-dart 各ジョブ共通）
- name: Upload coverage
  uses: codecov/codecov-action@e28ff129e5465c2c0dcc6f003fc735cb6ae0c673 # v5
  with:
    # カバレッジアップロード失敗を CI エラーとして扱う（M-16対応）
    fail_ci_if_error: true
```

---

### security.yaml Trivy severity を MEDIUM,HIGH,CRITICAL に変更（M-23対応）

`security.yaml` の全 Trivy スキャンジョブ（`trivy-scan` / `image-scan` / `iac-scan` / `license-scan`）で
`severity` を `HIGH,CRITICAL` から `MEDIUM,HIGH,CRITICAL` に変更した。

**背景（M-23指摘）**: サプライチェーン攻撃の踏み台となる MEDIUM 脆弱性が見落とされていた。
MEDIUM 以上をスキャン対象に含めることで、潜在的な攻撃経路を早期に検出できる。

| 変更前 | 変更後 |
| --- | --- |
| `severity: HIGH,CRITICAL` | `severity: MEDIUM,HIGH,CRITICAL` |

```yaml
# security.yaml（全 Trivy スキャンジョブ共通）
- name: Trivy スキャン
  uses: aquasecurity/trivy-action@76071ef0d7ec797419534a183b498b4d6a132a02 # 0.29.0
  with:
    # MEDIUM も含めてスキャン: サプライチェーン攻撃は MEDIUM 脆弱性の悪用から始まることが多い（M-23対応）
    severity: MEDIUM,HIGH,CRITICAL
```

---

### Cosign identity-regexp を厳密化（C-06対応）

`deploy.yaml` および `_service-deploy.yaml` の全デプロイジョブで
`cosign verify` の `--certificate-identity-regexp` を厳密化した。

**背景（C-06指摘）**: 従来の正規表現 `github.com/k1s0-org/k1s0` は `^` / `$` アンカーがなく、
`evil-github.com/k1s0-org/k1s0-evil` のような文字列でもマッチしてしまう危険があった。
`^https://` で始まり `\\.github/workflows/.*` で終わる厳密なパターンに変更した。

| 変更前 | 変更後 |
| --- | --- |
| `--certificate-identity-regexp "github.com/k1s0-org/k1s0"` | `--certificate-identity-regexp "^https://github\\.com/k1s0-org/k1s0/\\.github/workflows/.*"` |

```yaml
# deploy.yaml / _service-deploy.yaml（deploy-dev / deploy-staging / deploy-prod 各ジョブ共通）
- name: Verify image signature
  run: |
    cosign verify \
      --certificate-oidc-issuer https://token.actions.githubusercontent.com \
      --certificate-identity-regexp "^https://github\\.com/k1s0-org/k1s0/\\.github/workflows/.*" \
      ${{ env.REGISTRY }}/...
```

---

## self-hosted Runner セキュリティガイドライン（H-11 監査対応）

k1s0 のデプロイジョブはクラスタ内で動作する self-hosted Runner を使用する（`runs-on: [self-hosted, dev]` 等）。self-hosted Runner はコードを直接実行するため、セキュリティ上の隔離が不十分な場合、シークレット漏洩・横移動・本番環境破壊のリスクがある。以下のガイドラインを遵守すること。

### 1. Ephemeral Runner の使用（最重要）

各ジョブ終了後に Runner を破棄する Ephemeral モードを使用する。

```bash
# Runner 登録時に --ephemeral フラグを指定する
# 各ジョブ完了後に Runner が自動的に登録解除・破棄される
./config.sh --url https://github.com/k1s0-org/k1s0 \
            --token <RUNNER_TOKEN> \
            --ephemeral \
            --labels "self-hosted,dev"
./run.sh
```

Ephemeral Runner を使用することで、あるジョブで発生した状態汚染（キャッシュ汚染・環境変数の残留等）が次のジョブに引き継がれることを防ぐ。Kubernetes 上では Actions Runner Controller（ARC）の `ephemeral: true` 設定を使用すること。

### 2. 専用ネットワークセグメントへの隔離

Runner は対応する Kubernetes Namespace（`k1s0-system` / `k1s0-business` 等）内に配置し、NetworkPolicy で通信範囲を制限する。

| Runner ラベル | 配置 Namespace | アクセス可能なリソース |
|--------------|--------------|----------------------|
| `self-hosted, dev` | `k1s0-system` (dev) | dev クラスタの Helm / kubectl |
| `self-hosted, staging` | `k1s0-system` (staging) | staging クラスタの Helm / kubectl |
| `self-hosted, prod` | `k1s0-system` (prod) | prod クラスタの Helm / kubectl（人手承認後のみ） |

Runner Pod の NetworkPolicy で、GitHub Actions の通信先（`api.github.com`・`*.actions.githubusercontent.com`）および Kubernetes API Server のみへの Egress を許可し、それ以外の外部通信は原則禁止とする。

### 3. 本番環境シークレットへのアクセス制限

環境別に Runner ラベルを分離し、CI ワークフローで環境ラベルを明示的に指定する。

```yaml
# deploy.yaml（本番デプロイジョブの例）
# prod ラベルの Runner のみが本番デプロイを実行できるよう制限する
deploy-prod:
  runs-on: [self-hosted, prod]
  environment:
    name: production
    # GitHub Environment Protection Rule で手動承認を必須とする
```

- `prod` ラベル Runner は本番専用の Kubernetes ServiceAccount トークンのみを保持し、dev / staging のシークレットには一切アクセスできない構成とすること
- GitHub Environments の Protection Rule（必須レビュアー・デプロイブランチ制限）を `production` 環境に設定し、承認なしの本番デプロイを防止すること

### 4. Runner の権限最小化

- Runner プロセスは専用の非 root ユーザーで実行すること（UID 1000 等）
- Docker socket（`/var/run/docker.sock`）のマウントはコンテナビルドが不要なジョブでは排除すること。ビルドが必要な場合は Docker-in-Docker（DinD）または Kaniko を使用すること
- Runner Pod の `securityContext` に `runAsNonRoot: true` と `allowPrivilegeEscalation: false` を設定すること

```yaml
# Runner Pod の最小権限設定例
# コンテナが root 権限を取得できないよう制限する
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  capabilities:
    drop:
      - ALL
```

### 5. インシデント対応: Runner 侵害時の即時オフライン化手順

Runner が侵害された疑いがある場合（不審なプロセス・予期しない外部通信・異常なシークレットアクセス等）は、以下の手順で即座にオフライン化する。

```bash
# ステップ1: GitHub 上で Runner をオフライン化する（WebUI または GitHub CLI）
# Settings → Actions → Runners → 対象 Runner → Remove Runner
gh api -X DELETE repos/k1s0-org/k1s0/actions/runners/<runner-id>

# ステップ2: Runner Pod をクラスタから即座に削除する
kubectl delete pod -n k1s0-system <runner-pod-name> --grace-period=0

# ステップ3: Runner が使用していた ServiceAccount トークンをローテーションする
kubectl delete secret -n k1s0-system <runner-sa-token-secret>

# ステップ4: 影響範囲を調査する（Runner が実行した直近のジョブログを確認）
gh run list --limit 20 --json databaseId,displayTitle,status,createdAt
```

侵害が確認された場合は、Runner が保持していた全シークレット（Kubernetes ServiceAccount トークン・Vault トークン等）を即座にローテーションし、セキュリティチームへ報告すること。

---

## E2E テストワークフロー（M-2 対応）

`tests/e2e/` 配下の Playwright テストを実行する専用ワークフロー。PR ごとの自動実行はリソースコストが高いため、**手動実行のみ**とする。

### ワークフロー概要

| 項目 | 設定 |
| --- | --- |
| ファイル | `.github/workflows/e2e.yaml` |
| トリガー | `workflow_dispatch`（手動実行のみ） |
| 実行時間 | 最大 30 分 |
| テストフレームワーク | Playwright（Chromium） |
| テストスペック | `tests/e2e/specs/`（5 スペック） |

### 実行手順

```bash
# GitHub Actions の Web UI から手動実行する
# または gh CLI で実行する
gh workflow run e2e.yaml
```

### E2E ワークフローの処理フロー

```
1. リポジトリチェックアウト
2. Node.js / pnpm セットアップ
3. Playwright 依存関係インストール（Chromium のみ）
4. Docker Compose infra プロファイル起動
   └── PostgreSQL, Redis, Kafka, Keycloak, Kong
5. Keycloak 起動待機（最大 120 秒）
6. Kong JWT 公開鍵セットアップ（setup-kong-jwt.sh）
7. bff-proxy 等のシステムサービス起動
8. Playwright E2E テスト実行
9. テスト結果（HTML レポート）を artifact に保存
10. Docker Compose クリーンアップ（volumes 含む）
```

### 今後の拡張計画

- **Phase 2**: `schedule` トリガーを追加して夜間自動実行（例: `cron: '0 1 * * *'`）
- **Phase 3**: 重要な PR（main へのマージ前）でのトリガー追加を検討
- **CI リソース改善**: より高速なランナーへの移行でテスト時間を短縮

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
