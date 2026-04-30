#!/usr/bin/env bash
#
# tools/local-stack/verify-cluster.sh — ADR-POL-002 SoT 整合の検証
#
# 用途:
#   1. up.sh 実行後の cluster 状態が canonical 集合と一致しているか確認
#   2. drift-check workflow の sync-check と integration を local で再現
#   3. backup → down → up → restore サイクルの完了確認
#
# Exit code:
#   0 = 全 check pass / 1 = 1 件以上の check 失敗 / 2 = pre-flight 失敗

set -uo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
KIND_CLUSTER_NAME="${KIND_CLUSTER_NAME:-k1s0-local}"
EXPECTED_CONTEXT="kind-${KIND_CLUSTER_NAME}"

PASS=0
FAIL=0
WARN=0

ok()   { printf '\033[32m  ✓\033[0m %s\n' "$*"; PASS=$((PASS+1)); }
ng()   { printf '\033[31m  ✗\033[0m %s\n' "$*" >&2; FAIL=$((FAIL+1)); }
warn() { printf '\033[33m  !\033[0m %s\n' "$*" >&2; WARN=$((WARN+1)); }
header(){ printf '\n\033[36m== %s ==\033[0m\n' "$*"; }

# ---------- Pre-flight ----------

header "Pre-flight"
current_ctx=$(kubectl config current-context 2>/dev/null || echo "")
if [[ "${current_ctx}" != "${EXPECTED_CONTEXT}" ]]; then
    ng "kubectl context is '${current_ctx}', expected '${EXPECTED_CONTEXT}'"
    exit 2
fi
ok "kubectl context = ${EXPECTED_CONTEXT}"

if ! kind get clusters 2>/dev/null | grep -q "^${KIND_CLUSTER_NAME}$"; then
    ng "kind cluster '${KIND_CLUSTER_NAME}' が存在しない"
    exit 2
fi
ok "kind cluster '${KIND_CLUSTER_NAME}' 存在確認"

# ---------- Check 1: helm release set vs canonical ----------

header "Helm releases ↔ canonical 集合"
ACTUAL=$(helm list -A --output json 2>/dev/null | python3 -c '
import json, sys
for r in json.load(sys.stdin):
    print(r["name"])' | sort -u)
KNOWN=$("${REPO_ROOT}/tools/local-stack/known-releases.sh" | sort -u)
DRIFT=$(comm -23 <(echo "${ACTUAL}") <(echo "${KNOWN}"))
MISSING=$(comm -13 <(echo "${ACTUAL}") <(echo "${KNOWN}"))

if [[ -z "${DRIFT}" ]]; then
    ok "drift 無し (canonical 集合外の helm release は 0)"
else
    ng "drift 検出 (canonical 集合外):"
    echo "${DRIFT}" | sed 's/^/      /'
fi
if [[ -n "${MISSING}" ]]; then
    warn "未配備の canonical layer (role 限定なら正常):"
    echo "${MISSING}" | sed 's/^/      /'
fi

# ---------- Check 2: 重要 namespace 存在 ----------

header "重要 namespace 存在"
for ns in argocd cnpg-system dapr-system gitops registry kyverno spire-system; do
    if kubectl get ns "${ns}" >/dev/null 2>&1; then
        ok "ns/${ns}"
    else
        ng "ns/${ns} 不在"
    fi
done

# ---------- Check 3: Kyverno policy 適用 + mode ----------

header "Kyverno drift policy"
if kubectl get clusterpolicy block-non-canonical-helm-releases >/dev/null 2>&1; then
    ok "ClusterPolicy block-non-canonical-helm-releases 存在"
    ACTION=$(kubectl get clusterpolicy block-non-canonical-helm-releases \
        -o jsonpath='{.spec.validationFailureAction}' 2>/dev/null)
    if [[ "${ACTION}" == "Enforce" ]]; then
        ok "validationFailureAction = Enforce (mode=strict)"
    elif [[ "${ACTION}" == "Audit" ]]; then
        ok "validationFailureAction = Audit (mode=dev)"
    else
        ng "validationFailureAction = ${ACTION:-(unset)}"
    fi
else
    ng "drift policy 未適用"
fi

# ---------- Check 4: argocd Applications ----------

header "Argo CD Applications"
APPS=$(kubectl -n argocd get applications.argoproj.io --no-headers 2>/dev/null | wc -l)
if [[ ${APPS} -eq 0 ]]; then
    warn "argocd Application が 0 件 (まだ ApplicationSet が generate していない可能性)"
else
    ok "argocd Applications: ${APPS} 件"
    NOT_SYNCED=$(kubectl -n argocd get applications.argoproj.io \
        -o json 2>/dev/null | python3 -c '
import json, sys
d = json.load(sys.stdin)
for a in d["items"]:
    sync = a.get("status",{}).get("sync",{}).get("status","Unknown")
    health = a.get("status",{}).get("health",{}).get("status","Unknown")
    if sync != "Synced" or health != "Healthy":
        print(f"  {a[\"metadata\"][\"name\"]}: sync={sync} health={health}")')
    if [[ -z "${NOT_SYNCED}" ]]; then
        ok "全 Application Synced + Healthy"
    else
        warn "未 Synced または unhealthy な Application:"
        echo "${NOT_SYNCED}" | sed 's/^/      /'
    fi
fi

# ---------- Check 5: ApplicationSet 8 件 ----------

header "ApplicationSet 8 件"
EXPECTED_APPSETS=(infra ops tier1-facade tier1-rust-service tier2-dotnet-service tier2-go-service tier3-bff tier3-web-app)
for as in "${EXPECTED_APPSETS[@]}"; do
    if kubectl -n argocd get applicationset "${as}" >/dev/null 2>&1; then
        ok "ApplicationSet/${as}"
    else
        ng "ApplicationSet/${as} 不在"
    fi
done

# ---------- Check 6: gitea 内容 ----------

header "Gitea repo 内容"
if kubectl -n gitops get deploy gitea >/dev/null 2>&1; then
    ok "gitea Deployment 存在"
    GITEA_POD=$(kubectl -n gitops get pod -l app=gitea -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    if [[ -n "${GITEA_POD}" ]]; then
        if kubectl -n gitops exec "${GITEA_POD}" -- ls /data/git/repositories/argocd/k1s0.git >/dev/null 2>&1; then
            ok "argocd/k1s0 repo bootstrap 完了"
        else
            ng "argocd/k1s0 repo が gitea に無い"
        fi
    fi
else
    ng "gitea Deployment 不在"
fi

# ---------- Check 7: 主要 endpoint 疎通 (NodePort) ----------

header "アクセスポイント (NodePort)"
for endpoint in "30080|Argo CD UI" "30700|Backstage" "30300|Grafana"; do
    PORT="${endpoint%%|*}"
    NAME="${endpoint##*|}"
    if curl -sf --max-time 3 "http://localhost:${PORT}" -o /dev/null 2>&1; then
        ok "${NAME} (port ${PORT}) reachable"
    else
        warn "${NAME} (port ${PORT}) 未疎通 (kind extraPortMappings or 起動中の可能性)"
    fi
done

# ---------- 集計 ----------

header "Summary"
printf '  PASS: %d / FAIL: %d / WARN: %d\n' "${PASS}" "${FAIL}" "${WARN}"
if [[ ${FAIL} -gt 0 ]]; then
    printf '\033[31mVerification FAILED\033[0m\n'
    exit 1
fi
printf '\033[32mVerification PASSED\033[0m'
[[ ${WARN} -gt 0 ]] && printf ' (warnings: %d)' "${WARN}"
printf '\n'
exit 0
