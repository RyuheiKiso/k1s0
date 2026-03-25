# Docker イメージ戦略

k1s0 プロジェクトにおける Docker イメージのビルド・管理方針を定義する。

## 基本方針

- マルチステージビルドで最小限のランタイムイメージを生成する
- ベースイメージは distroless または Alpine を使用する
- イメージタグはセマンティックバージョニング + Git SHA で管理する
- Harbor の脆弱性スキャン（Trivy）を全イメージに適用する

## ベースイメージ

| 言語 / FW   | ビルドステージ               | ランタイムステージ                        |
| ----------- | ---------------------------- | ----------------------------------------- |
| Rust        | `rust:1.93-bookworm`         | `gcr.io/distroless/cc-debian12:nonroot`   |
| Go BFF      | `golang:1.24-alpine`         | `gcr.io/distroless/static-debian12`       |
| React       | `node:22-bookworm` (ビルド)  | `nginx:1.27-alpine`（静的配信）           |
| Flutter Web | `ghcr.io/cirruslabs/flutter:3.24.0` (ビルド) | `nginx:1.27-alpine`（静的配信）  |
| Flutter Desktop | `ghcr.io/cirruslabs/flutter:3.24.0` (ビルド) | ネイティブバイナリ（exe / AppImage / dmg）— App Registry Server から直接配信（[アプリ配布基盤設計](../distribution/アプリ配布基盤設計.md) 参照） |

## Dockerfile テンプレート

### Rust サーバー

```dockerfile
# ---- Build ----
FROM rust:1.93-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN cargo build --release

# ---- Runtime ----
# 攻撃面の最小化のため distroless イメージを採用
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /src/target/release/app /app
# config.yaml は Kubernetes 環境では ConfigMap としてマウントされる（helm設計.md 参照）
# ローカル実行時は -v オプションで config/ をマウントすること
EXPOSE 8080 50051
ENTRYPOINT ["/app"]
```

### Go BFF サーバー

```dockerfile
# ---- Build ----
FROM golang:1.24-alpine AS build
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /app .

# ---- Runtime ----
FROM gcr.io/distroless/static-debian12
COPY --from=build /app /app
USER nonroot:nonroot
EXPOSE 8080
ENTRYPOINT ["/app"]
```

### React クライアント

```dockerfile
# ---- Build ----
FROM node:22-bookworm AS build
WORKDIR /src
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

# ---- Runtime ----
FROM nginx:1.27-alpine
COPY --from=build /src/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
# nginx のデフォルトユーザーは root のため、非 root 実行に切り替える。
# helm設計.md の securityContext（runAsUser: 65532）を使用する場合は
# Dockerfile 側で該当 UID のユーザーを作成し、nginx が listen する
# ポートを 1024 以上に変更する必要がある。
# 簡易的な非 root 化として USER nginx を使用する場合は、
# helm 側の runAsUser を nginx ユーザーの UID（101）に合わせること。
USER nginx
EXPOSE 8080
```

### Flutter Web クライアント

```dockerfile
# ---- Build Stage ----
FROM ghcr.io/cirruslabs/flutter:3.24.0 AS build
WORKDIR /app
COPY pubspec.* ./
RUN flutter pub get
COPY . .
RUN flutter build web --release

# ---- Runtime Stage ----
FROM nginx:1.27-alpine
COPY --from=build /app/build/web /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
# nginx のデフォルトユーザーは root のため、非 root 実行に切り替える。
# helm設計.md の securityContext との整合については React クライアントと同様。
USER nginx
EXPOSE 8080
```

## イメージタグ規則

| タグ形式                          | 用途                             | 例                            |
| --------------------------------- | -------------------------------- | ----------------------------- |
| `{version}`                       | リリースバージョン               | `1.2.3`                       |
| `{version}-{git-sha}`             | ビルド特定用                     | `1.2.3-a1b2c3d`               |
| `latest`                          | 最新の安定版                     | `latest`                      |
| `{branch}-{git-sha}`              | 開発ブランチのビルド             | `feature-auth-a1b2c3d`        |

