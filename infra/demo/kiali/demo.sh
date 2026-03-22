#!/usr/bin/env bash
# Interactive Kiali demo script
# Exercises the acceptance-critical Flagger and CronJob flows.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
LOCAL_TOOLS_BIN="${REPO_ROOT}/.tools/bin"
if [ -d "${LOCAL_TOOLS_BIN}" ]; then
  PATH="${LOCAL_TOOLS_BIN}:${PATH}"
fi

CLUSTER_NAME="${CLUSTER_NAME:-k1s0-demo}"
TRAFFIC_PID=""

cleanup() {
  if [ -n "${TRAFFIC_PID}" ] && kill -0 "${TRAFFIC_PID}" 2>/dev/null; then
    kill "${TRAFFIC_PID}" 2>/dev/null || true
    wait "${TRAFFIC_PID}" 2>/dev/null || true
  fi
}
trap cleanup EXIT

stop_traffic() {
  if [ -n "${TRAFFIC_PID}" ] && kill -0 "${TRAFFIC_PID}" 2>/dev/null; then
    kill "${TRAFFIC_PID}" 2>/dev/null || true
    wait "${TRAFFIC_PID}" 2>/dev/null || true
    TRAFFIC_PID=""
  fi
}

start_traffic() {
  local mode="${1:-normal}"
  stop_traffic
  bash "${SCRIPT_DIR}/traffic-gen.sh" 2 3600 "${mode}" &
  TRAFFIC_PID=$!
  echo "  Traffic generator started (PID: ${TRAFFIC_PID}, mode: ${mode})"
}

reset_flagger_state() {
  echo ""
  echo "Resetting to Flagger-managed baseline..."
  bash "${SCRIPT_DIR}/reset-demo-state.sh" flagger
}

reset_manual_state() {
  echo ""
  echo "Resetting to manual baseline..."
  bash "${SCRIPT_DIR}/reset-demo-state.sh" manual
}

show_menu() {
  echo ""
  echo "==========================================="
  echo "       k1s0 Service Mesh Demo"
  echo "==========================================="
  echo ""
  echo " [Acceptance Flows]"
  echo "  1) Normal Traffic        -- Flagger baseline, healthy mesh traffic"
  echo "  2) Auto Canary Promote   -- Flagger shifts 20% steps to canary"
  echo "  3) Auto Canary Rollback  -- Flagger aborts on 503/latency regression"
  echo "  4) Scheduled Fault Test  -- CronJob template creates a 60s fault window"
  echo ""
  echo " [Observability]"
  echo "  5) Distributed Tracing   -- Jaeger trace chain for order flow"
  echo "  6) Log Aggregation       -- Grafana + Loki structured logs"
  echo "  7) Kafka Messaging       -- Async event flow visualization"
  echo ""
  echo " [Control]"
  echo "  8) Reset All             -- Restore Flagger baseline and resume traffic"
  echo "  q) Quit"
  echo ""
}

scenario_normal() {
  reset_flagger_state
  start_traffic normal
  echo ""
  echo "  [Kiali Guide]"
  echo "  - Graph > Namespace: k1s0-system, k1s0-business, k1s0-service"
  echo "  - Graph Type: workload"
  echo "  - All edges should be green and task-server-primary should be present"
  echo "  - kubectl get canary task-server -n k1s0-service"
}

scenario_canary_promote() {
  reset_flagger_state
  bash "${SCRIPT_DIR}/start-flagger-rollout.sh" promote
  start_traffic normal
  echo ""
  echo "  [Flagger Guide]"
  echo "  - kubectl get canary task-server -n k1s0-service -w"
  echo "  - Expect weights to move in 20%% steps until promotion completes"
  echo ""
  echo "  [Kiali Guide]"
  echo "  - Graph Type: workload"
  echo "  - Watch task-bff traffic split across task-server and task-server-primary"
}

scenario_canary_rollback() {
  reset_flagger_state
  bash "${SCRIPT_DIR}/start-flagger-rollout.sh" rollback
  start_traffic normal
  echo ""
  echo "  [Flagger Guide]"
  echo "  - kubectl get canary task-server -n k1s0-service -w"
  echo "  - Expect failed checks and automatic rollback"
  echo ""
  echo "  [Grafana Guide] http://localhost:3200"
  echo "  - Open 'k1s0 Demo / Fault Injection Results'"
  echo "  - Error rate and latency should spike before rollback"
}

scenario_fault() {
  reset_manual_state
  bash "${SCRIPT_DIR}/run-fault-cronjob.sh"
  start_traffic normal
  echo ""
  echo "  [CronJob Guide]"
  echo "  - kubectl get cronjob fault-injection-test -n k1s0-service"
  echo "  - kubectl get job -n k1s0-service -l fault-injection-run=manual -w"
  echo "  - Fault window self-cleans after about 60s"
  echo ""
  echo "  [Grafana Guide] http://localhost:3200"
  echo "  - Open 'k1s0 Demo / Fault Injection Results'"
  echo "  - Error rate and P99 latency should rise while the Job is active"
}

scenario_tracing() {
  reset_flagger_state
  start_traffic normal
  echo ""
  echo "  [Jaeger Guide] http://localhost:16686"
  echo "  - Search > Service: task-bff.k1s0-service"
  echo "  - Open a trace to inspect the full span chain"
}

scenario_logs() {
  reset_flagger_state
  start_traffic normal
  echo ""
  echo "  [Grafana Guide] http://localhost:3200"
  echo "  - Open 'k1s0 Demo / k1s0 Log Explorer'"
  echo "  - Query example: {namespace=\"k1s0-service\",app=\"task-bff\"}"
}

scenario_kafka() {
  reset_flagger_state
  stop_traffic
  echo ""
  echo "  [Kafka Demo]"
  echo "  - Producer sends task-events every 5s to Kafka"
  echo "  - Consumer (project-master-processor group) reads events from Kafka"
  echo "  - Producer logs: kubectl logs -f deploy/kafka-demo-producer -n messaging"
  echo "  - Consumer logs: kubectl logs -f deploy/kafka-demo-consumer -n messaging"
}

echo ""
echo "Checking cluster connectivity..."
kubectl cluster-info --context "kind-${CLUSTER_NAME}" >/dev/null 2>&1 || {
  echo "ERROR: Cannot connect to kind-${CLUSTER_NAME} cluster."
  echo "Run 'bash setup.sh' first."
  exit 1
}
echo "Connected to cluster."

reset_flagger_state >/dev/null

while true; do
  show_menu
  read -rp "Select [1-8/q]: " choice
  case "${choice}" in
    1) scenario_normal ;;
    2) scenario_canary_promote ;;
    3) scenario_canary_rollback ;;
    4) scenario_fault ;;
    5) scenario_tracing ;;
    6) scenario_logs ;;
    7) scenario_kafka ;;
    8) reset_flagger_state
       start_traffic normal
       echo "  Reset complete. Flagger baseline restored." ;;
    q|Q) echo "Shutting down..."; exit 0 ;;
    *) echo "Invalid choice. Please select 1-8 or q." ;;
  esac
done
