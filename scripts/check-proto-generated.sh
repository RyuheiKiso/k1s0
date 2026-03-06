#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"$ROOT_DIR/scripts/generate-proto.sh"

if [[ -n "$(git -C "$ROOT_DIR" status --short -- api/proto/gen)" ]]; then
  echo "Generated proto artifacts are out of date. Run scripts/generate-proto.sh and commit the changes."
  git -C "$ROOT_DIR" status --short -- api/proto/gen
  exit 1
fi