### タグ付けルール

- `main` ブランチへのマージ時にバージョンタグ + `latest` を付与する
- feature ブランチのビルドはブランチ名 + SHA で管理し、マージ後に削除する
- Git SHA は先頭 7 文字を使用する

## レジストリ構成

イメージは Harbor のプロジェクトに階層ごとに格納する。

```
harbor.internal.example.com/k1s0-system/auth:1.0.0
harbor.internal.example.com/k1s0-system/gateway:1.0.0
harbor.internal.example.com/k1s0-business/taskmanagement-project-master:1.0.0
harbor.internal.example.com/k1s0-service/task:1.0.0
harbor.internal.example.com/k1s0-service/task-client:1.0.0
```

### 命名規則

```
harbor.internal.example.com/{プロジェクト}/{サービス名}:{タグ}
```

| プロジェクト    | 対象                            |
| --------------- | ------------------------------- |
| k1s0-system     | system tier のサーバー             |
| k1s0-business   | business tier のサーバー・クライアント（`{領域名}-{サービス名}`） |
| k1s0-service    | service tier のサーバー・クライアント |
| k1s0-infra      | カスタムインフライメージ        |

## ビルドコンテキスト最適化（CRIT-02 対応）

### 問題

`regions/system/server/rust/` の全 Rust サービス（27 サービス）をビルドする際、ビルドコンテキストに全サービスのソースコードが含まれると転送量が 3 GB を超える。

### 方針

`.dockerignore` の二層構成で不要ファイルをコンテキストから除外する。

| ファイル | 役割 |
| --- | --- |
| リポジトリルート `.dockerignore` | `graphql-gateway` のリポジトリルートビルド時に適用 |
| `regions/system/.dockerignore` | system tier サービス（`context: ./regions/system`）のビルド時に適用 |

### リポジトリルート `.dockerignore` 除外項目

| 除外パス | 理由 |
| --- | --- |
| `CLI/` | Rust CLI ソースはサーバービルドに不要 |
| `.github/` | GitHub Actions 定義はコンテナ内に不要 |
| `scripts/` | ビルドスクリプトはコンテナ内に不要 |
| `tasks/` | タスク管理ファイルはコンテナ内に不要 |
| `docs/` | ドキュメントはコンテナ内に不要 |

**注意:** `graphql-gateway` は `api/proto` を必要とするため、`api/` ディレクトリは除外してはならない。

### `regions/system/.dockerignore` 除外項目

| 除外パス | 理由 |
| --- | --- |
| `**/tests/` | 統合テストコードはランタイムイメージに不要（ソースの `src/tests/` は除外対象外） |

### ADR 参照

Dockerfile テンプレート戦略については [ADR-0035](../../architecture/adr/0035-dockerfile-template-strategy.md) を参照。

## ベースイメージ ダイジェスト固定（H-05 対応）

### 目的

タグ（`rust:1.93-bookworm` 等）は可変であり、同じタグでも異なるレイヤーに書き換えられる可能性がある。
ダイジェスト（`@sha256:...`）を固定することでサプライチェーン攻撃を防ぎ、ビルドの再現性を保証する。

### 固定済みダイジェスト一覧（2026-03-23 時点）

