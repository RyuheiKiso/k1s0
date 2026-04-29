# =============================================================================
# t1-workflow Pod 用 Dockerfile（WorkflowService、Dapr Workflow + Temporal 振り分け）
#
# 設計: docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md
# 関連 ID: IMP-BUILD-002 / FR-T1-WORKFLOW / ADR-TIER1-001 / ADR-RULE-002
#
# build context: リポジトリルート必須（replace ../../sdk/go のため）
#   docker build -f src/tier1/go/Dockerfile.workflow -t k1s0-tier1-workflow:dev .
# =============================================================================

FROM golang:1.25-alpine AS builder

WORKDIR /workspace
COPY src/sdk/go /workspace/src/sdk/go
COPY src/tier1/go /workspace/src/tier1/go
WORKDIR /workspace/src/tier1/go

RUN CGO_ENABLED=0 go build \
    -ldflags="-s -w" \
    -trimpath \
    -o /out/t1-workflow \
    ./cmd/workflow

FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=builder /out/t1-workflow /usr/local/bin/t1-workflow

EXPOSE 50001 50080 9090

USER nonroot:nonroot

ENTRYPOINT ["/usr/local/bin/t1-workflow"]
