# テンプレート仕様 — CI/CD

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **GitHub Actions ワークフローファイル** のテンプレート仕様を定義する。CI（継続的インテグレーション）と Deploy（継続的デプロイ）の2種類のワークフローを、サービスの `kind` と `language` に応じて自動生成する。

CI/CD 設計の全体像は [CI-CD設計](CI-CD設計.md) を参照。

## 生成対象

| kind       | CI ワークフロー | Deploy ワークフロー |
| ---------- | --------------- | ------------------- |
| `server`   | 生成する        | 生成する            |
| `bff`      | 生成する        | 生成する            |
| `client`   | 生成する        | 生成しない          |
| `library`  | 生成する        | 生成しない          |
| `database` | 生成する        | 生成しない          |

- **CI**: 全 kind で生成する。lint → test → build → security scan のパイプラインを構成する。
- **Deploy**: `server` および `bff` kind で生成する。Docker イメージのビルド・プッシュと Helm による環境別デプロイを構成する。

## 配置パス

生成されるワークフローファイルは `.github/workflows/` 直下に配置する。

| ファイル            | 配置パス                                         |
| ------------------- | ------------------------------------------------ |
| CI ワークフロー     | `.github/workflows/{{ service_name }}-ci.yaml`     |
| Deploy ワークフロー | `.github/workflows/{{ service_name }}-deploy.yaml` |

## テンプレートファイル一覧

テンプレートは `CLI/templates/cicd/` 配下に配置する。

| テンプレートファイル | 生成先                                           | 説明                              |
| -------------------- | ------------------------------------------------ | --------------------------------- |
| `ci.yaml.tera`       | `.github/workflows/{{ service_name }}-ci.yaml`     | CI ワークフロー                   |
| `deploy.yaml.tera`   | `.github/workflows/{{ service_name }}-deploy.yaml` | Deploy ワークフロー（server / bff）|
| `dependabot.yml.tera` | `.github/dependabot.yml`                          | Dependabot設定（言語別のパッケージエコシステム定義） |

### ディレクトリ構成

```
CLI/
└── templates/
    └── cicd/
        ├── ci.yaml.tera
        ├── deploy.yaml.tera
        └── dependabot.yml.tera
```

## 使用するテンプレート変数

CI/CD テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](テンプレートエンジン仕様.md) を参照。

| 変数名               | 型       | CI  | Deploy | 用途                                       |
| -------------------- | -------- | --- | ------ | ------------------------------------------ |
| `service_name`       | String   | 用  | 用     | ワークフロー名、paths フィルタ             |
| `service_name_snake`  | String   | 用  | —      | アーティファクト名                         |
| `module_path`        | String   | 用  | 用     | paths フィルタ（変更検出）                 |
| `language`           | String   | 用  | 用     | 言語別ステップの分岐（`"go"` / `"rust"` / `"typescript"` / `"dart"`） |
| `kind`               | String   | 用  | 用     | Deploy 生成判定、ビルドステップの分岐      |
| `tier`               | String   | —   | 用     | Docker プロジェクト名の導出                |
| `api_styles`         | [String] | 用  | —      | gRPC 時の buf lint ステップ追加等、API スタイル分岐 |
| `has_database`       | bool     | 用  | —      | DB マイグレーションテストステップの追加    |
| `database_type`      | String   | 用  | —      | DB 固有のマイグレーションツール選択        |
| `docker_registry`    | String   | —   | 用     | Docker レジストリ URL                      |
| `docker_project`     | String   | —   | 用     | Docker プロジェクト名                      |
| `framework`          | String   | 用  | —      | クライアント言語の判定（react / flutter）  |
| `go_module`          | String   | 用  | —      | Go テスト時のモジュールパス                |
| `rust_crate`         | String   | 用  | —      | Rust テスト時のクレート名                  |
| `helm_path`          | String   | —   | 用     | Helm Chart の Tier 別相対パス（helm upgrade パス） |

---

## GitHub Actions / Tera 構文衝突の回避

GitHub Actions は `${{ }}` 構文を使用し、Tera も `{{ }}` 構文を使用するため、テンプレート内で衝突が発生する。以下の方針で回避する。

### 回避方針