| ベースイメージ                          | ダイジェスト                                                                 |
| --------------------------------------- | ---------------------------------------------------------------------------- |
| `rust:1.93-bookworm`                    | `rust@sha256:7c4ae649a84014c467d79319bbf17ce2632ae8b8be123ac2fb2ea5be46823f31` |
| `busybox:1.36.1-musl`                   | `busybox@sha256:3c6ae8008e2c2eedd141725c30b20d9c36b026eb796688f88205845ef17aa213` |
| `gcr.io/distroless/cc-debian12:nonroot` | `gcr.io/distroless/cc-debian12@sha256:7e5b8df2f4d36f5599ef4ab856d7d444922531709becb03f3368c6d797d0a5eb` |
| `gcr.io/distroless/static-debian12:nonroot` | `gcr.io/distroless/static-debian12@sha256:a9329520abc449e3b14d5bc3a6ffae065bdde0f02667fa10880c49b35c109fd1` |
| `gcr.io/distroless/static-debian12`     | `gcr.io/distroless/static-debian12@sha256:20bc6c0bc4d625a22a8fde3e55f6515709b32055ef8fb9cfbddaa06d1760f838` |
| `gcr.io/distroless/cc:nonroot`          | `gcr.io/distroless/cc@sha256:9c4fe2381c2e6d53c4cfdefeff6edbd2a67ec7713e2c3ca6653806cbdbf27a1e` |
| `gcr.io/distroless/static:nonroot`      | `gcr.io/distroless/static@sha256:e3f945647ffb95b5839c07038d64f9811adf17308b9121d8a2b87b6a22a80a39` |
| `golang:1.26-alpine`                    | `golang@sha256:2389ebfa5b7f43eeafbd6be0c3700cc46690ef842ad962f6c5bd6be49ed82039` |
| `golang:1.24-bookworm`                  | `golang@sha256:1a6d4452c65dea36aac2e2d606b01b4a029ec90cc1ae53890540ce6173ea77ac` |
| `node:22-alpine`                        | `node@sha256:8094c002d08262dba12645a3b4a15cd6cd627d30bc782f53229a2ec13ee22a00` |
| `node:22-bookworm`                      | `node@sha256:f90672bf4c76dfc077d17be4c115b1ae7731d2e8558b457d86bca42aeb193866` |
| `nginx:alpine`                          | `nginx@sha256:f46cb72c7df02710e693e863a983ac42f6a9579058a59a35f1ae36c9958e4ce0` |
| `nginx:1.27-alpine`                     | `nginx@sha256:65645c7bb6a0661892a8b03b89d0743208a18dd2f3f17a54ef4b76fb8e2f2a10` |
| `ghcr.io/cirruslabs/flutter:3.24.0`     | `ghcr.io/cirruslabs/flutter@sha256:eeef49aa71066f71c5b53962ff957f6edd84949da6496ea432f7f455db220b08` |
| `alpine:3.19`                           | `alpine@sha256:6baf43584bcb78f2e5847d1de515f23499913ac9f12bdf834811a3145eb11ca1` |

### ダイジェスト更新手順

ベースイメージを更新する際は以下のスクリプトを実行してダイジェストを再固定する。

```bash
# 全 Dockerfile を更新（Docker CLI が必要）
bash scripts/pin-docker-digests.sh

# 変更内容をプレビューのみ（ファイル変更なし）
bash scripts/pin-docker-digests.sh --dry-run
```

### CI/CD での自動更新（MED-06 対応）

ダイジェスト固定の自動更新は `.github/workflows/pin-docker-digests.yaml` で実装済みである。

| 項目 | 内容 |
| --- | --- |
| 実行頻度 | 毎週月曜日 09:00 UTC（日本時間 18:00） |
| 実行内容 | `scripts/pin-docker-digests.sh` でダイジェストを更新し、変更があれば PR を自動作成 |
| ブランチ | `chore/pin-docker-digests` |
| ラベル | `security`, `automated` |
| 手動実行 | `workflow_dispatch` イベントで任意タイミングに実行可能 |

全 Dockerfile の TODO コメントは以下の形式に統一されている。

```dockerfile
# ダイジェスト固定は .github/workflows/pin-docker-digests.yaml で自動更新される
```

## セキュリティ

| 対策                       | 方法                                         |
| -------------------------- | -------------------------------------------- |
| 脆弱性スキャン             | Harbor 組み込みの Trivy で push 時に自動実行 |
| 非 root 実行               | distroless の `nonroot` ユーザーを使用       |
| 不要ツール排除             | distroless / Alpine で攻撃面を最小化         |
| イメージ署名               | Cosign でイメージに署名                      |
| プル制限                   | Harbor のロボットアカウントで認証必須         |
| ベースイメージ固定         | `@sha256:` ダイジェスト固定（H-05 対応）     |

