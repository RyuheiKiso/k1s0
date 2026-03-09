#!/usr/bin/env bash
# Interactive Kiali demo script
# Switches between Istio traffic management scenarios in real-time
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLUSTER_NAME="${CLUSTER_NAME:-k1s0-demo}"
NAMESPACE="k1s0-service"
TRAFFIC_PID=""

cleanup() {
  if [ -n "${TRAFFIC_PID}" ] && kill -0 "${TRAFFIC_PID}" 2>/dev/null; then
    kill "${TRAFFIC_PID}" 2>/dev/null || true
    wait "${TRAFFIC_PID}" 2>/dev/null || true
  fi
}
trap cleanup EXIT

start_traffic() {
  local mode="${1:-normal}"
  # Stop existing traffic generator
  if [ -n "${TRAFFIC_PID}" ] && kill -0 "${TRAFFIC_PID}" 2>/dev/null; then
    kill "${TRAFFIC_PID}" 2>/dev/null || true
    wait "${TRAFFIC_PID}" 2>/dev/null || true
  fi

  if [ "${mode}" = "canary" ]; then
    bash "${SCRIPT_DIR}/traffic-gen.sh" 2 3600 canary &
  else
    bash "${SCRIPT_DIR}/traffic-gen.sh" 2 3600 &
  fi
  TRAFFIC_PID=$!
  echo "  Traffic generator started (PID: ${TRAFFIC_PID}, mode: ${mode})"
}

reset_scenarios() {
  echo ""
  echo "Resetting all scenarios..."
  kubectl delete vs order-server-canary order-server-mirror order-server-fault \
    -n "${NAMESPACE}" 2>/dev/null || true
  echo "  All scenario VirtualServices removed."
}

show_menu() {
  echo ""
  echo "==========================================="
  echo "       k1s0 Service Mesh Demo"
  echo "==========================================="
  echo ""
  echo " [Traffic Control]"
  echo "  1) Normal Traffic        -- 7 services, healthy communication"
  echo "  2) Canary Release        -- order-server v1:v2 = 90:10 weight split"
  echo "  3) Header Routing        -- x-canary: true routes to canary only"
  echo "  4) Traffic Mirroring     -- Copy 10% to canary (zero impact)"
  echo "  5) Fault Injection       -- 500ms delay (10%) + 503 error (5%)"
  echo ""
  echo " [Observability]"
  echo "  6) Distributed Tracing   -- Jaeger: view request trace chain"
  echo "  7) Log Aggregation       -- Grafana+Loki: search structured logs"
  echo "  8) Kafka Messaging       -- Async event flow visualization"
  echo ""
  echo " [Control]"
  echo "  9) Reset All             -- Remove all scenarios, back to normal"
  echo "  q) Quit"
  echo ""
}

scenario_normal() {
  reset_scenarios
  start_traffic normal
  echo ""
  echo "  [Kiali Guide]"
  echo "  - Graph > Namespace: k1s0-system, k1s0-business, k1s0-service"
  echo "  - Display > Traffic Animation ON"
  echo "  - All edges should be green (healthy 200 OK)"
  echo "  - Click any edge to see request rate and success %"
}

scenario_canary() {
  reset_scenarios
  echo "Applying canary VirtualService (90:10 weight split)..."
  kubectl apply -f "${SCRIPT_DIR}/scenarios/canary.yaml"
  start_traffic normal
  echo ""
  echo "  [Kiali Guide]"
  echo "  - Graph > order-server node splits into stable/canary"
  echo "  - Click order-server node > Traffic tab > verify 90:10 ratio"
  echo "  - Versioned app graph shows weight distribution"
}

scenario_header() {
  reset_scenarios
  echo "Applying canary VirtualService (header-based routing)..."
  kubectl apply -f "${SCRIPT_DIR}/scenarios/canary.yaml"
  start_traffic canary
  echo ""
  echo "  [Kiali Guide]"
  echo "  - Graph > order-server node splits into stable/canary"
  echo "  - Requests with x-canary:true header go directly to canary"
  echo "  - Traffic tab shows canary receiving header-routed requests"
}

scenario_mirror() {
  reset_scenarios
  echo "Applying mirror VirtualService (10% traffic copy)..."
  kubectl apply -f "${SCRIPT_DIR}/scenarios/mirror.yaml"
  start_traffic normal
  echo ""
  echo "  [Kiali Guide]"
  echo "  - Graph > Display > Traffic Animation ON"
  echo "  - Look for dashed line (mirror) to canary workload"
  echo "  - Mirrored traffic has no impact on client responses"
  echo "  - canary receives shadow copy for validation"
}

