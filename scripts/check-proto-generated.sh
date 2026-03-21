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

if [[ -n "$(git -C "$ROOT_DIR" status --short -- api/proto/gen)" ]]; then
  echo "Generated proto artifacts are out of date. Run scripts/generate-proto.sh and commit the changes."
  git -C "$ROOT_DIR" status --short -- api/proto/gen
  exit 1
fi
