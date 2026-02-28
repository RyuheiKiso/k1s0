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
| Rust        | `rust:1.82-bookworm`         | `gcr.io/distroless/cc-debian12`           |
| React       | `node:22-bookworm` (ビルド)  | `nginx:1.27-alpine`（静的配信）           |
| Flutter Web | `ghcr.io/cirruslabs/flutter:3.24.0` (ビルド) | `nginx:1.27-alpine`（静的配信）  |

## Dockerfile テンプレート

### Rust サーバー

```dockerfile
# ---- Build ----
FROM rust:1.82-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN cargo build --release

# ---- Runtime ----
FROM gcr.io/distroless/cc-debian12
COPY --from=build /src/target/release/app /app
# config.yaml は Kubernetes 環境では ConfigMap としてマウントされる（helm設計.md 参照）
# ローカル実行時は -v オプションで config/ をマウントすること
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
harbor.internal.example.com/k1s0-business/accounting-ledger:1.0.0
harbor.internal.example.com/k1s0-service/order:1.0.0
harbor.internal.example.com/k1s0-service/order-client:1.0.0
```

### 命名規則

```
harbor.internal.example.com/{プロジェクト}/{サービス名}:{タグ}
```

| プロジェクト    | 対象                            |
| --------------- | ------------------------------- |
| k1s0-system     | system 層のサーバー             |
| k1s0-business   | business 層のサーバー・クライアント（`{領域名}-{サービス名}`） |
| k1s0-service    | service 層のサーバー・クライアント |
| k1s0-infra      | カスタムインフライメージ        |

## セキュリティ

| 対策                       | 方法                                         |
| -------------------------- | -------------------------------------------- |
| 脆弱性スキャン             | Harbor 組み込みの Trivy で push 時に自動実行 |
| 非 root 実行               | distroless の `nonroot` ユーザーを使用       |
| 不要ツール排除             | distroless / Alpine で攻撃面を最小化         |
| イメージ署名               | Cosign でイメージに署名                      |
| プル制限                   | Harbor のロボットアカウントで認証必須         |

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

- [CI-CD設計](../cicd/CI-CD設計.md)
- [helm設計](../kubernetes/helm設計.md)
- [devcontainer設計](../devenv/devcontainer設計.md)
- [インフラ設計](../overview/インフラ設計.md)
- [API設計](../../architecture/api/API設計.md)
- [テンプレート仕様-サーバー](../../templates/server/サーバー.md) — サーバー Dockerfile テンプレート
- [テンプレート仕様-クライアント](../../templates/client/クライアント.md) — クライアント Dockerfile テンプレート
