#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
LOCAL_TOOLS_BIN="${REPO_ROOT}/.tools/bin"
if [ -d "${LOCAL_TOOLS_BIN}" ]; then
  PATH="${LOCAL_TOOLS_BIN}:${PATH}"
fi

NAMESPACE="k1s0-service"
JOB_NAME="fault-injection-manual-$(date +%s)"

echo "Starting one-off fault window from CronJob template..."

kubectl delete job -n "${NAMESPACE}" -l app.kubernetes.io/part-of=k1s0-demo,fault-injection-run=manual --ignore-not-found >/dev/null 2>&1 || true
kubectl create job "${JOB_NAME}" --from=cronjob/fault-injection-test -n "${NAMESPACE}" >/dev/null
kubectl label job "${JOB_NAME}" -n "${NAMESPACE}" \
  app.kubernetes.io/part-of=k1s0-demo \
  fault-injection-run=manual \
  --overwrite >/dev/null

deadline=$((SECONDS + 60))
until kubectl get virtualservice order-server-fault-window -n "${NAMESPACE}" >/dev/null 2>&1; do
  if [ "${SECONDS}" -ge "${deadline}" ]; then
    echo "ERROR: fault injection job did not activate the fault window in time."
    exit 1
  fi
  sleep 2
done

echo "Fault window active for approximately 60 seconds."
echo "Track the job with:"
echo "  kubectl get job ${JOB_NAME} -n ${NAMESPACE} -w"