scenario_fault() {
  reset_scenarios
  echo "Applying fault injection (500ms delay 10% + 503 abort 5%)..."
  kubectl apply -f "${SCRIPT_DIR}/scenarios/fault-abort.yaml"
  start_traffic normal
  echo ""
  echo "  [Kiali Guide]"
  echo "  - Graph > edges to order-server turn red/orange (errors)"
  echo "  - Click order-server > Health > see error rate increase"
  echo "  - Some requests show elevated latency (500ms delay)"
  echo "  - 5% of requests return 503 Service Unavailable"
}

scenario_tracing() {
  reset_scenarios
  start_traffic normal
  echo ""
  echo "  [Jaeger Guide] http://localhost:16686"
  echo "  - Search > Service: order-bff.k1s0-service"
  echo "  - Click 'Find Traces' to see recent traces"
  echo "  - Open a trace to see the full span chain:"
  echo "    order-bff -> order-server -> auth-server -> config-server"
  echo "  - Each span shows latency, status code, and Envoy metadata"
  echo ""
  echo "  [Kiali Guide]"
  echo "  - Click any service node > Traces tab"
  echo "  - Kiali now links directly to Jaeger traces"
}

scenario_logs() {
  reset_scenarios
  start_traffic normal
  echo ""
  echo "  [Grafana Guide] http://localhost:3200"
  echo "  - Explore > Select 'Loki' datasource"
  echo "  - Query: {namespace=\"k1s0-service\"}"
  echo "  - Click 'Live' to stream logs in real-time"
  echo "  - Filter by app: {app=\"order-server\"}"
  echo ""
  echo "  [Dashboards]"
  echo "  - k1s0 Demo > Service Mesh Overview: RED metrics (Rate/Error/Duration)"
  echo "  - k1s0 Demo > Log Explorer: Logs by namespace with volume chart"
}

scenario_kafka() {
  reset_scenarios
  if [ -n "${TRAFFIC_PID}" ] && kill -0 "${TRAFFIC_PID}" 2>/dev/null; then
    kill "${TRAFFIC_PID}" 2>/dev/null || true
    wait "${TRAFFIC_PID}" 2>/dev/null || true
    TRAFFIC_PID=""
  fi
  echo ""
  echo "  [Kafka Demo]"
  echo "  - Producer sends order-events every 5s to Kafka"
  echo "  - Consumer (accounting-processor group) reads events from Kafka"
  echo "  - Check producer logs:"
  echo "    kubectl logs -f deploy/kafka-demo-producer -n messaging"
  echo "  - Check consumer logs:"
  echo "    kubectl logs -f deploy/kafka-demo-consumer -n messaging"
  echo ""
  echo "  [Kiali Guide]"
  echo "  - Add 'messaging' namespace to graph view"
  echo "  - See TCP flow: kafka-demo-producer -> kafka"
  echo "  - See TCP flow: kafka-demo-consumer -> kafka"
}

# Main loop
echo ""
echo "Checking cluster connectivity..."
kubectl cluster-info --context "kind-${CLUSTER_NAME}" > /dev/null 2>&1 || {
  echo "ERROR: Cannot connect to kind-${CLUSTER_NAME} cluster."
  echo "Run 'bash setup.sh' first."
  exit 1
}
echo "Connected to cluster."

# Ensure canary deployment exists
echo "Ensuring canary deployment is ready..."
kubectl apply -f "${SCRIPT_DIR}/manifests/04-canary-deploy.yaml"
kubectl wait --for=condition=ready pod -l app=order-server,version=canary \
  -n "${NAMESPACE}" --timeout=60s 2>/dev/null || true

while true; do
  show_menu
  read -rp "Select [1-9/q]: " choice
  case "${choice}" in
    1) scenario_normal ;;
    2) scenario_canary ;;
    3) scenario_header ;;
    4) scenario_mirror ;;
    5) scenario_fault ;;
    6) scenario_tracing ;;
    7) scenario_logs ;;
    8) scenario_kafka ;;
    9) reset_scenarios
       start_traffic normal
       echo "  Reset complete. Normal traffic resumed." ;;
    q|Q) echo "Shutting down..."; exit 0 ;;
    *) echo "Invalid choice. Please select 1-9 or q." ;;
  esac
done