1. **GitHub Actions の式** (`${{ secrets.XXX }}`, `${{ github.ref }}` 等) は `{% raw %}` ... `{% endraw %}` ブロックで保護する
2. **Tera 変数** (`{{ service_name }}`, `{{ module_path }}` 等) は `{% raw %}` ブロックの **外** に配置する

### 構文の使い分け

| 構文                  | 処理主体       | 例                                    |
| --------------------- | -------------- | ------------------------------------- |
| `{{ service_name }}`  | Tera（生成時） | ワークフロー名、paths フィルタ        |
| `${{ github.ref }}`   | GitHub Actions | ランタイムの変数参照                  |
| `${{ secrets.XXX }}`  | GitHub Actions | シークレット参照                      |

### 記述例

```tera
name: {{ service_name }}-ci

on:
  pull_request:
    branches: [main]
    paths:
      - '{{ module_path }}/**'

{% raw %}
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
{% endraw %}

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
{% endraw %}
```

### ルール

- Tera 変数が必要な行は `{% raw %}` ブロックの外に記述する
- GitHub Actions の `${{ }}` が含まれる行は必ず `{% raw %}` ブロック内に記述する
- 1つのジョブ内で Tera 変数と GitHub Actions 変数が混在する場合は、`{% raw %}` / `{% endraw %}` を細かく区切る

---

## CI ワークフロー仕様（ci.yaml.tera）

### パイプライン構成

```
lint → test → build → security-scan (trivy)
```

全 kind（server, bff, client, library, database）で生成される。言語と kind に応じてステップが異なる。

### ワークフロー基本構造

```tera
name: {{ service_name }}-ci

on:
  pull_request:
    branches: [main]
    paths:
      - '{{ module_path }}/**'

{% raw %}
concurrency:
  group: ci-${{ github.ref }}-{{ service_name }}
  cancel-in-progress: true
{% endraw %}
```

- `paths` フィルタにより、該当サービスのファイル変更時のみ CI が実行される
- `concurrency` により同一ブランチの重複実行をキャンセルする

### 言語別ステップ

#### Rust

```tera
{% if language == "rust" %}
  lint:
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.82
        with:
          components: clippy, rustfmt
{% endraw %}
      - run: cargo fmt --all -- --check
        working-directory: {{ module_path }}
      - run: cargo clippy --all-targets -- -D warnings
        working-directory: {{ module_path }}

  test:
    needs: lint
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.82
{% endraw %}
      - run: cargo test --all
        working-directory: {{ module_path }}

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.82
{% endraw %}
      - run: cargo build --release
        working-directory: {{ module_path }}
{% endif %}
```

#### TypeScript (React)

```tera
{% if language == "typescript" or framework == "react" %}
  lint:
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "22"
{% endraw %}
      - run: npm ci
        working-directory: {{ module_path }}
      - run: npx eslint .
        working-directory: {{ module_path }}
      - run: npx prettier --check .
        working-directory: {{ module_path }}

  test:
    needs: lint
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "22"
{% endraw %}
      - run: npm ci
        working-directory: {{ module_path }}
      - run: npm test
        working-directory: {{ module_path }}

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "22"
{% endraw %}
      - run: npm ci
        working-directory: {{ module_path }}
      - run: npm run build
        working-directory: {{ module_path }}
{% endif %}
```

#### Dart / Flutter

```tera
{% if language == "dart" or framework == "flutter" %}
  lint:
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with:
          flutter-version: "3.24.0"
{% endraw %}
      - run: dart analyze
        working-directory: {{ module_path }}
      - run: dart format --set-exit-if-changed .
        working-directory: {{ module_path }}

  test:
    needs: lint
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with:
          flutter-version: "3.24.0"
{% endraw %}
      - run: flutter test
        working-directory: {{ module_path }}

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with:
          flutter-version: "3.24.0"
{% endraw %}
      - run: flutter build web
        working-directory: {{ module_path }}
{% endif %}
```

### 条件付きステップ

言語別ステップに加えて、以下の条件で追加ステップが挿入される。

#### gRPC 使用時（buf lint）

`api_styles` に `"grpc"` が含まれる場合、lint ジョブに buf lint ステップを追加する。

