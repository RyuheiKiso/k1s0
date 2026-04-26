# =============================================================================
# examples/tier1-go-facade Dockerfile（IMP-DIR-COMM-113 / Golden Path 最小例）
#
# 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md
#       src/tier1/go/Dockerfile.state（distroless / nonroot / multi-stage の正典形）
# 関連 ID: IMP-BUILD-002 / ADR-TIER1-001
#
# build context: 本ディレクトリ（`docker build -t k1s0-example-facade examples/tier1-go-facade/`）
#
# 設計方針:
#   - multi-stage: golang:1.22-alpine で build → distroless/static-debian12:nonroot で run
#   - CGO_ENABLED=0: pure Go ビルド、glibc 依存なしで distroless でも動作
#   - -ldflags="-s -w": symbol table と debug info 削除でバイナリサイズ削減
#   - -trimpath: ビルド path を normalize（再現性向上、CI/CD で同一ハッシュ保証）
#   - distroless: shell / package manager 含まない最小 base image（攻撃面最小化）
#   - nonroot: UID 65532 で起動（特権削除、k8s securityContext 整合）
# =============================================================================

# build stage: Go 1.22 Alpine（最小サイズの builder）
FROM golang:1.22-alpine AS builder

# ビルド作業ディレクトリ。
WORKDIR /workspace

# 依存関係を先に解決してレイヤキャッシュを効かせる（go.mod 変更時のみ再 download）。
COPY go.mod go.sum ./
RUN go mod download

# ソース全体をコピーしてビルド。
COPY . .

# example-facade バイナリをビルド。
# -ldflags="-s -w" で symbol/debug info 削除、-trimpath で path 正規化。
RUN CGO_ENABLED=0 go build \
    -ldflags="-s -w" \
    -trimpath \
    -o /out/example-facade \
    ./cmd/example-facade

# =============================================================================
# runtime stage: distroless static（debian12 ベース、shell/pkg manager なし）
# =============================================================================
FROM gcr.io/distroless/static-debian12:nonroot

# build stage からバイナリのみをコピー（ソース / Go toolchain は image に残らない）。
COPY --from=builder /out/example-facade /usr/local/bin/example-facade

# gRPC（50001）ポートを公開する（docs 正典 EXPOSE）。
EXPOSE 50001

# distroless の nonroot user（UID 65532）で起動。
USER nonroot:nonroot

# exec 形式の ENTRYPOINT（PID 1 で signal が直接バイナリに届く、graceful shutdown が正常動作）。
ENTRYPOINT ["/usr/local/bin/example-facade"]
