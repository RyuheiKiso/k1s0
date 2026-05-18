#!/usr/bin/env bash
# notarize.sh — 日次公証 script: audit hash chain root を RFC 3161 + Sigstore で attest

set -euo pipefail

# 対象 audit hash root を取得する
AUDIT_ROOT_HASH=$(python3 tools/lock_yaml_generator/generate_audit_root_hash.py)

# RFC 3161 timestamp を取得する
openssl ts -query -data <(echo "$AUDIT_ROOT_HASH") -no_nonce -sha256 -out /tmp/ts_req.tsq
curl -s -S -o /tmp/ts_resp.tsr \
  -H "Content-Type: application/timestamp-query" \
  --data-binary @/tmp/ts_req.tsq \
  http://timestamp.digicert.com

# Sigstore transparency log に記録する
cosign attest --type slsaprovenance \
  --predicate /tmp/audit_predicate.json \
  "ghcr.io/k1s0/audit-root:${AUDIT_ROOT_HASH}"

echo "Notarization complete: RFC 3161 + Sigstore recorded"