```tera
{% if api_styles is containing("grpc") %}
  proto-lint:
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - uses: bufbuild/buf-setup-action@v1
        with:
          version: "1.47.2"
{% endraw %}
      - name: Lint
        run: buf lint {{ module_path }}/api/proto
      - name: Breaking change detection
{% raw %}
        run: buf breaking {{ module_path }}/api/proto --against '.git#branch=main'
{% endraw %}
{% endif %}
```

#### DB 使用時（マイグレーションテスト）

`has_database == true` の場合、test ジョブに DB マイグレーションテストステップを追加する。

```tera
{% if has_database %}
  migration-test:
    runs-on: ubuntu-latest
{% if database_type == "postgresql" %}
    services:
      postgres:
{% raw %}
        image: postgres:16
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
{% endraw %}
{% elif database_type == "mysql" %}
    services:
      mysql:
{% raw %}
        image: mysql:8
        env:
          MYSQL_ROOT_PASSWORD: test
          MYSQL_DATABASE: test
        ports:
          - 3306:3306
        options: >-
          --health-cmd "mysqladmin ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
{% endraw %}
{% endif %}
    steps:
{% raw %}
      - uses: actions/checkout@v4
{% endraw %}
      - name: Run migrations
        run: |
          # マイグレーションファイルの適用テスト
          echo "Running migration test for {{ service_name }}"
{% endif %}
```

### セキュリティスキャン

全ての CI ワークフローに Trivy によるファイルシステムスキャンを含める。

```tera
  security-scan:
    needs: build
    runs-on: ubuntu-latest
    steps:
{% raw %}
      - uses: actions/checkout@v4
      - name: Trivy filesystem scan
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: fs
          scan-ref: {{ module_path }}
          severity: HIGH,CRITICAL
          exit-code: 1
{% endraw %}
```

---

## Dependabot 設定仕様（dependabot.yml.tera）

言語とフレームワークに応じて、Dependabot の自動依存関係更新設定を生成する。Docker と GitHub Actions のエコシステムは全 kind で共通で含まれる。

### テンプレート内容

```tera
version: 2
updates:
{% if language == "rust" %}
  - package-ecosystem: cargo
    directory: "/{{ module_path }}"
    schedule:
      interval: weekly
{% endif %}
{% if language == "typescript" or framework == "react" %}
  - package-ecosystem: npm
    directory: "/{{ module_path }}"
    schedule:
      interval: weekly
{% endif %}
{% if language == "dart" or framework == "flutter" %}
  - package-ecosystem: pub
    directory: "/{{ module_path }}"
    schedule:
      interval: weekly
{% endif %}
  - package-ecosystem: docker
    directory: "/{{ module_path }}"
    schedule:
      interval: weekly
  - package-ecosystem: github-actions
    directory: "/"
    schedule:
      interval: weekly
```

### エコシステム選択ルール

| 条件                                              | パッケージエコシステム |
| ------------------------------------------------- | ---------------------- |
| `language == "rust"`                              | `cargo`                |
| `language == "typescript"` or `framework == "react"` | `npm`                  |
| `language == "dart"` or `framework == "flutter"`  | `pub`                  |
| 共通（全 kind）                                   | `docker`               |
| 共通（全 kind）                                   | `github-actions`       |

---

## Deploy ワークフロー仕様（deploy.yaml.tera）

`kind == "server"` または `kind == "bff"` の場合に生成される。main ブランチへのマージ時に、Docker イメージのビルド・プッシュと Helm による段階的デプロイを実行する。

### デプロイパイプライン構成

```
push (main) → build-and-push → deploy-dev → deploy-staging → deploy-prod（承認ゲート付き）
```

### 環境別デプロイ戦略

| 環境    | トリガー         | 承認     | ランナー                 | ロールバック             |
| ------- | ---------------- | -------- | ------------------------ | ------------------------ |
| dev     | main マージ 自動 | 不要     | `[self-hosted, dev]`     | `helm rollback` 手動     |
| staging | dev 成功後 自動  | 不要     | `[self-hosted, staging]` | `helm rollback` 手動     |
| prod    | staging 成功後   | 手動承認 | `[self-hosted, prod]`    | `helm rollback` 即時実行 |

### GitHub Environments 設定

