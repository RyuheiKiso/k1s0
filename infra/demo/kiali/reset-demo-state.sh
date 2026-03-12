#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
LOCAL_TOOLS_BIN="${REPO_ROOT}/.tools/bin"
if [ -d "${LOCAL_TOOLS_BIN}" ]; then
  PATH="${LOCAL_TOOLS_BIN}:${PATH}"
fi

MODE="${1:-flagger}"
NAMESPACE="k1s0-service"
REVISION="reset-$(date +%s)"

delete_flagger_resources() {
  kubectl delete canary order-server -n "${NAMESPACE}" --ignore-not-found >/dev/null 2>&1 || true
  kubectl wait --for=delete canary/order-server -n "${NAMESPACE}" --timeout=60s >/dev/null 2>&1 || true
  kubectl delete deployment order-server-primary -n "${NAMESPACE}" --ignore-not-found >/dev/null 2>&1 || true
  kubectl delete service order-server-canary order-server-primary -n "${NAMESPACE}" --ignore-not-found >/dev/null 2>&1 || true
  kubectl delete virtualservice order-server -n "${NAMESPACE}" --ignore-not-found >/dev/null 2>&1 || true
  kubectl delete destinationrule order-server -n "${NAMESPACE}" --ignore-not-found >/dev/null 2>&1 || true
  kubectl delete configmap demo-service-script-primary -n "${NAMESPACE}" --ignore-not-found >/dev/null 2>&1 || true
}

wait_for_flagger_bootstrap() {
  local deadline=$((SECONDS + 180))
  local phase=""

  until kubectl get deployment order-server-primary -n "${NAMESPACE}" >/dev/null 2>&1 \
    && kubectl get service order-server-primary -n "${NAMESPACE}" >/dev/null 2>&1 \
    && kubectl get service order-server-canary -n "${NAMESPACE}" >/dev/null 2>&1 \
    && kubectl get virtualservice order-server -n "${NAMESPACE}" >/dev/null 2>&1 \
    && kubectl get destinationrule order-server-primary -n "${NAMESPACE}" >/dev/null 2>&1 \
    && kubectl get destinationrule order-server-canary -n "${NAMESPACE}" >/dev/null 2>&1; do
    if [ "${SECONDS}" -ge "${deadline}" ]; then
      echo "ERROR: Flagger did not bootstrap order-server resources in time."
      return 1
    fi
    sleep 5
  done

  while [ "${SECONDS}" -lt "${deadline}" ]; do
    phase="$(kubectl get canary order-server -n "${NAMESPACE}" -o jsonpath='{.status.phase}' 2>/dev/null || true)"
    if [ "${phase}" = "Initialized" ] || [ "${phase}" = "Succeeded" ]; then
      break
    fi
    sleep 5
  done

  if [ "${phase}" != "Initialized" ] && [ "${phase}" != "Succeeded" ]; then
    echo "ERROR: Flagger canary did not reach Initialized state."
    return 1
  fi

  kubectl wait --for=condition=available deployment/order-server-primary \
    -n "${NAMESPACE}" --timeout=120s >/dev/null
}

echo "Resetting demo state (${MODE})..."

kubectl delete virtualservice order-server-fault-window -n "${NAMESPACE}" --ignore-not-found >/dev/null 2>&1 || true
kubectl delete job -n "${NAMESPACE}" -l app.kubernetes.io/part-of=k1s0-demo,fault-injection-run=manual --ignore-not-found >/dev/null 2>&1 || true

delete_flagger_resources

kubectl apply -f "${SCRIPT_DIR}/manifests/02-virtualservices.yaml" >/dev/null

kubectl patch deployment order-server -n "${NAMESPACE}" --type strategic -p "
spec:
  template:
    metadata:
      labels:
        version: stable
        demo.k1s0.io/release-track: stable
        demo.k1s0.io/release-revision: ${REVISION}
    spec:
      containers:
        - name: stub
          env:
            - name: VERSION
              value: stable
            - name: FAILURE_RATE
              value: \"0\"
            - name: FIXED_DELAY_MS
              value: \"0\"
            - name: RELEASE_TRACK
              value: stable
            - name: RELEASE_REVISION
              value: ${REVISION}
" >/dev/null

kubectl rollout status deployment/order-server -n "${NAMESPACE}" --timeout=120s >/dev/null

if [ "${MODE}" = "flagger" ]; then
  kubectl apply -f "${SCRIPT_DIR}/manifests/12-flagger-canary.yaml" >/dev/null
  wait_for_flagger_bootstrap
fi

echo "Demo state is ready."
