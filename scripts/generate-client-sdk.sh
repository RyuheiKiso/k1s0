#!/usr/bin/env bash
set -euo pipefail

# Multi-language Client SDK Generator
# Usage: ./scripts/generate-client-sdk.sh --service <name> --languages <go,rust,ts,dart> --proto <path>

usage() {
  echo "Usage: $0 --service <name> --languages <go,rust,ts,dart> --proto <proto-dir>"
  echo ""
  echo "Options:"
  echo "  --service     Service name (e.g., user-profile)"
  echo "  --languages   Comma-separated list of target languages (default: go,rust,ts,dart)"
  echo "  --proto       Path to proto directory containing .proto files"
  exit 1
}

SERVICE=""
LANGUAGES="go,rust,ts,dart"
PROTO_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --service) SERVICE="$2"; shift 2 ;;
    --languages) LANGUAGES="$2"; shift 2 ;;
    --proto) PROTO_DIR="$2"; shift 2 ;;
    *) usage ;;
  esac
done

[[ -z "$SERVICE" || -z "$PROTO_DIR" ]] && usage

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEMPLATE_DIR="$ROOT_DIR/CLI/crates/k1s0-cli/templates/client-sdk"
OUTPUT_BASE="$ROOT_DIR/regions/system/library"

# Extract service info from proto files
SERVICE_SNAKE=$(echo "$SERVICE" | tr '-' '_')
# macOS 互換のため sed -E を使用（-r は GNU sed 専用）
SERVICE_PASCAL=$(echo "$SERVICE" | sed -E 's/(^|-)([a-z])/\U\2/g')

echo "=== Generating Client SDK for $SERVICE ==="
echo "  Languages: $LANGUAGES"
echo "  Proto dir: $PROTO_DIR"

# Extract methods from proto service definitions
extract_methods() {
  grep -E '^\s*rpc\s+' "$PROTO_DIR"/*.proto 2>/dev/null | \
    sed -E 's/.*rpc\s+(\w+)\s*\(\s*(\w+)\s*\)\s*returns\s*\(\s*(\w+)\s*\).*/\1|\2|\3/' || true
}

METHODS=$(extract_methods)

IFS=',' read -ra LANG_ARRAY <<< "$LANGUAGES"
for lang in "${LANG_ARRAY[@]}"; do
  echo "--- Generating $lang SDK ---"

  case "$lang" in
    go)
      OUT_DIR="$OUTPUT_BASE/go/${SERVICE_SNAKE}_client"
      mkdir -p "$OUT_DIR"

      if [[ -d "$TEMPLATE_DIR/go" ]]; then
        for tmpl in "$TEMPLATE_DIR/go"/*.tera; do
          [[ -f "$tmpl" ]] || continue
          base=$(basename "$tmpl" .tera)
          # Simple template variable substitution
          sed -e "s/{{ service_name }}/$SERVICE/g" \
              -e "s/{{ service_snake }}/$SERVICE_SNAKE/g" \
              -e "s/{{ service_pascal }}/$SERVICE_PASCAL/g" \
              "$tmpl" > "$OUT_DIR/$base"
        done
        echo "  Generated: $OUT_DIR"
      else
        echo "  WARNING: Go templates not found at $TEMPLATE_DIR/go"
      fi
      ;;
    rust)
      OUT_DIR="$OUTPUT_BASE/rust/${SERVICE_SNAKE}-client"
      mkdir -p "$OUT_DIR/src"

      if [[ -d "$TEMPLATE_DIR/rust" ]]; then
        for tmpl in "$TEMPLATE_DIR/rust"/*.tera; do
          [[ -f "$tmpl" ]] || continue
          base=$(basename "$tmpl" .tera)
          sed -e "s/{{ service_name }}/$SERVICE/g" \
              -e "s/{{ service_snake }}/$SERVICE_SNAKE/g" \
              -e "s/{{ service_pascal }}/$SERVICE_PASCAL/g" \
              "$tmpl" > "$OUT_DIR/$base"
        done
        for tmpl in "$TEMPLATE_DIR/rust/src"/*.tera; do
          [[ -f "$tmpl" ]] || continue
          base=$(basename "$tmpl" .tera)
          sed -e "s/{{ service_name }}/$SERVICE/g" \
              -e "s/{{ service_snake }}/$SERVICE_SNAKE/g" \
              -e "s/{{ service_pascal }}/$SERVICE_PASCAL/g" \
              "$tmpl" > "$OUT_DIR/src/$base"
        done
        echo "  Generated: $OUT_DIR"
      else
        echo "  WARNING: Rust templates not found at $TEMPLATE_DIR/rust"
      fi
      ;;
    ts)
      OUT_DIR="$OUTPUT_BASE/typescript/${SERVICE}_client"
      mkdir -p "$OUT_DIR/src"

      if [[ -d "$TEMPLATE_DIR/typescript" ]]; then
        for tmpl in "$TEMPLATE_DIR/typescript"/*.tera; do
          [[ -f "$tmpl" ]] || continue
          base=$(basename "$tmpl" .tera)
          sed -e "s/{{ service_name }}/$SERVICE/g" \
              -e "s/{{ service_snake }}/$SERVICE_SNAKE/g" \
              -e "s/{{ service_pascal }}/$SERVICE_PASCAL/g" \
              "$tmpl" > "$OUT_DIR/$base"
        done
        for tmpl in "$TEMPLATE_DIR/typescript/src"/*.tera; do
          [[ -f "$tmpl" ]] || continue
          base=$(basename "$tmpl" .tera)
          sed -e "s/{{ service_name }}/$SERVICE/g" \
              -e "s/{{ service_snake }}/$SERVICE_SNAKE/g" \
              -e "s/{{ service_pascal }}/$SERVICE_PASCAL/g" \
              "$tmpl" > "$OUT_DIR/src/$base"
        done
        echo "  Generated: $OUT_DIR"
      else
        echo "  WARNING: TypeScript templates not found at $TEMPLATE_DIR/typescript"
      fi
      ;;
    dart)
      OUT_DIR="$OUTPUT_BASE/dart/${SERVICE_SNAKE}_client"
      mkdir -p "$OUT_DIR/lib/src"

      if [[ -d "$TEMPLATE_DIR/dart" ]]; then
        for tmpl in "$TEMPLATE_DIR/dart"/*.tera; do
          [[ -f "$tmpl" ]] || continue
          base=$(basename "$tmpl" .tera)
          sed -e "s/{{ service_name }}/$SERVICE/g" \
              -e "s/{{ service_snake }}/$SERVICE_SNAKE/g" \
              -e "s/{{ service_pascal }}/$SERVICE_PASCAL/g" \
              "$tmpl" > "$OUT_DIR/$base"
        done
        for tmpl in "$TEMPLATE_DIR/dart/lib/src"/*.tera; do
          [[ -f "$tmpl" ]] || continue
          base=$(basename "$tmpl" .tera)
          sed -e "s/{{ service_name }}/$SERVICE/g" \
              -e "s/{{ service_snake }}/$SERVICE_SNAKE/g" \
              -e "s/{{ service_pascal }}/$SERVICE_PASCAL/g" \
              "$tmpl" > "$OUT_DIR/lib/src/$base"
        done
        echo "  Generated: $OUT_DIR"
      else
        echo "  WARNING: Dart templates not found at $TEMPLATE_DIR/dart"
      fi
      ;;
    *)
      echo "  WARNING: Unknown language '$lang', skipping"
      ;;
  esac
done

echo "=== SDK generation complete ==="
