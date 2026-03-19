# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull golang:1.24-bookworm && docker inspect --format="{{index .RepoDigests 0}}" golang:1.24-bookworm
FROM golang:1.24-bookworm AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /bin/auth ./cmd/

# 実行ステージ: 軽量Alpineイメージでバイナリを実行する
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull alpine:3.19 && docker inspect --format="{{index .RepoDigests 0}}" alpine:3.19
FROM alpine:3.19

# 非rootユーザーでアプリケーションを実行するためのセキュリティ設定
# rootで実行しないことで、コンテナ脱出時のホストへの影響を最小化する
RUN addgroup -S app && adduser -S app -G app

COPY --from=builder /bin/auth /bin/auth

# appユーザーに切り替えて最小権限で実行する
USER app
ENTRYPOINT ["/bin/auth"]
