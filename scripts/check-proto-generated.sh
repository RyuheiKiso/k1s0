#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"$ROOT_DIR/scripts/generate-proto.sh"

# リポジトリルートの gen/ ディレクトリが存在しないことを確認する。
# Proto 生成物は api/proto/gen/、OpenAPI 生成物は api/openapi/gen/ に配置すること。
# ルート直下の gen/ は使用禁止（CI と proto README ポリシーに基づく）。
if [[ -d "$ROOT_DIR/gen" ]]; then
  echo "Unexpected legacy generated directory detected at $ROOT_DIR/gen. Remove it and keep generated artifacts under api/proto/gen only."
  exit 1
fi

# Go / TypeScript / Rust 生成物は常に最新であることを要求する（厳格チェック）
if [[ -n "$(git -C "$ROOT_DIR" status --short -- api/proto/gen/go api/proto/gen/ts api/proto/gen/rust)" ]]; then
  echo "Generated proto artifacts are out of date. Run scripts/generate-proto.sh and commit the changes."
  git -C "$ROOT_DIR" status --short -- api/proto/gen/go api/proto/gen/ts api/proto/gen/rust
  exit 1
fi

# Dart 生成物は protoc-gen-dart インストール環境でのみ生成される（C-002）。
# CI に Dart SDK + protoc_plugin が設定されており、生成物は自動コミットされる。
# 差分がある場合はローカルで `buf generate api/proto` を実行してコミットすること。
if [[ -n "$(git -C "$ROOT_DIR" status --short -- api/proto/gen/dart)" ]]; then
  echo "WARNING: Dart proto artifacts are out of date."
  echo "  Run: dart pub global activate protoc_plugin && buf generate api/proto"
  echo "  Then commit the generated files in api/proto/gen/dart/"
  git -C "$ROOT_DIR" status --short -- api/proto/gen/dart
  # Dart 生成物の差分は警告のみ（CI 初期セットアップ期間中は非ブロッキング）
  # TODO(C-002): Dart 生成物の自動コミットが整備されたらここを exit 1 に変更する
fi