### Cosign イメージ署名（CI/CD 組み込み）

イメージ署名は CI/CD パイプラインに組み込み、ビルド・プッシュ後に自動で署名を行う。

#### 署名方式

- GitHub Actions の OIDC トークンを使用した **keyless signing** を採用する
- 署名鍵の管理が不要となり、運用コストを低減する
- Sigstore の Fulcio（証明書発行）と Rekor（透明性ログ）を利用する

#### CI/CD での署名フロー

```yaml
# CI/CD の build ジョブ完了後に実行
- name: Sign image with Cosign
  run: |
    cosign sign --yes \
      harbor.internal.example.com/k1s0-system/auth:${VERSION}
  env:
    COSIGN_EXPERIMENTAL: "1"  # keyless signing を有効化
```

#### デプロイ前の署名検証

```bash
# デプロイ前に署名検証を実施
cosign verify \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity-regexp "github.com/k1s0-org/k1s0" \
  harbor.internal.example.com/k1s0-system/auth:${VERSION}
```

#### 運用ルール

- CI/CD の build ジョブ完了後に `cosign sign` を必ず実行する
- デプロイパイプラインでは `cosign verify` による署名検証を通過したイメージのみデプロイする
- Kubernetes 側では Kyverno または OPA Gatekeeper で署名検証ポリシーを適用する

### 脆弱性ポリシー

| 深刻度     | ポリシー                                   |
| ---------- | ------------------------------------------ |
| Critical   | push をブロック。即時修正が必要            |
| High       | push をブロック。1 週間以内に修正          |
| Medium     | 警告のみ。次回リリースまでに修正           |
| Low        | 記録のみ                                   |

## イメージ保持ポリシー

| 対象              | 保持ルール                    |
| ----------------- | ----------------------------- |
| リリースタグ      | 直近 10 バージョン            |
| latest            | 常に 1 つ                     |
| ブランチビルド    | マージ後 7 日で自動削除       |

Harbor のタグ保持ポリシーで自動管理する。

## マルチアーキテクチャ対応方針

### 現時点の方針

- **amd64（x86_64）のみ**をビルドターゲットとする
- オンプレミス環境のサーバーがすべて x86_64 アーキテクチャであるため、ARM64 ビルドは現時点では不要

### 将来の ARM64 対応への備え

- Dockerfile では `--platform` 引数を受け付ける構造にしておく
- ベースイメージはマルチアーキテクチャ対応のものを選定済み（distroless, Alpine, nginx 等）
- `docker buildx` によるマルチプラットフォームビルドは、ARM64 環境への対応需要が発生した段階で有効化する

### マルチプラットフォームビルド（将来有効化時）

```bash
# docker buildx によるマルチプラットフォームビルド例
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t harbor.internal.example.com/k1s0-system/auth:1.0.0 \
  --push .
```

## 関連ドキュメント

- [ADR-0035: Dockerfile テンプレート戦略](../../architecture/adr/0035-dockerfile-template-strategy.md) — 27 サービス個別 Dockerfile 維持の決定理由
- [CI-CD設計](../cicd/CI-CD設計.md)
- [helm設計](../kubernetes/helm設計.md)
- [devcontainer設計](../devenv/devcontainer設計.md)
- [インフラ設計](../overview/インフラ設計.md)
- [API設計](../../architecture/api/API設計.md)
- [テンプレート仕様-サーバー](../../templates/server/サーバー.md) — サーバー Dockerfile テンプレート
- [テンプレート仕様-クライアント](../../templates/client/クライアント.md) — クライアント Dockerfile テンプレート
- [アプリ配布基盤設計](../distribution/アプリ配布基盤設計.md) — デスクトップアプリバイナリの配布
