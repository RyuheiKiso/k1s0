#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
LOCAL_TOOLS_BIN="${REPO_ROOT}/.tools/bin"
if [ -d "${LOCAL_TOOLS_BIN}" ]; then
  PATH="${LOCAL_TOOLS_BIN}:${PATH}"
fi

MODE="${1:-promote}"
NAMESPACE="k1s0-service"
REVISION="${MODE}-$(date +%s)"

if ! kubectl get canary order-server -n "${NAMESPACE}" >/dev/null 2>&1; then
  echo "ERROR: Flagger canary for order-server is not installed."
  exit 1
fi

kubectl wait --for=condition=available deployment/order-server-primary \
  -n "${NAMESPACE}" --timeout=120s >/dev/null

case "${MODE}" in
  promote)
    VERSION_VALUE="canary-v2"
    FAILURE_RATE="0"
    FIXED_DELAY_MS="0"
    RELEASE_TRACK="promotion"
    ;;
  rollback)
    VERSION_VALUE="canary-bad"
    FAILURE_RATE="0.25"
    FIXED_DELAY_MS="1200"
    RELEASE_TRACK="rollback"
    ;;
  *)
    echo "ERROR: Unknown rollout mode: ${MODE}"
    exit 1
    ;;
esac

echo "Starting Flagger rollout (${MODE})..."

kubectl patch deployment order-server -n "${NAMESPACE}" --type strategic -p "
spec:
  template:
    metadata:
      labels:
        version: canary
        demo.k1s0.io/release-track: ${RELEASE_TRACK}
        demo.k1s0.io/release-revision: ${REVISION}
    spec:
      containers:
        - name: stub
          env:
            - name: VERSION
              value: ${VERSION_VALUE}
            - name: FAILURE_RATE
              value: \"${FAILURE_RATE}\"
            - name: FIXED_DELAY_MS
              value: \"${FIXED_DELAY_MS}\"
            - name: RELEASE_TRACK
              value: ${RELEASE_TRACK}
            - name: RELEASE_REVISION
              value: ${REVISION}
" >/dev/null

kubectl rollout status deployment/order-server -n "${NAMESPACE}" --timeout=120s >/dev/null

echo "Flagger rollout triggered. Watch with:"
echo "  kubectl get canary order-server -n ${NAMESPACE} -w"
