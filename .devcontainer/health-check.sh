#!/bin/bash
# .devcontainer/health-check.sh
# devcontainer セットアップ完了後のツール確認スクリプト
# post-create.sh の最後に呼び出し、インストール済みツールのバージョンを一覧表示する

set -e

ok()   { echo "  [OK]  $*"; }
fail() { echo "  [NG]  $*"; FAILED=1; }

FAILED=0

echo ""
echo "=== devcontainer ツール確認 ==="
echo ""

# Go
if command -v go &>/dev/null; then
    ok "Go:            $(go version)"
else
    fail "Go が見つかりません"
fi

# Rust
if command -v rustc &>/dev/null; then
    ok "Rust:          $(rustc --version)"
else
    fail "Rust が見つかりません"
fi

# Node.js
if command -v node &>/dev/null; then
    ok "Node.js:       $(node --version)"
else
    fail "Node.js が見つかりません"
fi

# pnpm
if command -v pnpm &>/dev/null; then
    ok "pnpm:          $(pnpm --version)"
else
    fail "pnpm が見つかりません（corepack enable pnpm を実行してください）"
fi

# protobuf コンパイラ
if command -v protoc &>/dev/null; then
    ok "protoc:        $(protoc --version)"
else
    fail "protoc が見つかりません"
fi

# buf
if command -v buf &>/dev/null; then
    ok "buf:           $(buf --version)"
else
    fail "buf が見つかりません"
fi

# just
if command -v just &>/dev/null; then
    ok "just:          $(just --version)"
else
    fail "just が見つかりません"
fi

# sqlx-cli
if command -v sqlx &>/dev/null; then
    ok "sqlx:          $(sqlx --version)"
else
    fail "sqlx が見つかりません"
fi

# k1s0 CLI（プロジェクト管理ツール）
if command -v k1s0 &>/dev/null; then
    ok "k1s0:          $(k1s0 --version 2>/dev/null || echo '(version不明)')"
else
    fail "k1s0 CLI が見つかりません（cargo install --path CLI/crates/k1s0-cli を実行してください）"
fi

# Docker（docker-in-docker 経由）
if command -v docker &>/dev/null; then
    ok "Docker:        $(docker --version 2>/dev/null || echo '(デーモン未起動)')"
else
    fail "Docker が見つかりません"
fi

# Flutter（devcontainer.json の features で事前インストール済み）
if command -v flutter &>/dev/null; then
    ok "Flutter:       $(flutter --version 2>&1 | head -1)"
else
    fail "Flutter が見つかりません"
fi

# Dart（Flutter SDK に同梱）
if command -v dart &>/dev/null; then
    ok "Dart:          $(dart --version 2>&1)"
else
    fail "Dart が見つかりません"
fi

echo ""

if [[ "${FAILED}" -eq 0 ]]; then
    echo "全ツールの確認が完了しました。開発を開始できます。"
else
    echo "一部のツールが見つかりませんでした。post-create.sh のログを確認してください。"
    exit 1
fi

echo ""
