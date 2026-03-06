#!/usr/bin/env bash
# Kiali demo environment setup script
# Prerequisites: docker, kind, istioctl, kubectl, helm
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLUSTER_NAME="k1s0-demo"

echo "=== k1s0 Kiali Demo Environment Setup ==="

# 1. Check prerequisites
for cmd in docker kind istioctl kubectl helm; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "ERROR: $cmd is not installed. Please install it first."
    exit 1
  fi
done

echo "All prerequisites found."

# 2. Create kind cluster (skip if already exists)
if kind get clusters 2>/dev/null | grep -q "^${CLUSTER_NAME}$"; then
  echo "Cluster '${CLUSTER_NAME}' already exists. Skipping creation."
else
  echo "Creating kind cluster '${CLUSTER_NAME}'..."
  kind create cluster --config "${SCRIPT_DIR}/kind-config.yaml"
fi

kubectl cluster-info --context "kind-${CLUSTER_NAME}"

# 3. Install Istio (minimal profile + tracing)
echo "Installing Istio..."
istioctl install --set profile=minimal \
  --set meshConfig.defaultConfig.holdApplicationUntilProxyStarts=true \
  --set meshConfig.enableTracing=true \
  --set meshConfig.defaultConfig.tracing.zipkin.address=otel-collector.observability.svc.cluster.local:9411 \
  --set meshConfig.defaultConfig.tracing.sampling=100 \
  -y

# Wait for istiod
echo "Waiting for istiod to be ready..."
kubectl wait --for=condition=available deployment/istiod -n istio-system --timeout=120s

# 4. Apply manifests
echo "Applying namespaces..."
kubectl apply -f "${SCRIPT_DIR}/manifests/00-namespaces.yaml"

echo "Applying stub services..."
kubectl apply -f "${SCRIPT_DIR}/manifests/01-stub-services.yaml"

echo "Applying Istio resources..."
kubectl apply -f "${SCRIPT_DIR}/manifests/02-istio.yaml"

echo "Applying canary deployment..."
kubectl apply -f "${SCRIPT_DIR}/manifests/04-canary-deploy.yaml"

echo "Applying Prometheus..."
kubectl apply -f "${SCRIPT_DIR}/manifests/03-prometheus.yaml"

echo "Applying OTel Collector..."
kubectl apply -f "${SCRIPT_DIR}/manifests/05-otel-collector.yaml"

echo "Applying Jaeger..."
kubectl apply -f "${SCRIPT_DIR}/manifests/06-jaeger.yaml"

echo "Applying Loki..."
kubectl apply -f "${SCRIPT_DIR}/manifests/07-loki.yaml"

echo "Applying Promtail..."
kubectl apply -f "${SCRIPT_DIR}/manifests/08-promtail.yaml"

echo "Creating Grafana dashboards ConfigMap..."
kubectl create configmap grafana-dashboards \
  --from-file="${SCRIPT_DIR}/dashboards/" \
  -n observability --dry-run=client -o yaml | kubectl apply -f -

echo "Applying Grafana..."
kubectl apply -f "${SCRIPT_DIR}/manifests/09-grafana.yaml"

echo "Applying Kafka..."
kubectl apply -f "${SCRIPT_DIR}/manifests/10-kafka.yaml"

# 5. Install Kiali via Helm
echo "Installing Kiali..."
helm repo add kiali https://kiali.org/helm-charts 2>/dev/null || true
helm repo update kiali

if helm status kiali-server -n istio-system &>/dev/null; then
  echo "Kiali already installed. Upgrading..."
  helm upgrade kiali-server kiali/kiali-server \
    -n istio-system \
    -f "${SCRIPT_DIR}/values/kiali-values.yaml"
else
  helm install kiali-server kiali/kiali-server \
    -n istio-system \
    -f "${SCRIPT_DIR}/values/kiali-values.yaml"
fi

# 6. Wait for all pods
echo "Waiting for all pods to be ready..."

for ns in k1s0-system k1s0-business k1s0-service; do
  echo "  Waiting for pods in ${ns}..."
  kubectl wait --for=condition=ready pod --all -n "$ns" --timeout=180s
done

echo "  Waiting for observability stack..."
kubectl wait --for=condition=ready pod -l app=prometheus -n observability --timeout=120s
kubectl wait --for=condition=ready pod -l app=otel-collector -n observability --timeout=120s
kubectl wait --for=condition=ready pod -l app=jaeger -n observability --timeout=120s
kubectl wait --for=condition=ready pod -l app=loki -n observability --timeout=120s
kubectl wait --for=condition=ready pod -l app=grafana -n observability --timeout=120s

echo "  Waiting for Kafka..."
kubectl wait --for=condition=ready pod -l app=kafka -n messaging --timeout=180s

echo "  Waiting for Kiali..."
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=kiali -n istio-system --timeout=120s

# 7. Start port-forward for Kiali access
echo "Starting port-forward for Kiali (localhost:20001)..."
kubectl port-forward svc/kiali -n istio-system 20001:20001 &>/dev/null &
KIALI_PF_PID=$!
echo "${KIALI_PF_PID}" > "${SCRIPT_DIR}/.kiali-pf.pid"
sleep 2

# Verify Kiali is accessible
if curl -s -o /dev/null -w "" http://localhost:20001/kiali/ 2>/dev/null; then
  echo "Kiali is accessible."
else
  echo "WARNING: Kiali may not be accessible yet. Try manually:"
  echo "  kubectl port-forward svc/kiali -n istio-system 20001:20001"
fi

# 8. Summary
echo ""
echo "=== Setup Complete ==="
echo ""
echo "Dashboards:"
echo "  Kiali:    http://localhost:20001/kiali"
echo "  Jaeger:   http://localhost:16686"
echo "  Grafana:  http://localhost:3000"
echo "  Demo UI:  http://localhost:5173  (run 'cd ui && npm install && npm run dev')"
echo ""
echo "Next steps:"
echo "  Option A: React Demo UI (recommended for demos)"
echo "    cd ${SCRIPT_DIR}/ui && npm install && npm run dev"
echo "    Open http://localhost:5173"
echo ""
echo "  Option B: CLI demo (interactive terminal)"
echo "    bash ${SCRIPT_DIR}/demo.sh"
echo ""
echo "To clean up: bash ${SCRIPT_DIR}/teardown.sh"
