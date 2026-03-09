#!/usr/bin/env bash
# Kiali demo environment setup script
# Prerequisites: docker, kind, istioctl, kubectl, helm, curl
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
LOCAL_TOOLS_BIN="${REPO_ROOT}/.tools/bin"
if [ -d "${LOCAL_TOOLS_BIN}" ]; then
  PATH="${LOCAL_TOOLS_BIN}:${PATH}"
fi

CLUSTER_NAME="${CLUSTER_NAME:-k1s0-demo}"
RECREATE_CLUSTER="${RECREATE_CLUSTER:-0}"
REQUIRED_ISTIO_MINOR="${REQUIRED_ISTIO_MINOR:-1.24}"

echo "=== k1s0 Kiali Demo Environment Setup ==="

cluster_has_required_mapping() {
  local host_port="${1}"
  local container_port="${2}"
  docker port "${CLUSTER_NAME}-control-plane" "${container_port}/tcp" 2>/dev/null | grep -Eq ":${host_port}$"
}

# 1. Check prerequisites
for cmd in docker kind istioctl kubectl helm curl; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "ERROR: $cmd is not installed. Please install it first."
    exit 1
  fi
done

echo "All prerequisites found."

ISTIOCTL_VERSION="$(istioctl version --remote=false 2>/dev/null | awk '/client version:/ {print $3; exit}')"
if [ -z "${ISTIOCTL_VERSION}" ]; then
  echo "ERROR: Failed to detect istioctl client version."
  exit 1
fi

if [[ "${ISTIOCTL_VERSION}" != "${REQUIRED_ISTIO_MINOR}."* ]]; then
  echo "ERROR: istioctl ${REQUIRED_ISTIO_MINOR}.x is required, but found ${ISTIOCTL_VERSION}."
  echo "Update PATH or set it explicitly before running setup.sh."
  exit 1
fi

echo "Using istioctl ${ISTIOCTL_VERSION}."

# 2. Create kind cluster (skip if already exists)
if kind get clusters 2>/dev/null | grep -q "^${CLUSTER_NAME}$"; then
  if [ "${RECREATE_CLUSTER}" = "1" ]; then
    echo "Recreating cluster '${CLUSTER_NAME}' to refresh host port mappings..."
    kind delete cluster --name "${CLUSTER_NAME}"
    kind create cluster --name "${CLUSTER_NAME}" --config "${SCRIPT_DIR}/kind-config.yaml"
  else
    missing_mappings=()
    for mapping in \
      "20001:30001" \
      "16686:30686" \
      "4318:30418" \
      "4317:30417" \
      "9090:30909" \
      "3200:30300"; do
      host_port="${mapping%%:*}"
      container_port="${mapping##*:}"
      if ! cluster_has_required_mapping "${host_port}" "${container_port}"; then
        missing_mappings+=("${host_port}->${container_port}")
      fi
    done

    if [ "${#missing_mappings[@]}" -gt 0 ]; then
      echo "ERROR: Cluster '${CLUSTER_NAME}' already exists but is missing required host port mappings:"
      for mapping in "${missing_mappings[@]}"; do
        echo "  - ${mapping}"
      done
      echo "Run 'RECREATE_CLUSTER=1 bash ${SCRIPT_DIR}/setup.sh' to recreate it with the current config."
      exit 1
    fi

    echo "Cluster '${CLUSTER_NAME}' already exists with required host port mappings. Skipping creation."
  fi
else
  echo "Creating kind cluster '${CLUSTER_NAME}'..."
  kind create cluster --name "${CLUSTER_NAME}" --config "${SCRIPT_DIR}/kind-config.yaml"
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

# 7. Verify externally mapped endpoints
sleep 2

if curl -s -o /dev/null -w "" http://localhost:20001/kiali/ 2>/dev/null; then
  echo "Kiali is accessible."
else
  echo "WARNING: Kiali may not be accessible yet at http://localhost:20001/kiali"
fi

# 8. Summary
echo ""
echo "=== Setup Complete ==="
echo ""
echo "Dashboards:"
echo "  Kiali:    http://localhost:20001/kiali"
echo "  Jaeger:   http://localhost:16686"
echo "  Grafana:  http://localhost:3200"
echo "  Prometheus API: http://localhost:9090"
echo "  Jaeger OTLP HTTP: http://localhost:4318"
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
echo "If you updated kind-config host ports on an existing cluster:"
echo "  RECREATE_CLUSTER=1 bash ${SCRIPT_DIR}/setup.sh"
echo ""
echo "To clean up: bash ${SCRIPT_DIR}/teardown.sh"
