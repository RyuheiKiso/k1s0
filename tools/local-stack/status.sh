#!/usr/bin/env bash
#
# tools/local-stack/status.sh — 配備状態のサマリ表示

set -euo pipefail

KIND_CLUSTER_NAME="k1s0-local"

log() { printf '\033[36m[local-stack]\033[0m %s\n' "$*"; }

if ! kind get clusters 2>/dev/null | grep -q "^${KIND_CLUSTER_NAME}$"; then
    log "kind cluster '${KIND_CLUSTER_NAME}' は存在しません。tools/local-stack/up.sh で起動"
    exit 0
fi

log "## kind cluster"
kind get clusters
log ""
log "## nodes"
kubectl get nodes -o wide
log ""
log "## namespaces (k1s0.io/layer label 付き)"
kubectl get ns -L k1s0.io/layer | awk 'NR==1 || $4 != "<none>"'
log ""
log "## not-Running pods (まだ起動中 or 失敗)"
kubectl get pods -A --field-selector=status.phase!=Running,status.phase!=Succeeded || true
log ""
log "## helm releases"
helm ls -A
log ""
log "## NodePort services (host からアクセス可能)"
kubectl get svc -A -o jsonpath='{range .items[?(@.spec.type=="NodePort")]}{.metadata.namespace}{"/"}{.metadata.name}{":"}{.spec.ports[*].nodePort}{"\n"}{end}'
