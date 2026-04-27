# notification-hub サービスの multi-stage Dockerfile。
#
# docs 正典:
#   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
#
# build context 要件:
#   docker build は src/tier2/go/ をルートとして実行する前提で書いている。
#   CI / 開発者は次のコマンドで build する:
#     docker build -f src/tier2/go/services/notification-hub/Dockerfile src/tier2/go/

# stage 1: build。
FROM golang:1.22 AS builder

# 作業ディレクトリ。
WORKDIR /workspace

# モジュール情報を先に COPY して download をキャッシュさせる。
COPY go.mod go.sum ./

# モジュールを download する。
RUN go mod download

# ソース全体を COPY する。
COPY . .

# 静的バイナリをビルドする。
RUN CGO_ENABLED=0 go build \
    -ldflags="-s -w" \
    -trimpath \
    -o bin/notification-hub \
    ./services/notification-hub/cmd/

# stage 2: runtime（distroless static + nonroot）。
FROM gcr.io/distroless/static-debian12:nonroot

# build artifact を取り込む。
COPY --from=builder /workspace/bin/notification-hub /usr/local/bin/notification-hub

# 非 root 実行。
USER nonroot

# 公開ポート（8080: HTTP）。
EXPOSE 8080 9090

# エントリポイント。
ENTRYPOINT ["/usr/local/bin/notification-hub"]
