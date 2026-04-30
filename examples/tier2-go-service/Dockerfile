# tier2 Go サービス Golden Path 完動例の Dockerfile。
# distroless / nonroot / multi-stage で `src/tier1/go/Dockerfile.state` と整合。
#
# build context は **リポジトリルート** で指定すること（`docker build -f
# examples/tier2-go-service/Dockerfile -t k1s0-example-tier2-go:dev .`）。
# go.mod の replace で `../../src/sdk/go` / `../../src/tier2/go` を参照しているため、
# build context にこれらが含まれている必要がある。

# stage 1: build
FROM golang:1.25-alpine AS builder
WORKDIR /workspace

# replace 先の依存を先にコピー（cache 効率化）。
COPY src/sdk/go /workspace/src/sdk/go
COPY src/tier2/go /workspace/src/tier2/go

# 依存解決を行う（example 自体の go.mod / go.sum）。
COPY examples/tier2-go-service/go.mod examples/tier2-go-service/go.sum /workspace/examples/tier2-go-service/
WORKDIR /workspace/examples/tier2-go-service
RUN go mod download

# ソースをコピーしてビルド。
COPY examples/tier2-go-service/cmd/ cmd/
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o /out/example-service ./cmd/example-service

# stage 2: runtime（distroless static / nonroot）
FROM gcr.io/distroless/static-debian12:nonroot
WORKDIR /
COPY --from=builder /out/example-service /example-service
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/example-service"]
