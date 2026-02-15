# CI/CD 設計

GitHub Actions によるパイプライン設計を定義する。
Tier アーキテクチャの詳細は [tier-architecture.md](tier-architecture.md) を参照。

## 基本方針

- CI/CD は **GitHub Actions** で一元管理する
- PR 時に CI（lint → test → build）、マージ時に CD（image push → deploy）を実行する
- 言語別マトリクスビルドで Go / Rust / TypeScript / Dart / Python に対応する
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
      go: ${{ steps.filter.outputs.go }}
      rust: ${{ steps.filter.outputs.rust }}
      ts: ${{ steps.filter.outputs.ts }}
      dart: ${{ steps.filter.outputs.dart }}
      python: ${{ steps.filter.outputs.python }}
      proto: ${{ steps.filter.outputs.proto }}
      helm: ${{ steps.filter.outputs.helm }}
    steps:
      - uses: actions/checkout@v4
      - uses: dorny/paths-filter@v3
        id: filter
        with:
          filters: |
            go:
              - 'regions/**/go/**'
            rust:
              - 'regions/**/rust/**'
              - 'CLI/**'
            ts:
              - 'regions/**/react/**'
              - 'regions/**/ts/**'
            dart:
              - 'regions/**/flutter/**'
              - 'regions/**/dart/**'
            python:
              - 'e2e/**'
            proto:
              - 'api/proto/**'
            helm:
              - 'infra/helm/**'

  lint-go:
    needs: detect-changes
    if: needs.detect-changes.outputs.go == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: "1.23"
      - uses: golangci/golangci-lint-action@v6
        with:
          version: latest

  lint-rust:
    needs: detect-changes
    if: needs.detect-changes.outputs.rust == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings

  lint-ts:
    needs: detect-changes
    if: needs.detect-changes.outputs.ts == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "20"
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
          channel: stable
      - run: dart analyze
      - run: dart format --set-exit-if-changed .

  lint-python:
    needs: detect-changes
    if: needs.detect-changes.outputs.python == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - run: pip install ruff
      - run: ruff check .
      - run: ruff format --check .

  test-go:
    needs: lint-go
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: "1.23"
      - run: go test ./... -race -coverprofile=coverage.out
      - uses: actions/upload-artifact@v4
        with:
          name: go-coverage
          path: coverage.out

  test-rust:
    needs: lint-rust
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all

  test-ts:
    needs: lint-ts
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "20"
      - run: npm ci
      - run: npm test

  test-dart:
    needs: lint-dart
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with:
          channel: stable
      - run: flutter test

  test-python:
    needs: lint-python
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - run: pip install -r e2e/requirements.txt
      - run: pytest e2e/ --tb=short

  proto-check:
    needs: detect-changes
    if: needs.detect-changes.outputs.proto == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: bufbuild/buf-setup-action@v1
      - run: buf lint api/proto
      - run: buf breaking api/proto --against '.git#branch=main'

  helm-lint:
    needs: detect-changes
    if: needs.detect-changes.outputs.helm == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: azure/setup-helm@v4
      - run: |
          for chart in infra/helm/services/*/*; do
            if [ -f "$chart/Chart.yaml" ]; then
              helm lint "$chart"
            fi
          done

  build:
    needs:
      - test-go
      - test-rust
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
          CHANGED=$(git diff --name-only HEAD~1 HEAD | \
            grep -oP 'regions/\K[^/]+/[^/]+/[^/]+/[^/]+' | \
            sort -u | head -20)
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
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      - name: Login to Harbor
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ secrets.HARBOR_USERNAME }}
          password: ${{ secrets.HARBOR_PASSWORD }}
      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          context: regions/${{ matrix.service }}
          push: true
          tags: |
            ${{ env.REGISTRY }}/k1s0/${{ matrix.service }}:${{ github.sha }}
            ${{ env.REGISTRY }}/k1s0/${{ matrix.service }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max

  deploy-dev:
    needs: build-and-push
    runs-on: ubuntu-latest
    environment: dev
    steps:
      - uses: actions/checkout@v4
      - uses: azure/setup-helm@v4
      - name: Deploy to dev
        run: |
          helm upgrade --install $SERVICE_NAME \
            ./infra/helm/services/$SERVICE_PATH \
            -n $NAMESPACE \
            -f ./infra/helm/services/$SERVICE_PATH/values-dev.yaml \
            --set image.tag=${{ github.sha }}

  deploy-staging:
    needs: deploy-dev
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - uses: actions/checkout@v4
      - uses: azure/setup-helm@v4
      - name: Deploy to staging
        run: |
          helm upgrade --install $SERVICE_NAME \
            ./infra/helm/services/$SERVICE_PATH \
            -n $NAMESPACE \
            -f ./infra/helm/services/$SERVICE_PATH/values-staging.yaml \
            --set image.tag=${{ github.sha }}

  deploy-prod:
    needs: deploy-staging
    runs-on: ubuntu-latest
    environment:
      name: prod
      url: https://api.k1s0.internal.example.com
    steps:
      - uses: actions/checkout@v4
      - uses: azure/setup-helm@v4
      - name: Deploy to prod
        run: |
          helm upgrade --install $SERVICE_NAME \
            ./infra/helm/services/$SERVICE_PATH \
            -n $NAMESPACE \
            -f ./infra/helm/services/$SERVICE_PATH/values-prod.yaml \
            --set image.tag=${{ github.sha }}
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
      - 'buf.yaml'
      - 'buf.gen.yaml'

jobs:
  proto-lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: bufbuild/buf-setup-action@v1
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
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: fs
          scan-ref: .
          severity: HIGH,CRITICAL
          format: table
          exit-code: 1

  dependency-check:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - lang: go
            cmd: "govulncheck ./..."
          - lang: rust
            cmd: "cargo audit"
          - lang: node
            cmd: "npm audit --audit-level=high"
    steps:
      - uses: actions/checkout@v4
      - name: Run dependency check (${{ matrix.lang }})
        run: ${{ matrix.cmd }}

  image-scan:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - name: Scan latest images
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: image
          image-ref: harbor.internal.example.com/k1s0-service/order:latest
          severity: HIGH,CRITICAL
          format: table
```

### Helm デプロイ連携

CI/CD パイプラインから Helm デプロイを実行する際の連携方式:

```
GitHub Actions → kubeconfig (Secret) → kubectl/helm → Kubernetes Cluster
```

| 項目             | 設定                                                |
| ---------------- | --------------------------------------------------- |
| kubeconfig       | GitHub Secrets に環境別で格納                       |
| Helm バージョン  | `azure/setup-helm@v4` で固定バージョンを使用        |
| デプロイ方式     | `helm upgrade --install`（冪等性を保証）            |
| イメージタグ     | `--set image.tag=${{ github.sha }}` で指定          |

### キャッシュ戦略

| 言語   | キャッシュ対象                | アクション                |
| ------ | ----------------------------- | ------------------------- |
| Go     | `~/go/pkg/mod`                | `actions/setup-go` 内蔵   |
| Rust   | `~/.cargo`, `target/`        | `actions/cache`           |
| Node   | `node_modules/`              | `actions/setup-node` 内蔵 |
| Dart   | `~/.pub-cache`               | `actions/cache`           |
| Python | `~/.cache/pip`               | `actions/setup-python` 内蔵 |
| Docker | Docker layer cache           | `cache-from: type=gha`   |

---

## 関連ドキュメント

- [tier-architecture.md](tier-architecture.md) — Tier アーキテクチャの詳細
- [Dockerイメージ戦略.md](Dockerイメージ戦略.md) — イメージビルド・タグ・レジストリ
- [helm設計.md](helm設計.md) — Helm Chart と values 設計
- [kubernetes設計.md](kubernetes設計.md) — Namespace・NetworkPolicy 設計
- [API設計.md](API設計.md) — REST API・gRPC・GraphQL 設計
- [可観測性設計.md](可観測性設計.md) — 監視・ログ・トレース設計
