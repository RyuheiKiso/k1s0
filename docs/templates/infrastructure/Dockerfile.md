# テンプレート仕様 — Dockerfile

## 概要

k1s0 CLI ひな形生成のDockerfileテンプレート仕様。server/rust、bff/go、bff/rust、client/react、client/flutter の5種類の Dockerfile テンプレートを提供し、各言語・フレームワークに最適化されたマルチステージビルドを自動生成する。

Docker イメージ戦略の全体像は [Dockerイメージ戦略](../../infrastructure/docker/Dockerイメージ戦略.md) を参照。

## テンプレートファイル一覧

テンプレートは各 kind/lang ディレクトリに配置する。

| テンプレートファイル              | 生成先         | 説明                          |
| --------------------------------- | -------------- | ----------------------------- |
| `server/rust/Dockerfile.tera`     | `Dockerfile`   | Rust サーバー用 Dockerfile    |
| `bff/go/Dockerfile.tera`          | `Dockerfile`   | Go BFF 用 Dockerfile          |
| `bff/rust/Dockerfile.tera`        | `Dockerfile`   | Rust BFF 用 Dockerfile        |
| `client/react/Dockerfile.tera`    | `Dockerfile`   | React クライアント用 Dockerfile |
| `client/flutter/Dockerfile.tera`  | `Dockerfile`   | Flutter クライアント用 Dockerfile |

### ディレクトリ構成

```
CLI/
└── templates/
    ├── server/
    │   └── rust/
    │       └── Dockerfile.tera
    ├── bff/
    │   ├── go/
    │   │   └── Dockerfile.tera
    │   └── rust/
    │       └── Dockerfile.tera
    └── client/
        ├── react/
        │   └── Dockerfile.tera
        └── flutter/
            └── Dockerfile.tera
```

## 使用するテンプレート変数

Dockerfile テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名         | 型     | server/rust | bff/go | bff/rust | client/react | client/flutter | 用途                 |
| -------------- | ------ | ----------- | ------ | -------- | ------------ | -------------- | -------------------- |
| `service_name` | String | 用          | 用     | 用       | -            | -              | バイナリ名・COPY 先  |

## 共通方針

全 Dockerfile に共通する設計方針を以下に示す。

| 方針                       | 説明                                                         |
| -------------------------- | ------------------------------------------------------------ |
| マルチステージビルド       | ビルドステージとランタイムステージを分離し、最終イメージサイズを最小化 |
| distroless / nginx         | サーバー系は distroless、クライアント系は nginx をランタイムとして使用 |
| nonroot ユーザー           | サーバー系は nonroot ユーザーで実行し、特権昇格を防止        |
| レイヤーキャッシュ最適化   | 依存関係のダウンロードを先に実行し、ビルドキャッシュを活用   |

---

## server/rust Dockerfile テンプレート

Rust サーバー用のマルチステージビルド Dockerfile。

```tera
# Build stage
FROM rust:1.82 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY . .
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /app/target/release/{{ service_name }} /{{ service_name }}

USER nonroot:nonroot

ENTRYPOINT ["/{{ service_name }}"]
```

### ポイント

- `rust:1.82` をビルドステージのベースイメージとして使用する
- ダミーの `main.rs` で事前に依存関係をビルドし、キャッシュを活用する
- `--release` フラグで最適化ビルドを行う
- ランタイムは `distroless/cc-debian12:nonroot` を使用する（Rust バイナリは `libc` が必要なため `cc` バリアントを使用）
- `USER nonroot:nonroot` で非特権ユーザーとして実行する

---

## bff/go Dockerfile テンプレート

Go BFF 用のマルチステージビルド Dockerfile。

```tera
# Build stage
FROM golang:1.23-alpine AS builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o /{{ service_name }} ./cmd/bff

# Runtime stage
FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=builder /{{ service_name }} /{{ service_name }}

USER nonroot:nonroot

ENTRYPOINT ["/{{ service_name }}"]
```

### ポイント

- `golang:1.23-alpine` をビルドステージのベースイメージとして使用する（BFF は alpine ベースで軽量化）
- `CGO_ENABLED=0` で静的バイナリを生成する
- ランタイムは `distroless/static-debian12:nonroot` を使用する
- ビルドターゲットは `./cmd/bff` ディレクトリを指定する

---

## bff/rust Dockerfile テンプレート

Rust BFF 用のマルチステージビルド Dockerfile。

```tera
# Build stage
FROM rust:1.82 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY . .
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /app/target/release/{{ service_name }} /{{ service_name }}

USER nonroot:nonroot

ENTRYPOINT ["/{{ service_name }}"]
```

### ポイント

- `rust:1.82` をビルドステージのベースイメージとして使用する
- ダミーの `main.rs` で依存関係を事前ビルドし、キャッシュを活用する
- ランタイムは `distroless/cc-debian12:nonroot` を使用する
- server/rust と同様の構成だが、ビルドターゲットが BFF 用に異なる

---

## client/react Dockerfile テンプレート

React クライアント用のマルチステージビルド Dockerfile。

```tera
# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

# Runtime stage
FROM nginx:1.27-alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

### ポイント

- `node:20-alpine` をビルドステージのベースイメージとして使用する
- `npm ci` で再現性のある依存関係インストールを行う
- ビルド成果物を `nginx:1.27-alpine` のドキュメントルートにコピーする
- カスタム `nginx.conf` で SPA のルーティング（fallback）を設定する

---

## client/flutter Dockerfile テンプレート

Flutter クライアント用のマルチステージビルド Dockerfile。

```tera
# Build stage
FROM ghcr.io/cirruslabs/flutter:stable AS builder

WORKDIR /app

COPY pubspec.yaml pubspec.lock ./
RUN flutter pub get

COPY . .
RUN flutter build web --release

# Runtime stage
FROM nginx:1.27-alpine

COPY --from=builder /app/build/web /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

### ポイント

- `ghcr.io/cirruslabs/flutter:stable` をビルドステージのベースイメージとして使用する
- `flutter pub get` で依存関係を先にダウンロードし、キャッシュを活用する
- `flutter build web --release` で最適化された Web ビルドを行う
- ランタイムは React と同様に `nginx:1.27-alpine` を使用する

---

## イメージサイズ比較

各 Dockerfile で生成されるイメージの概算サイズを以下に示す。

| kind/lang        | ベースイメージ                    | ランタイムイメージ                     | 概算サイズ |
| ---------------- | --------------------------------- | -------------------------------------- | ---------- |
| server/rust      | rust:1.82                         | distroless/cc-debian12:nonroot         | ~15MB      |
| bff/go           | golang:1.22-alpine                | distroless/static-debian12:nonroot     | ~10MB      |
| bff/rust         | rust:1.82                         | distroless/cc-debian12:nonroot         | ~15MB      |
| client/react     | node:20-alpine                    | nginx:1.27-alpine                      | ~25MB      |
| client/flutter   | cirruslabs/flutter:stable         | nginx:1.27-alpine                      | ~25MB      |

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [Dockerイメージ戦略](../../infrastructure/docker/Dockerイメージ戦略.md) -- Docker イメージの設計方針・ベースイメージ選定
- [テンプレート仕様-DockerCompose](DockerCompose.md) -- Docker Compose テンプレート仕様
- [テンプレート仕様-CICD](CICD.md) -- CI/CD テンプレート仕様
- [テンプレート仕様-Config](../data/Config.md) -- Config テンプレート仕様