| Environment | Protection Rules                                 |
| ----------- | ------------------------------------------------ |
| dev         | なし                                             |
| staging     | なし                                             |
| prod        | Required reviewers（2名以上）+ Wait timer（5分） |

### ワークフロー基本構造

```tera
name: {{ service_name }}-deploy

on:
  push:
    branches: [main]
    paths:
      - '{{ module_path }}/**'

{% raw %}
env:
  REGISTRY: {{ docker_registry }}
{% endraw %}
```

### Docker ビルド & プッシュ

```tera
  build-and-push:
    runs-on: ubuntu-latest
    steps:
{% raw %}
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
          context: {{ module_path }}
          push: true
          tags: |
            ${{ env.REGISTRY }}/{{ docker_project }}/{{ service_name }}:${{ steps.version.outputs.value }}
            ${{ env.REGISTRY }}/{{ docker_project }}/{{ service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
            ${{ env.REGISTRY }}/{{ docker_project }}/{{ service_name }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
      - name: Install Cosign
        uses: sigstore/cosign-installer@v3
      - name: Sign image with Cosign
        run: |
          cosign sign --yes \
            ${{ env.REGISTRY }}/{{ docker_project }}/{{ service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
        env:
          COSIGN_EXPERIMENTAL: "1"
{% endraw %}
```

### Docker イメージ Trivy スキャン

build-and-push ジョブ内で、Docker イメージに対する Trivy 脆弱性スキャンを実施する。CI ワークフローのファイルシステムスキャンとは異なり、ビルド済みイメージ内のパッケージを検査する。

```tera
      - name: Trivy image scan
{% raw %}
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ env.REGISTRY }}/{{ docker_project }}/{{ service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
          severity: HIGH,CRITICAL
          exit-code: 1
{% endraw %}
```

### SBOM（Software Bill of Materials）生成

build-and-push ジョブ内で、ビルド済み Docker イメージから SBOM を生成する。SBOM はソフトウェアサプライチェーンの透明性を確保するために使用される。

```tera
      - name: Generate SBOM
{% raw %}
        uses: anchore/sbom-action@v0
        with:
          image: ${{ env.REGISTRY }}/{{ docker_project }}/{{ service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
          output-file: sbom.spdx.json
{% endraw %}
```

**イメージタグ規則**:

| タグ形式                  | 例                          | 用途             |
| ------------------------- | --------------------------- | ---------------- |
| `{version}`               | `1.2.3`                     | リリースバージョン |
| `{version}-{git-sha}`     | `1.2.3-abc1234`             | 一意識別（デプロイ用） |
| `latest`                  | `latest`                    | 最新ビルド       |

タグ規則の詳細は [Dockerイメージ戦略](Dockerイメージ戦略.md) を参照。

### Helm デプロイ（環境別）

各環境へのデプロイは self-hosted ランナー上で `helm upgrade --install` を実行する。環境固有の設定は `values-{env}.yaml` を参照する。

