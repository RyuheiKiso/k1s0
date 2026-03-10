#!/usr/bin/env bash
# Generate demo traffic by hitting the local order-bff process.
# The demo service fans out to downstream services, emits JSON logs,
# and forwards B3 headers so Istio produces real metrics and traces.
set -euo pipefail

INTERVAL="${1:-2}"
DURATION="${2:-300}"
MODE="${3:-normal}"
BURST_SIZE="${BURST_SIZE:-5}"

echo "=== Traffic Generator ==="
echo "Interval: ${INTERVAL}s | Duration: ${DURATION}s | Mode: ${MODE} | Burst: ${BURST_SIZE}"
echo "Press Ctrl+C to stop"
echo ""

ORDER_BFF=$(kubectl get pod -n k1s0-service -l app=order-bff -o jsonpath='{.items[0].metadata.name}')

echo "Root pod found:"
echo "  order-bff: ${ORDER_BFF}"
echo ""

END_TIME=$((SECONDS + DURATION))
COUNT=0

while [ "${SECONDS}" -lt "${END_TIME}" ]; do
  COUNT=$((COUNT + 1))
  echo "[$(date +%H:%M:%S)] Round ${COUNT}"

  if ! kubectl exec -i "${ORDER_BFF}" -n k1s0-service -c stub -- \
    python - "${MODE}" "${BURST_SIZE}" <<'PY'
import json
import secrets
import sys
import urllib.error
import urllib.request

mode = sys.argv[1]
burst_size = int(sys.argv[2])


def hex_id(bytes_len: int) -> str:
    return secrets.token_hex(bytes_len)


for idx in range(burst_size):
    trace_id = hex_id(16)
    span_id = hex_id(8)
    headers = {
        "x-b3-traceid": trace_id,
        "x-b3-spanid": span_id,
        "x-b3-sampled": "1",
        "x-request-id": trace_id,
    }
    if mode == "canary":
        headers["x-canary"] = "true"

    req = urllib.request.Request("http://127.0.0.1:8080/", headers=headers, method="GET")
    status = 200

    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            status = resp.getcode()
            resp.read()
    except urllib.error.HTTPError as exc:
        status = exc.code
    except Exception:
        status = 599

    print(json.dumps({"flow": idx + 1, "status": status, "trace_id": trace_id}, separators=(",", ":")))
PY
  then
    echo "  traffic round execution failed"
  fi

  echo ""
  sleep "${INTERVAL}"
done

echo "=== Traffic generation complete (${COUNT} rounds) ==="
