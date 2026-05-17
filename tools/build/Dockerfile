# ---- python-base: Python 依存インストール ----
FROM python:3.12-slim AS python-base

RUN pip install --no-cache-dir uv

# ツール群の Python 依存を uv でインストール
RUN uv pip install --system --no-cache \
    pyyaml \
    jsonschema \
    click \
    rich \
    pytest

# ---- rust-tools: Rust ツールチェイン ----
FROM rust:1.95 AS rust-tools

# cargo-deny: Cargo 依存ライセンス/脆弱性検査
RUN cargo install cargo-deny --locked

# ---- go-tools: Go ツールチェイン ----
FROM golang:1.23 AS go-tools

# golangci-lint: Go 静的解析
RUN go install github.com/golangci/golangci-lint/cmd/golangci-lint@v1.61.0

# ---- node-tools: Node.js ツールチェイン ----
FROM node:20-slim AS node-tools

# markdownlint-cli + textlint: ドキュメント lint
RUN npm install -g \
    markdownlint-cli \
    textlint \
    textlint-rule-preset-japanese

# ---- final: 実行イメージ（全ステージの成果物を統合）----
FROM python:3.12-slim AS final

ENV PYTHONUNBUFFERED=1

# Python 依存をコピー
COPY --from=python-base /usr/local/lib/python3.12/site-packages /usr/local/lib/python3.12/site-packages
COPY --from=python-base /usr/local/bin/uv /usr/local/bin/uv
COPY --from=python-base /usr/local/bin/pytest /usr/local/bin/pytest

# Rust ツールをコピー
COPY --from=rust-tools /usr/local/cargo/bin/cargo-deny /usr/local/bin/cargo-deny

# Go ツールをコピー
COPY --from=go-tools /go/bin/golangci-lint /usr/local/bin/golangci-lint

# Node.js ランタイムとツールをコピー
COPY --from=node-tools /usr/local/bin/node /usr/local/bin/node
COPY --from=node-tools /usr/local/lib/node_modules /usr/local/lib/node_modules
COPY --from=node-tools /usr/local/bin/markdownlint /usr/local/bin/markdownlint
COPY --from=node-tools /usr/local/bin/textlint /usr/local/bin/textlint

WORKDIR /workspace

CMD ["python3", "--version"]
