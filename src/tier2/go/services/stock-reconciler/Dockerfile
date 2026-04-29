# stock-reconciler サービスの multi-stage Dockerfile。
#
# docs 正典:
#   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
#
# build context 要件:
#   replace ../../sdk/go directive のためリポジトリルートが必要:
#     docker build -f src/tier2/go/services/stock-reconciler/Dockerfile -t k1s0-stock-reconciler:dev .

# stage 1: build。
FROM golang:1.25-alpine AS builder

WORKDIR /workspace
# replace ../../sdk/go を満たすため sdk-go を先に取り込む。
COPY src/sdk/go /workspace/src/sdk/go
COPY src/tier2/go /workspace/src/tier2/go
WORKDIR /workspace/src/tier2/go

# 静的バイナリをビルドする（CGO 無効、symbol/dwarf 削除）。
RUN CGO_ENABLED=0 go build \
    -ldflags="-s -w" \
    -trimpath \
    -o /out/stock-reconciler \
    ./services/stock-reconciler/cmd/

# stage 2: runtime（distroless static + nonroot）。
FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=builder /out/stock-reconciler /usr/local/bin/stock-reconciler

USER nonroot

EXPOSE 8080 9090

ENTRYPOINT ["/usr/local/bin/stock-reconciler"]