```tera
  deploy-dev:
    needs: build-and-push
    runs-on: [self-hosted, dev]
{% raw %}
    environment: dev
{% endraw %}
    steps:
{% raw %}
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
      - uses: sigstore/cosign-installer@v3
      - name: Verify image signature
        run: |
          cosign verify \
            --certificate-oidc-issuer https://token.actions.githubusercontent.com \
            --certificate-identity-regexp "github.com/k1s0-org/k1s0" \
            ${{ env.REGISTRY }}/{{ docker_project }}/{{ service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
      - uses: azure/setup-helm@v4
        with:
          version: "3.16"
      - name: Deploy to dev
        run: |
          helm upgrade --install {{ service_name }} \
            ./infra/helm/services/{{ helm_path }} \
            -n k1s0-{{ tier }} \
            -f ./infra/helm/services/{{ helm_path }}/values-dev.yaml \
            --set image.tag=${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
{% endraw %}

  deploy-staging:
    needs: deploy-dev
    runs-on: [self-hosted, staging]
{% raw %}
    environment: staging
{% endraw %}
    steps:
      # dev と同様の構成（values-staging.yaml を参照）
{% raw %}
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
      - uses: sigstore/cosign-installer@v3
      - name: Verify image signature
        run: |
          cosign verify \
            --certificate-oidc-issuer https://token.actions.githubusercontent.com \
            --certificate-identity-regexp "github.com/k1s0-org/k1s0" \
            ${{ env.REGISTRY }}/{{ docker_project }}/{{ service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
      - uses: azure/setup-helm@v4
        with:
          version: "3.16"
      - name: Deploy to staging
        run: |
          helm upgrade --install {{ service_name }} \
            ./infra/helm/services/{{ helm_path }} \
            -n k1s0-{{ tier }} \
            -f ./infra/helm/services/{{ helm_path }}/values-staging.yaml \
            --set image.tag=${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
{% endraw %}

  deploy-prod:
    needs: deploy-staging
    runs-on: [self-hosted, prod]
{% raw %}
    environment:
      name: prod
{% endraw %}
    steps:
      # staging と同様の構成（values-prod.yaml を参照、手動承認ゲート付き）
{% raw %}
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
      - uses: sigstore/cosign-installer@v3
      - name: Verify image signature
        run: |
          cosign verify \
            --certificate-oidc-issuer https://token.actions.githubusercontent.com \
            --certificate-identity-regexp "github.com/k1s0-org/k1s0" \
            ${{ env.REGISTRY }}/{{ docker_project }}/{{ service_name }}:${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
      - uses: azure/setup-helm@v4
        with:
          version: "3.16"
      - name: Deploy to prod
        run: |
          helm upgrade --install {{ service_name }} \
            ./infra/helm/services/{{ helm_path }} \
            -n k1s0-{{ tier }} \
            -f ./infra/helm/services/{{ helm_path }}/values-prod.yaml \
            --set image.tag=${{ steps.version.outputs.value }}-${{ steps.sha.outputs.short }}
{% endraw %}
```

**Helm デプロイのポイント**:

| 項目            | 設定                                                                        |
| --------------- | --------------------------------------------------------------------------- |
| ランナー        | 各環境のクラスタ内で動作する self-hosted ランナー                           |
| Helm バージョン | `azure/setup-helm@v4` で 3.16 を指定                                       |
| デプロイ方式    | `helm upgrade --install`（冪等性を保証）                                    |
| イメージタグ    | `--set image.tag=${VERSION}-${GITHUB_SHA::7}` で `{version}-{git-sha}` 形式 |
| 署名検証        | Cosign によるイメージ署名検証をデプロイ前に実施                             |

Helm Chart の詳細は [helm設計](helm設計.md) を参照。

---

## 言語バージョン

CI/CD ワークフローで使用する言語・ツールのバージョンを以下に示す。バージョンは [CI-CD設計](CI-CD設計.md) および [devcontainer設計](devcontainer設計.md) と同期する。

| 言語/ツール | バージョン | セットアップ Action              |
| ----------- | ---------- | -------------------------------- |
| Go          | 1.23       | `actions/setup-go@v5`            |
| Rust        | 1.82       | `dtolnay/rust-toolchain@1.82`    |
| Node.js     | 22         | `actions/setup-node@v4`          |
| Dart        | 3.5        | `subosito/flutter-action@v2`     |
| Flutter     | 3.24.0     | `subosito/flutter-action@v2`     |
| Helm        | 3.16       | `azure/setup-helm@v4`            |
| buf         | 1.47.2     | `bufbuild/buf-setup-action@v1`   |

---

## キャッシュ戦略

CI の実行時間を短縮するため、言語ごとのキャッシュを活用する。

| 言語   | キャッシュ対象              | アクション                |
| ------ | --------------------------- | ------------------------- |
| Go     | `~/go/pkg/mod`             | `actions/cache`           |
| Rust   | `~/.cargo`, `target/`      | `actions/cache`           |
| Node   | `node_modules/`            | `actions/setup-node` 内蔵 |
| Dart   | `~/.pub-cache`             | `actions/cache`           |
| Docker | Docker layer cache         | `cache-from: type=gha`   |

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、ワークフロー内のステップが変わる。

