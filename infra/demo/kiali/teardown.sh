#!/usr/bin/env bash
# Clean up the Kiali demo environment
set -euo pipefail

CLUSTER_NAME="${CLUSTER_NAME:-k1s0-demo}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== k1s0 Kiali Demo Teardown ==="

# Stop any legacy port-forward processes from older revisions
for pid_file in .kiali-pf.pid .prometheus-pf.pid .jaeger-otlp-pf.pid; do
  if [ -f "${SCRIPT_DIR}/${pid_file}" ]; then
    PF_PID=$(cat "${SCRIPT_DIR}/${pid_file}")
    if kill -0 "$PF_PID" 2>/dev/null; then
      echo "Stopping legacy port-forward (PID: ${PF_PID})..."
      kill "$PF_PID" 2>/dev/null || true
    fi
    rm -f "${SCRIPT_DIR:?}/${pid_file}"
  fi
done

if kind get clusters 2>/dev/null | grep -q "^${CLUSTER_NAME}$"; then
  echo "Deleting kind cluster '${CLUSTER_NAME}'..."
  kind delete cluster --name "${CLUSTER_NAME}"
  echo "Cluster deleted."
else
  echo "Cluster '${CLUSTER_NAME}' not found. Nothing to do."
fi

echo "=== Teardown Complete ==="
