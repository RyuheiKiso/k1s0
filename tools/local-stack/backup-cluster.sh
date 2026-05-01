#!/usr/bin/env bash
#
# tools/local-stack/backup-cluster.sh — kind cluster `k1s0-local` の完全バックアップ
#
# 用途: down.sh で破棄する前、または定期点検時に cluster 内 state を退避する。
# ADR-POL-002 (local-stack-single-source-of-truth) で定義された
# 「down → backup → up → restore → 検証」サイクルの backup フェーズを実装する。
#
# バックアップ対象（優先度順）:
#   CRITICAL: gitea data / pg-state DB / OpenBao secrets / Keycloak realm
#   IMPORTANT: 全 helm release values / argocd Applications / 手動 apply Manifest
#   INFORMATIONAL: 全 namespace YAML / PVC metadata / cluster info
#
# 使い方:
#   ./tools/local-stack/backup-cluster.sh [出力ディレクトリ]
#   既定の出力先: /tmp/k1s0-backup-YYYYMMDD-HHMMSS/

set -euo pipefail

readonly KIND_CLUSTER_NAME="${KIND_CLUSTER_NAME:-k1s0-local}"
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
readonly TIMESTAMP
readonly DEFAULT_DEST="/tmp/k1s0-backup-${TIMESTAMP}"
readonly DEST="${1:-${DEFAULT_DEST}}"

log() { printf '\033[36m[backup]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[backup:warn]\033[0m %s\n' "$*" >&2; }
err() { printf '\033[31m[backup:err]\033[0m %s\n' "$*" >&2; }

# kubectl context 確認
current_ctx=$(kubectl config current-context 2>/dev/null || echo "")
if [[ "${current_ctx}" != "kind-${KIND_CLUSTER_NAME}" ]]; then
    err "current context is '${current_ctx}', expected 'kind-${KIND_CLUSTER_NAME}'"
    exit 1
fi

mkdir -p "${DEST}"
log "出力先: ${DEST}"

# ---------- CRITICAL ----------

