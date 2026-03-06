#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROTO_DIR="$ROOT_DIR/api/proto"
TEMPLATE="$PROTO_DIR/buf.gen.yaml"

buf generate "$PROTO_DIR" --template "$TEMPLATE"