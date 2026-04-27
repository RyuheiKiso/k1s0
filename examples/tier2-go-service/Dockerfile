# tier2 Go サービス Golden Path 完動例の Dockerfile。
# distroless / nonroot / multi-stage で `src/tier1/go/Dockerfile.state` と整合。

# stage 1: build
FROM golang:1.22-alpine AS builder
WORKDIR /workspace

# 依存解決を先に行いキャッシュ効率を最大化
COPY go.mod go.sum ./
RUN go mod download

# ソースをコピーしてビルド
COPY cmd/ cmd/
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o /out/example-service ./cmd/example-service

# stage 2: runtime（distroless static / nonroot）
FROM gcr.io/distroless/static-debian12:nonroot
WORKDIR /
COPY --from=builder /out/example-service /example-service
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/example-service"]