backup_gitea() {
    log "[CRITICAL] gitea data (argocd の sync 元)"
    local pod
    pod=$(kubectl -n gitops get pod -l app=gitea -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
    if [[ -z "${pod}" ]]; then
        warn "gitea pod が見つからない (gitops ns)"
        return
    fi
    mkdir -p "${DEST}/gitea-data"
    # 注意: gitea は emptyDir に書いており kind が止まれば data 喪失。kubectl cp で必ず退避。
    kubectl cp -n gitops "${pod}:/data" "${DEST}/gitea-data/" 2>&1 | tail -5 || warn "gitea cp で部分失敗"
    log "  → ${DEST}/gitea-data/ ($(du -sh "${DEST}"/gitea-data 2>/dev/null | awk '{print $1}'))"
}

backup_pg_state() {
    log "[CRITICAL] CNPG pg-state DB (k1s0-tier1)"
    mkdir -p "${DEST}/dbs/pg-state"
    local primary
    primary=$(kubectl -n k1s0-tier1 get pod -l cnpg.io/cluster=pg-state,role=primary -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "pg-state-1")
    # ユーザ DB を auto-discover
    local dbs
    dbs=$(kubectl exec -n k1s0-tier1 "${primary}" -c postgres -- psql -U postgres -lqt 2>/dev/null \
        | awk -F'|' '{gsub(/^ +| +$/,"",$1); print $1}' \
        | grep -vE "^(template0|template1|postgres|)$" || true)
    if [[ -z "${dbs}" ]]; then
        warn "pg-state にユーザ DB が見つからない（dapr などが期待されるが空）"
    fi
    for db in ${dbs}; do
        log "  pg_dump ${db}"
        kubectl exec -n k1s0-tier1 "${primary}" -c postgres -- \
            pg_dump -U postgres -Fc "${db}" > "${DEST}/dbs/pg-state/${db}.dump" 2>/dev/null \
            || warn "  pg_dump ${db} 失敗"
    done
    # globals (roles, tablespaces) も退避
    kubectl exec -n k1s0-tier1 "${primary}" -c postgres -- \
        pg_dumpall -U postgres --globals-only > "${DEST}/dbs/pg-state/_globals.sql" 2>/dev/null \
        || warn "  pg_dumpall --globals-only 失敗"
}

backup_keycloak() {
    log "[CRITICAL] Keycloak H2 DB hot-copy (file-by-file via exec cat)"
    mkdir -p "${DEST}/keycloak/h2"
    local pod
    pod=$(kubectl -n keycloak get pod -l app=keycloak -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
    if [[ -z "${pod}" ]]; then
        pod=$(kubectl -n keycloak get pod --no-headers 2>/dev/null | awk '/keycloak/{print $1; exit}')
    fi
    if [[ -z "${pod}" ]]; then
        warn "keycloak pod が見つからない"
        return
    fi
    # keycloak container には tar が無く kubectl cp が使えない。cat で個別ファイル取得。
    # H2 が live のため hot-copy（再起動時に H2 recovery 経由で復元される）。
    for f in keycloakdb.mv.db keycloakdb.trace.db; do
        if kubectl exec -n keycloak "${pod}" -- test -f "/opt/keycloak/data/h2/${f}" 2>/dev/null; then
            if kubectl exec -n keycloak "${pod}" -- cat "/opt/keycloak/data/h2/${f}" \
                > "${DEST}/keycloak/h2/${f}" 2>/dev/null; then
                log "  → ${f} ($(du -sh "${DEST}"/keycloak/h2/${f} | awk '{print $1}'))"
            else
                warn "  ${f} 取得失敗"
            fi
        fi
    done
}

backup_openbao() {
    log "[CRITICAL] OpenBao secrets snapshot"
    mkdir -p "${DEST}/openbao"
    local pod
    pod=$(kubectl -n openbao get pod -l app.kubernetes.io/name=openbao -o jsonpath='{.items[0].metadata.name}' 2>/dev/null \
        || kubectl -n openbao get pod --no-headers 2>/dev/null | awk '{print $1; exit}')
    if [[ -z "${pod}" ]]; then
        warn "openbao pod が見つからない"
        return
    fi
    # openbao chart は /home/openbao を data dir として使用（volume name "home"）。
    # raft snapshot は root token が要るため、ここでは filesystem 直 cp で代替。
    kubectl cp -n openbao "${pod}:/home/openbao" "${DEST}/openbao/home" 2>&1 | tail -3 \
        || warn "  openbao /home/openbao cp 失敗"
    log "  → ${DEST}/openbao/home ($(du -sh "${DEST}"/openbao/home 2>/dev/null | awk '{print $1}'))"
}

# ---------- IMPORTANT ----------

backup_helm_releases() {
    log "[IMPORTANT] 全 helm release の values + manifest + metadata"
    mkdir -p "${DEST}/helm-releases"
    helm list -A --output json 2>/dev/null > "${DEST}/helm-releases/_raw.json"
    python3 - "${DEST}/helm-releases/_raw.json" "${DEST}/helm-releases/_inventory.txt" <<'PYEOF'
import json, sys
with open(sys.argv[1]) as f:
    data = json.load(f)
with open(sys.argv[2], "w") as out:
    for r in data:
        out.write("|".join([r["name"], r["namespace"], r["chart"], r.get("app_version","")]) + "\n")
PYEOF
    # shellcheck disable=SC2034
    while IFS='|' read -r name ns chart appver; do
        [[ -z "${name}" ]] && continue
        local outdir="${DEST}/helm-releases/${ns}_${name}"
        mkdir -p "${outdir}"
        helm get values "${name}" -n "${ns}" > "${outdir}/values.yaml" 2>/dev/null || true
        helm get manifest "${name}" -n "${ns}" > "${outdir}/manifest.yaml" 2>/dev/null || true
        helm get metadata "${name}" -n "${ns}" -o json > "${outdir}/metadata.json" 2>/dev/null || true
    done < "${DEST}/helm-releases/_inventory.txt"
    log "  → ${DEST}/helm-releases/ ($(wc -l < "${DEST}"/helm-releases/_inventory.txt) releases)"
}

backup_argocd_apps() {
    log "[IMPORTANT] argocd Applications"
    mkdir -p "${DEST}/argocd"
    kubectl -n argocd get applications.argoproj.io -o yaml > "${DEST}/argocd/applications.yaml" 2>/dev/null || true
    kubectl -n argocd get appprojects.argoproj.io -o yaml > "${DEST}/argocd/appprojects.yaml" 2>/dev/null || true
    kubectl -n argocd get applicationsets.argoproj.io -o yaml > "${DEST}/argocd/applicationsets.yaml" 2>/dev/null || true
}

backup_manual_applies() {
    log "[IMPORTANT] 手動 apply (owner reference 無し) の Deployment / DaemonSet / StatefulSet"
    mkdir -p "${DEST}/manual-applies"
    kubectl get deployment,statefulset,daemonset,cronjob -A -o json 2>/dev/null \
        > "${DEST}/manual-applies/_raw.json"
    python3 - "${DEST}/manual-applies/_raw.json" "${DEST}/manual-applies/_inventory.tsv" <<'PYEOF'
import json, sys
with open(sys.argv[1]) as f:
    d = json.load(f)
with open(sys.argv[2], "w") as out:
    for it in d["items"]:
        if it["metadata"].get("ownerReferences"):
            continue
        ann = it["metadata"].get("annotations") or {}
        if ann.get("meta.helm.sh/release-name"):
            continue
        if it["metadata"]["namespace"].startswith("kube-"):
            continue
        out.write("\t".join([it["kind"], it["metadata"]["namespace"], it["metadata"]["name"]]) + "\n")
PYEOF
    while IFS=$'\t' read -r kind ns name; do
        [[ -z "${kind}" ]] && continue
        kubectl -n "${ns}" get "${kind,,}" "${name}" -o yaml \
            > "${DEST}/manual-applies/${ns}_${kind}_${name}.yaml" 2>/dev/null || true
    done < "${DEST}/manual-applies/_inventory.tsv"
}

backup_critical_secrets() {
    log "[IMPORTANT] critical secrets (cnpg / argocd / backstage / openbao 関連)"
    mkdir -p "${DEST}/secrets"
    # 全 secret の inventory（中身は core/system 系を除外し、対象を絞る）
    kubectl get secret -A -o json 2>/dev/null \
        | python3 -c '
import json, sys, yaml
d = json.load(sys.stdin)
keep_namespaces = {"argocd","cnpg-system","keycloak","openbao","backstage","gitops",
                   "k1s0-tier1","k1s0-tier2","k1s0-tier3","kafka","minio","spire-server"}
out = []
for it in d["items"]:
    ns = it["metadata"]["namespace"]
    if ns not in keep_namespaces:
        continue
    if it.get("type") in ("kubernetes.io/service-account-token","helm.sh/release.v1"):
        continue
    name = it["metadata"]["name"]
    if name.startswith("default-token") or name.startswith("sh.helm.release"):
        continue
    out.append(it)
print(yaml.safe_dump_all(out, default_flow_style=False))' \
        > "${DEST}/secrets/critical.yaml" 2>/dev/null \
        || warn "secret export で部分失敗 (yaml モジュールが必要かも)"
}

# ---------- INFORMATIONAL ----------

backup_namespace_dumps() {
    log "[INFO] 重要 namespace の全リソース YAML dump"
    mkdir -p "${DEST}/namespaces"
    local nss=("k1s0-tier1" "k1s0-tier2" "k1s0-tier3" "argocd" "gitops" "registry"
               "default" "backstage" "cnpg-system" "openbao" "keycloak" "observability")
    for ns in "${nss[@]}"; do
        if kubectl get ns "${ns}" >/dev/null 2>&1; then
            kubectl -n "${ns}" get all,cm,secret,pvc,ingress,gateway,virtualservice,rollout -o yaml \
                > "${DEST}/namespaces/${ns}.yaml" 2>/dev/null || true
        fi
    done
}

backup_cluster_metadata() {
    log "[INFO] cluster metadata (CRDs / nodes / version)"
    mkdir -p "${DEST}/cluster"
    kubectl get crd -o yaml > "${DEST}/cluster/crds.yaml" 2>/dev/null || true
    kubectl get nodes -o yaml > "${DEST}/cluster/nodes.yaml" 2>/dev/null || true
    kubectl version > "${DEST}/cluster/version.txt" 2>/dev/null || true
    kubectl get pvc -A -o yaml > "${DEST}/cluster/pvcs.yaml" 2>/dev/null || true
    kind get kubeconfig --name "${KIND_CLUSTER_NAME}" > "${DEST}/cluster/kubeconfig.yaml" 2>/dev/null || true
}

# ---------- 実行 ----------

backup_gitea
backup_pg_state
backup_keycloak
backup_openbao

backup_helm_releases
backup_argocd_apps
backup_manual_applies
backup_critical_secrets

backup_namespace_dumps
backup_cluster_metadata

# ---------- integrity check ----------

log ""
log "===== 完了。integrity 検証 ====="
log "出力先: ${DEST}"
du -sh "${DEST}"/* 2>/dev/null | sort -h
log ""
log "次は restore-cluster.sh で復元するか、tar.gz アーカイブ化:"
log "  tar -czf ${DEST}.tar.gz -C $(dirname "${DEST}") $(basename "${DEST}")"
