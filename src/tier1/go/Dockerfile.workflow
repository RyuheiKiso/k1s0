# =============================================================================
# t1-workflow Pod 用 Dockerfile（Workflow API、Dapr Workflow + Temporal 振り分け）
#
# 設計: docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md（Pod 別 Dockerfile 配置正典）
#       docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md
#       plan/04_tier1_Goファサード実装/01_リポジトリレイアウト.md
# 関連 ID: IMP-BUILD-002 / IMP-BUILD-DOCKER-* / FR-T1-WORKFLOW / ADR-TIER1-001
#
# build context: リポジトリルート（`docker build -f src/tier1/go/Dockerfile.workflow src/tier1/go/`）
#
# 設計方針: Dockerfile.state と同じ multi-stage / distroless / nonroot 方針。
#           違いはビルドターゲットが ./cmd/workflow、出力 binary が t1-workflow なのみ。
# =============================================================================

# build stage: Go 1.22 Alpine
FROM golang:1.22-alpine AS builder

# ビルド作業ディレクトリ。
WORKDIR /workspace

# 依存解決（レイヤキャッシュ最適化）。
COPY go.mod go.sum ./
RUN go mod download

# ソース全体コピー → ビルド。
COPY . .

# t1-workflow バイナリをビルド（CGO_ENABLED=0 / -ldflags="-s -w" / -trimpath）。
RUN CGO_ENABLED=0 go build \
    -ldflags="-s -w" \
    -trimpath \
    -o /out/t1-workflow \
    ./cmd/workflow

# =============================================================================
# runtime stage: distroless static
# =============================================================================
FROM gcr.io/distroless/static-debian12:nonroot

# バイナリコピー。
COPY --from=builder /out/t1-workflow /usr/local/bin/t1-workflow

# gRPC + metrics ポート公開。
EXPOSE 50001 9090

# nonroot user（UID 65532）。
USER nonroot:nonroot

# exec 形式 ENTRYPOINT。
ENTRYPOINT ["/usr/local/bin/t1-workflow"]
