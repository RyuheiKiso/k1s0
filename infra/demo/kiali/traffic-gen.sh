#!/usr/bin/env bash
# Generate traffic between services to produce Envoy metrics for Kiali.
# Uses istio-proxy sidecar's built-in curl to send requests.
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

END_TIME=$((SECONDS + DURATION))
COUNT=0

while [ $SECONDS -lt $END_TIME ]; do
  COUNT=$((COUNT + 1))
  echo "[$(date +%H:%M:%S)] Round ${COUNT}"

  # service Tier: order-bff -> order-server, graphql-gateway
  CANARY_HEADER=""
  if [ "${MODE}" = "canary" ]; then
    CANARY_HEADER="-H x-canary:true"
  fi

  kubectl exec "${ORDER_BFF}" -n k1s0-service -c istio-proxy -- \
    curl -s -o /dev/null -w "  order-bff -> order-server: %{http_code}\n" \
    ${CANARY_HEADER} http://order-server.k1s0-service.svc.cluster.local/ &

  kubectl exec "${ORDER_BFF}" -n k1s0-service -c istio-proxy -- \
    curl -s -o /dev/null -w "  order-bff -> graphql-gateway: %{http_code}\n" \
    http://graphql-gateway.k1s0-system.svc.cluster.local/ &

  # service Tier: order-server -> auth, config, saga, accounting
  kubectl exec "${ORDER_SERVER}" -n k1s0-service -c istio-proxy -- \
    curl -s -o /dev/null -w "  order-server -> auth: %{http_code}\n" \
    http://auth-server.k1s0-system.svc.cluster.local/ &

  kubectl exec "${ORDER_SERVER}" -n k1s0-service -c istio-proxy -- \
    curl -s -o /dev/null -w "  order-server -> config: %{http_code}\n" \
    http://config-server.k1s0-system.svc.cluster.local/ &

  kubectl exec "${ORDER_SERVER}" -n k1s0-service -c istio-proxy -- \
    curl -s -o /dev/null -w "  order-server -> saga: %{http_code}\n" \
    http://saga-server.k1s0-system.svc.cluster.local/ &

  kubectl exec "${ORDER_SERVER}" -n k1s0-service -c istio-proxy -- \
    curl -s -o /dev/null -w "  order-server -> accounting: %{http_code}\n" \
    http://accounting-server.k1s0-business.svc.cluster.local/ &

  # business Tier: accounting -> auth, config
  kubectl exec "${ACCOUNTING}" -n k1s0-business -c istio-proxy -- \
    curl -s -o /dev/null -w "  accounting -> auth: %{http_code}\n" \
    http://auth-server.k1s0-system.svc.cluster.local/ &

  kubectl exec "${ACCOUNTING}" -n k1s0-business -c istio-proxy -- \
    curl -s -o /dev/null -w "  accounting -> config: %{http_code}\n" \
    http://config-server.k1s0-system.svc.cluster.local/ &

  # system Tier: graphql-gateway -> auth, config
  kubectl exec "${GRAPHQL_GW}" -n k1s0-system -c istio-proxy -- \
    curl -s -o /dev/null -w "  graphql-gw -> auth: %{http_code}\n" \
    http://auth-server.k1s0-system.svc.cluster.local/ &

  kubectl exec "${GRAPHQL_GW}" -n k1s0-system -c istio-proxy -- \
    curl -s -o /dev/null -w "  graphql-gw -> config: %{http_code}\n" \
    http://config-server.k1s0-system.svc.cluster.local/ &

  # system Tier: saga -> auth, config
  kubectl exec "${SAGA}" -n k1s0-system -c istio-proxy -- \
    curl -s -o /dev/null -w "  saga -> auth: %{http_code}\n" \
    http://auth-server.k1s0-system.svc.cluster.local/ &

  kubectl exec "${SAGA}" -n k1s0-system -c istio-proxy -- \
    curl -s -o /dev/null -w "  saga -> config: %{http_code}\n" \
    http://config-server.k1s0-system.svc.cluster.local/ &

  # system Tier: auth -> config
  kubectl exec "${AUTH}" -n k1s0-system -c istio-proxy -- \
    curl -s -o /dev/null -w "  auth -> config: %{http_code}\n" \
    http://config-server.k1s0-system.svc.cluster.local/ &

  wait
  echo ""
  sleep "${INTERVAL}"
done

echo "=== Traffic generation complete (${COUNT} rounds) ==="
