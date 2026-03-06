#!/usr/bin/env bash
# Generate traffic between services to produce Envoy metrics for Kiali.
# Uses istio-proxy sidecar's built-in curl to send requests.
# Propagates B3 trace headers to create multi-span traces for Jaeger Dependencies.
set -euo pipefail

INTERVAL="${1:-2}"
DURATION="${2:-300}"
MODE="${3:-normal}"

echo "=== Traffic Generator ==="
echo "Interval: ${INTERVAL}s | Duration: ${DURATION}s | Mode: ${MODE}"
echo "Press Ctrl+C to stop"
echo ""

# Get pod names for traffic sources
ORDER_BFF=$(kubectl get pod -n k1s0-service -l app=order-bff -o jsonpath='{.items[0].metadata.name}')
ORDER_SERVER=$(kubectl get pod -n k1s0-service -l app=order-server -o jsonpath='{.items[0].metadata.name}')
ACCOUNTING=$(kubectl get pod -n k1s0-business -l app=accounting-server -o jsonpath='{.items[0].metadata.name}')
GRAPHQL_GW=$(kubectl get pod -n k1s0-system -l app=graphql-gateway -o jsonpath='{.items[0].metadata.name}')
SAGA=$(kubectl get pod -n k1s0-system -l app=saga-server -o jsonpath='{.items[0].metadata.name}')
AUTH=$(kubectl get pod -n k1s0-system -l app=auth-server -o jsonpath='{.items[0].metadata.name}')

echo "Pods found:"
echo "  order-bff:         ${ORDER_BFF}"
echo "  order-server:      ${ORDER_SERVER}"
echo "  accounting-server: ${ACCOUNTING}"
echo "  graphql-gateway:   ${GRAPHQL_GW}"
echo "  saga-server:       ${SAGA}"
echo "  auth-server:       ${AUTH}"
echo ""

# Generate random hex string (16 chars = 8 bytes)
gen_id() {
  cat /dev/urandom | tr -dc 'a-f0-9' | head -c 16
}

# Send request with B3 trace propagation headers
# Usage: send_req POD NAMESPACE TRACE_ID PARENT_SPAN_ID URL LABEL [EXTRA_HEADERS]
send_req() {
  local pod="$1" ns="$2" trace_id="$3" parent_span="$4" url="$5" label="$6"
  local span_id
  span_id=$(gen_id)
  local extra="${7:-}"

  kubectl exec "${pod}" -n "${ns}" -c istio-proxy -- \
    curl -s -o /dev/null -w "  ${label}: %{http_code}\n" \
    -H "x-b3-traceid:${trace_id}" \
    -H "x-b3-spanid:${span_id}" \
    -H "x-b3-parentspanid:${parent_span}" \
    -H "x-b3-sampled:1" \
    ${extra} "${url}" &
}

END_TIME=$((SECONDS + DURATION))
COUNT=0

while [ $SECONDS -lt $END_TIME ]; do
  COUNT=$((COUNT + 1))
  echo "[$(date +%H:%M:%S)] Round ${COUNT}"

  CANARY_HEADER=""
  if [ "${MODE}" = "canary" ]; then
    CANARY_HEADER="-H x-canary:true"
  fi

  # --- Trace 1: order-bff -> order-server -> {auth, config, saga, accounting} ---
  T1=$(gen_id)$(gen_id)  # 32-char trace ID
  S1_BFF=$(gen_id)

  # order-bff -> order-server
  send_req "${ORDER_BFF}" k1s0-service "${T1}" "${S1_BFF}" \
    "http://order-server.k1s0-service.svc.cluster.local/" \
    "order-bff -> order-server" "${CANARY_HEADER}"

  # order-server -> auth
  S1_OS=$(gen_id)
  send_req "${ORDER_SERVER}" k1s0-service "${T1}" "${S1_OS}" \
    "http://auth-server.k1s0-system.svc.cluster.local/" \
    "order-server -> auth"

  # order-server -> config
  send_req "${ORDER_SERVER}" k1s0-service "${T1}" "${S1_OS}" \
    "http://config-server.k1s0-system.svc.cluster.local/" \
    "order-server -> config"

  # order-server -> saga
  send_req "${ORDER_SERVER}" k1s0-service "${T1}" "${S1_OS}" \
    "http://saga-server.k1s0-system.svc.cluster.local/" \
    "order-server -> saga"

  # order-server -> accounting
  send_req "${ORDER_SERVER}" k1s0-service "${T1}" "${S1_OS}" \
    "http://accounting-server.k1s0-business.svc.cluster.local/" \
    "order-server -> accounting"

  # --- Trace 2: order-bff -> graphql-gateway -> {auth, config} ---
  T2=$(gen_id)$(gen_id)
  S2_BFF=$(gen_id)

  # order-bff -> graphql-gateway
  send_req "${ORDER_BFF}" k1s0-service "${T2}" "${S2_BFF}" \
    "http://graphql-gateway.k1s0-system.svc.cluster.local/" \
    "order-bff -> graphql-gateway"

  # graphql-gateway -> auth
  S2_GW=$(gen_id)
  send_req "${GRAPHQL_GW}" k1s0-system "${T2}" "${S2_GW}" \
    "http://auth-server.k1s0-system.svc.cluster.local/" \
    "graphql-gw -> auth"

  # graphql-gateway -> config
  send_req "${GRAPHQL_GW}" k1s0-system "${T2}" "${S2_GW}" \
    "http://config-server.k1s0-system.svc.cluster.local/" \
    "graphql-gw -> config"

  # --- Trace 3: accounting -> {auth, config} ---
  T3=$(gen_id)$(gen_id)
  S3_ACC=$(gen_id)

  send_req "${ACCOUNTING}" k1s0-business "${T3}" "${S3_ACC}" \
    "http://auth-server.k1s0-system.svc.cluster.local/" \
    "accounting -> auth"

  send_req "${ACCOUNTING}" k1s0-business "${T3}" "${S3_ACC}" \
    "http://config-server.k1s0-system.svc.cluster.local/" \
    "accounting -> config"

  # --- Trace 4: saga -> {auth, config} ---
  T4=$(gen_id)$(gen_id)
  S4_SAGA=$(gen_id)

  send_req "${SAGA}" k1s0-system "${T4}" "${S4_SAGA}" \
    "http://auth-server.k1s0-system.svc.cluster.local/" \
    "saga -> auth"

  send_req "${SAGA}" k1s0-system "${T4}" "${S4_SAGA}" \
    "http://config-server.k1s0-system.svc.cluster.local/" \
    "saga -> config"

  # --- Trace 5: auth -> config ---
  T5=$(gen_id)$(gen_id)
  S5_AUTH=$(gen_id)

  send_req "${AUTH}" k1s0-system "${T5}" "${S5_AUTH}" \
    "http://config-server.k1s0-system.svc.cluster.local/" \
    "auth -> config"

  wait
  echo ""
  sleep "${INTERVAL}"
done

echo "=== Traffic generation complete (${COUNT} rounds) ==="