| 条件                     | 選択肢                            | CI への影響                                       |
| ------------------------ | --------------------------------- | ------------------------------------------------- |
| 言語 (`language`)        | `go`                              | Go 固有の lint → test → build ステップ            |
| 言語 (`language`)        | `rust`                            | 言語固有の lint → test → build ステップ           |
| フレームワーク (`framework`) | `react` / `flutter`              | クライアント固有の lint → test → build ステップ   |
| API 方式 (`api_styles`)  | `grpc` を含む                     | buf lint + breaking change detection ステップ追加 |
| DB 有無 (`has_database`) | `true`                            | DB マイグレーションテストステップ追加             |
| DB 種別 (`database_type`)| `postgresql` / `mysql` / `sqlite` | service コンテナの種別選択                        |
| kind (`kind`)            | `server`                          | Deploy ワークフローの生成                         |
| kind (`kind`)            | `bff`                             | Deploy ワークフローの生成                         |

---

## 生成例

### Go REST サーバー（DB あり）の場合

入力:
```json
{
  "service_name": "order-api",
  "module_path": "regions/service/order/server/go",
  "language": "go",
  "kind": "server",
  "tier": "service",
  "api_styles": ["rest"],
  "has_database": true,
  "database_type": "postgresql",
  "docker_registry": "harbor.internal.example.com",
  "docker_project": "k1s0-service"
}
```

生成されるファイル:
- `.github/workflows/order-api-ci.yaml` — lint (golangci-lint) → migration-test → test → build → security-scan
- `.github/workflows/order-api-deploy.yaml` — build-and-push → deploy-dev → deploy-staging → deploy-prod

### Rust gRPC サーバー（DB なし）の場合

入力:
```json
{
  "service_name": "auth-service",
  "module_path": "regions/system/server/rust/auth-service",
  "language": "rust",
  "kind": "server",
  "tier": "system",
  "api_styles": ["grpc"],
  "has_database": false,
  "docker_registry": "harbor.internal.example.com",
  "docker_project": "k1s0-system"
}
```

生成されるファイル:
- `.github/workflows/auth-service-ci.yaml` — lint (clippy + rustfmt) → proto-lint (buf) → test → build → security-scan
- `.github/workflows/auth-service-deploy.yaml` — build-and-push → deploy-dev → deploy-staging → deploy-prod

### Go BFF（GraphQL）の場合

入力:
```json
{
  "service_name": "order-bff",
  "module_path": "regions/service/order/bff/go",
  "language": "go",
  "kind": "bff",
  "tier": "service",
  "api_styles": ["graphql"],
  "has_database": false,
  "docker_registry": "harbor.internal.example.com",
  "docker_project": "k1s0-service"
}
```

生成されるファイル:
- `.github/workflows/order-bff-ci.yaml` — lint (golangci-lint) → test → build → security-scan
- `.github/workflows/order-bff-deploy.yaml` — build-and-push (Trivy image scan + SBOM 生成含む) → deploy-dev → deploy-staging → deploy-prod
- `.github/dependabot.yml` — gomod + docker + github-actions エコシステム

### React クライアント（Deploy なし）の場合

入力:
```json
{
  "service_name": "ledger-app",
  "module_path": "regions/business/accounting/client/react/ledger-app",
  "language": "typescript",
  "kind": "client",
  "framework": "react"
}
```

生成されるファイル:
- `.github/workflows/ledger-app-ci.yaml` — lint (eslint + prettier) → test → build → security-scan
- Deploy ワークフローは **生成されない**（kind が client のため）

---

## 関連ドキュメント

- [CI-CD設計](CI-CD設計.md) — CI/CD パイプラインの全体設計
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) — テンプレート変数・条件分岐・フィルタの仕様
- [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) — サーバーテンプレート仕様
- [テンプレート仕様-BFF](テンプレート仕様-BFF.md) — BFF テンプレート仕様
- [テンプレート仕様-クライアント](テンプレート仕様-クライアント.md) — クライアントテンプレート仕様
- [テンプレート仕様-ライブラリ](テンプレート仕様-ライブラリ.md) — ライブラリテンプレート仕様
- [テンプレート仕様-データベース](テンプレート仕様-データベース.md) — データベーステンプレート仕様
- [Dockerイメージ戦略](Dockerイメージ戦略.md) — Docker ビルド・タグ・レジストリ
- [helm設計](helm設計.md) — Helm Chart と values 設計
- [devcontainer設計](devcontainer設計.md) — 言語バージョンの同期元
- [コーディング規約](コーディング規約.md) — Linter・Formatter 設定
