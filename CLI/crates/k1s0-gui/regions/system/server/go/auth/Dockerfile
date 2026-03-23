# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull golang@sha256:1a6d4452c65dea36aac2e2d606b01b4a029ec90cc1ae53890540ce6173ea77ac && docker inspect --format="{{index .RepoDigests 0}}" golang@sha256:1a6d4452c65dea36aac2e2d606b01b4a029ec90cc1ae53890540ce6173ea77ac
FROM golang@sha256:1a6d4452c65dea36aac2e2d606b01b4a029ec90cc1ae53890540ce6173ea77ac AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /bin/auth ./cmd/

# 実行ステージ: 軽量Alpineイメージでバイナリを実行する
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull alpine@sha256:6baf43584bcb78f2e5847d1de515f23499913ac9f12bdf834811a3145eb11ca1 && docker inspect --format="{{index .RepoDigests 0}}" alpine@sha256:6baf43584bcb78f2e5847d1de515f23499913ac9f12bdf834811a3145eb11ca1
FROM alpine@sha256:6baf43584bcb78f2e5847d1de515f23499913ac9f12bdf834811a3145eb11ca1

# 非rootユーザーでアプリケーションを実行するためのセキュリティ設定
# rootで実行しないことで、コンテナ脱出時のホストへの影響を最小化する
RUN addgroup -S app && adduser -S app -G app

COPY --from=builder /bin/auth /bin/auth

# appユーザーに切り替えて最小権限で実行する
USER app
ENTRYPOINT ["/bin/auth"]
