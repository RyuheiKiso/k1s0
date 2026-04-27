# stock-reconciler サービスの multi-stage Dockerfile。
#
# docs 正典:
#   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
#
# build context 要件:
#   docker build は src/tier2/go/ をルートとして実行する前提で書いている。
#   CI / 開発者は次のコマンドで build する:
#     docker build -f src/tier2/go/services/stock-reconciler/Dockerfile src/tier2/go/
#   リポジトリルートから build すると go.mod の位置が合わず、`COPY go.mod` が失敗する。

# stage 1: build。
FROM golang:1.22 AS builder

# 作業ディレクトリ。
WORKDIR /workspace

# モジュール情報を先に COPY して download をキャッシュさせる。
COPY go.mod go.sum ./

# モジュールを download する（vendor 不採用）。
RUN go mod download

# ソース全体を COPY する。
COPY . .

# 静的バイナリをビルドする（CGO 無効、symbol/dwarf 削除）。
RUN CGO_ENABLED=0 go build \
    -ldflags="-s -w" \
    -trimpath \
    -o bin/stock-reconciler \
    ./services/stock-reconciler/cmd/

# stage 2: runtime（distroless static + nonroot）。
FROM gcr.io/distroless/static-debian12:nonroot

# build artifact を取り込む。
COPY --from=builder /workspace/bin/stock-reconciler /usr/local/bin/stock-reconciler

# 非 root 実行（distroless:nonroot は uid 65532）。
USER nonroot

# 公開ポート（8080: HTTP、9090: metrics 想定の予約）。
EXPOSE 8080 9090

# エントリポイント。
ENTRYPOINT ["/usr/local/bin/stock-reconciler"]
