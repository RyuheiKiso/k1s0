#!/usr/bin/env bash
#
# tools/local-stack/lib/apply-layers.sh — up.sh からソースされる apply_* 関数群
#
# ADR-POL-002 P4 finishing: up.sh が 500 行制限を超えたため apply_* 関数を本ファイルに分離。
# 親 up.sh で `source "${SCRIPT_DIR}/lib/apply-layers.sh"` として読み込む。
# 直接実行は不可（log / warn / has_layer / ensure_helm_repo / wait_for_pods_ready /
# generate_metallb_pool / 各 *_VERSION readonly / MANIFESTS / REPO_ROOT / MODE が
# 親 shell から継承されている前提）。
#
# 含まれる関数:
#   apply_cni / apply_cert_manager / apply_metallb / apply_istio / apply_kyverno /
#   apply_spire / apply_dapr / apply_flagd / apply_gitea / apply_registry / apply_argocd /
#   bootstrap_gitea_content / apply_argocd_appsets / apply_argo_rollouts /
#   apply_envoy_gateway / apply_cnpg / apply_kafka / apply_temporal / apply_minio /
#   apply_valkey / apply_openbao / apply_backstage / apply_observability / apply_keycloak

apply_cni() {
    has_layer cni || return 0
    log "Calico CNI install (${CALICO_VERSION}, quay.io mirror)"
    kubectl create -f "https://raw.githubusercontent.com/projectcalico/calico/${CALICO_VERSION}/manifests/tigera-operator.yaml" 2>/dev/null || true
    # registry: "quay.io/" 指定で Docker Hub の rate limit 回避（ADR-POL-002 P4 で実証済み）。
    # multinode (4 ノード) で同一イメージを 4× pull するため、unauthenticated docker.io 100 pulls/6h
    # の壁を踏みやすい。quay.io/calico/* は同等の image を提供する公式ミラー。
    kubectl apply -f - <<EOF || true
apiVersion: operator.tigera.io/v1
kind: Installation
metadata:
  name: default
spec:
  registry: quay.io/
  calicoNetwork:
    ipPools:
      - blockSize: 26
        cidr: 10.244.0.0/16
        encapsulation: VXLANCrossSubnet
        natOutgoing: Enabled
        nodeSelector: all()
EOF
    # multinode の image pull に時間がかかるため timeout を 600s に延長
    wait_for_pods_ready calico-system 600s
}

apply_cert_manager() {
    has_layer cert-manager || return 0
    log "cert-manager install (${CERT_MANAGER_VERSION})"
    # cert-manager の --enable-gateway-api 引数が要求する Gateway API CRDs を事前 install。
    # envoy-gateway 自身も同 CRDs を bundle するが kubectl apply は冪等に動作するため衝突しない。
    # ADR-POL-002 P4 rebuild 時に「Gateway API CRDs 不在で cert-manager-controller が CrashLoopBackOff」
    # を実証済み（v1.20.2 のデフォルト挙動として CRDs 不在を fatal とする変更が入った）。
    log "  Gateway API CRDs (standard v1.2.1) を事前 install"
    kubectl apply -f "https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.2.1/standard-install.yaml" 2>&1 | tail -3 \
        || warn "  Gateway API CRDs install 失敗（後続 cert-manager が CrashLoop の可能性）"
    ensure_helm_repo jetstack https://charts.jetstack.io
    helm upgrade --install cert-manager jetstack/cert-manager \
        --namespace cert-manager --version "${CERT_MANAGER_VERSION}" \
        -f "${MANIFESTS}/20-cert-manager/values.yaml" --wait
    kubectl apply -f "${MANIFESTS}/20-cert-manager/cluster-issuer-selfsigned.yaml"
}

patch_kind_storageclasses() {
    # ADR-POL-002 / ADR-INFRA-001 / ADR-NET-001 整合の kind 環境向け StorageClass 整備。
    # 正典 (infra/k8s/storage/storage-classes.yaml) は Longhorn 前提で、kind 環境では
    # Longhorn を install しないため `k1s0-default` 等の SC が `PLACEHOLDER_csi_provisioner`
    # で残り、observability (grafana / loki / prometheus / tempo) の PVC が unbound になる。
    #
    # 本関数は kind 環境のみで `k1s0-{default,high-iops,backup,shared}` の provisioner を
    # `rancher.io/local-path` に書き換える。provisioner は immutable のため delete + recreate。
    # SoT 整合: 本処理は helm release ではなく StorageClass の手当てなので、Kyverno
    # block-non-canonical-helm-releases policy / known-releases.sh の対象外。
    #
    # 採用判断 (Layer 1): C 案 (up.sh で patch) は ADR-POL-002 三層防御と整合し、
    # kind rebuild 時の再現性も確保する。production では本関数は no-op (実 cluster
    # の SC は CSI provisioner で正規に bind される前提)。
    has_layer storageclass-kind-patch || true  # 既定で常に実行する (kind 環境向け)

    # production cluster (CSI driver 設置済) では本処理が必要ない。
    # kind cluster かどうかを cluster name で判定する (kind は kind-* prefix)。
    local current_ctx
    current_ctx="$(kubectl config current-context 2>/dev/null || echo unknown)"
    if [[ "${current_ctx}" != "kind-${KIND_CLUSTER_NAME}" ]]; then
        log "kind cluster ではないため StorageClass patch をスキップ (current context: ${current_ctx})"
        return 0
    fi

    log "kind 環境向け StorageClass 整備 (k1s0-* の provisioner を rancher.io/local-path に置換)"
    local sc patched=0
    for sc in k1s0-default k1s0-high-iops k1s0-backup k1s0-shared; do
        local current_provisioner
        current_provisioner="$(kubectl get sc "${sc}" -o jsonpath='{.provisioner}' 2>/dev/null || echo NONE)"
        if [[ "${current_provisioner}" == NONE ]]; then
            continue
        fi
        if [[ "${current_provisioner}" =~ PLACEHOLDER ]]; then
            log "  ${sc}: ${current_provisioner} → rancher.io/local-path に置換"
            kubectl delete sc "${sc}" --ignore-not-found=true >/dev/null
            kubectl apply -f - <<EOF >/dev/null
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: ${sc}
  annotations:
    k1s0.io/kind-replacement: "true"
    k1s0.io/source-of-truth: "tools/local-stack/lib/apply-layers.sh:patch_kind_storageclasses"
provisioner: rancher.io/local-path
reclaimPolicy: Delete
volumeBindingMode: WaitForFirstConsumer
EOF
            patched=$((patched + 1))
        fi
    done

    # default StorageClass を k1s0-default に集約 (standard との重複 default を解消)
    if kubectl get sc standard -o jsonpath='{.metadata.annotations.storageclass\.kubernetes\.io/is-default-class}' 2>/dev/null | grep -q "true"; then
        log "  standard SC の is-default-class アノテーションを除去 (k1s0-default に default を集約)"
        kubectl annotate sc standard storageclass.kubernetes.io/is-default-class- --overwrite >/dev/null 2>&1 || true
    fi
    if [[ "$(kubectl get sc k1s0-default -o jsonpath='{.metadata.annotations.storageclass\.kubernetes\.io/is-default-class}' 2>/dev/null || echo)" != "true" ]]; then
        log "  k1s0-default を default StorageClass に設定"
        kubectl annotate sc k1s0-default storageclass.kubernetes.io/is-default-class=true --overwrite >/dev/null 2>&1 || true
    fi

    # 既に Pending 状態の PVC がある場合は再 bind を促す (PVC を削除はしない、provisioner が次回 retry で bind)
    if [[ "${patched}" -gt 0 ]]; then
        log "  ${patched} 件の SC を置換。Pending PVC は次回 provisioner retry で bind される"
    fi
}

apply_metallb() {
    has_layer metallb || return 0
    log "MetalLB install (${METALLB_VERSION})"
    kubectl apply -f "https://raw.githubusercontent.com/metallb/metallb/${METALLB_VERSION}/config/manifests/metallb-native.yaml"
    kubectl -n metallb-system wait --for=condition=Available deployment/controller --timeout=180s
    log "MetalLB IPAddressPool を動的生成（kind docker network 経由）"
    local pool_range
    pool_range="$(generate_metallb_pool)"
    log "  pool=${pool_range}"
    kubectl apply -f - <<EOF
apiVersion: metallb.io/v1beta1
kind: IPAddressPool
metadata:
  name: k1s0-local-pool
  namespace: metallb-system
spec:
  addresses:
    - ${pool_range}
  autoAssign: true
---
apiVersion: metallb.io/v1beta1
kind: L2Advertisement
metadata:
  name: k1s0-local-l2
  namespace: metallb-system
spec:
  ipAddressPools:
    - k1s0-local-pool
EOF
}

apply_istio() {
    has_layer istio || return 0
    log "Istio Ambient install"
    if ! command -v istioctl >/dev/null 2>&1; then
        warn "istioctl が PATH に無い。infra-ops / full プロファイルから実行してください"
        return 0
    fi
    istioctl install --set profile=ambient --skip-confirmation \
        -f "${MANIFESTS}/30-istio-ambient/values-ambient.yaml" || true
}

apply_kyverno() {
    has_layer kyverno || return 0
    log "Kyverno install (${KYVERNO_VERSION})"
    ensure_helm_repo kyverno https://kyverno.github.io/kyverno/
    helm upgrade --install kyverno kyverno/kyverno \
        --namespace kyverno --version "${KYVERNO_VERSION}" \
        -f "${MANIFESTS}/35-kyverno/values.yaml" --wait

    # baseline + drift policy を適用（ADR-POL-001 二分所有 + ADR-POL-002 SoT）
    log "  Kyverno ClusterPolicy (baseline + drift) を適用"
    kubectl apply -k "${REPO_ROOT}/infra/security/kyverno/" 2>&1 | tail -5 || warn "  policy apply 失敗"

    # ADR-POL-002 mode 切替: dev のみ drift policy を Audit に patch（逆設計）。
    # YAML default を Enforce にしたため strict mode では何もしない（Enforce が persistent）。
    if [[ "${MODE}" == "dev" ]]; then
        log "  mode=dev: drift policy を Audit に切替（探索を許す）"
        kubectl patch clusterpolicy block-non-canonical-helm-releases --type=json \
            -p='[{"op":"replace","path":"/spec/validationFailureAction","value":"Audit"}]' \
            2>/dev/null || warn "  drift policy Audit patch 失敗"
    else
        log "  mode=strict: drift policy は YAML default の Enforce で稼働"
    fi
}

apply_spire() {
    has_layer spire || return 0
    log "SPIRE install (umbrella ${SPIRE_VERSION} + crds ${SPIRE_CRDS_VERSION})"
    ensure_helm_repo spiffe https://spiffe.github.io/helm-charts-hardened/
    helm upgrade --install spire-crds spiffe/spire-crds \
        --namespace spire-system --version "${SPIRE_CRDS_VERSION}" --create-namespace
    helm upgrade --install spire spiffe/spire \
        --namespace spire-system --version "${SPIRE_VERSION}" \
        -f "${MANIFESTS}/40-spire/values.yaml" --wait || true
}

apply_dapr() {
    has_layer dapr || return 0
    log "Dapr install (${DAPR_VERSION})"
    ensure_helm_repo dapr https://dapr.github.io/helm-charts/
    helm upgrade --install dapr dapr/dapr \
        --namespace dapr-system --version "${DAPR_VERSION}" \
        -f "${MANIFESTS}/45-dapr/values.yaml" --wait
    log "Dapr Components を適用"
    if [[ -d "${REPO_ROOT}/tools/local-stack/dapr/components" ]]; then
        kubectl apply -f "${REPO_ROOT}/tools/local-stack/dapr/components/" || true
    fi
}

apply_flagd() {
    has_layer flagd || return 0
    log "flagd install"
    kubectl apply -f "${MANIFESTS}/50-flagd/manifest.yaml"
}

apply_gitea() {
    has_layer gitea || return 0
    log "Gitea install (Argo CD の sync 元 git)"
    kubectl apply -f "${REPO_ROOT}/infra/gitops/local-stack/gitea-deployment.yaml"
    kubectl apply -f "${REPO_ROOT}/infra/gitops/local-stack/gitea-service.yaml"
}

apply_registry() {
    has_layer registry || return 0
    log "OCI registry install (Kyverno ImageVerify 検証用)"
    kubectl apply -f "${REPO_ROOT}/infra/registry/local/deployment.yaml"
    kubectl apply -f "${REPO_ROOT}/infra/registry/local/service.yaml"
}

apply_argocd() {
    has_layer argocd || return 0
    log "Argo CD install (${ARGOCD_VERSION}, NodePort 30080)"
    ensure_helm_repo argo https://argoproj.github.io/argo-helm
    helm upgrade --install argocd argo/argo-cd \
        --namespace argocd --version "${ARGOCD_VERSION}" \
        -f "${MANIFESTS}/55-argocd/values.yaml" --wait || true

    # ADR-POL-002 (E 群解消): GitOps を SoT として確立する。
    # argocd 起動後、gitea に repo 内容を push し、deploy/apps/app-of-apps と
    # deploy/apps/application-sets の全 9 件を argocd に登録する（URL は local gitea に変換）。
    if has_layer gitea; then
        kubectl apply -f "${REPO_ROOT}/infra/gitops/local-stack/argocd-repo-secret.yaml" 2>/dev/null || true
        bootstrap_gitea_content
        apply_argocd_appsets
    fi
}

# gitea に local リポジトリ内容を push（idempotent）。
# argocd が repo を sync する前提として呼ばれる。
bootstrap_gitea_content() {
    log "  Gitea content bootstrap (admin user + argocd/k1s0 repo + push)"
    kubectl -n gitops wait --for=condition=Available deployment/gitea --timeout=300s 2>/dev/null \
        || { warn "  gitea が Available にならず bootstrap スキップ"; return; }
    local gitea_pod
    gitea_pod=$(kubectl -n gitops get pod -l app=gitea -o jsonpath='{.items[0].metadata.name}')

    # ADR-POL-002 P4 finishing で発覚: gitea web pod が Available になっても
    # SQLite schema 初期化が完了するまでに数秒かかる（admin user create が
    # "no such table: user" で失敗する）。schema 初期化を polling で確認する。
    log "  gitea SQLite schema 初期化を待機 (max 60s)"
    local schema_ready=0
    for i in {1..30}; do
        if kubectl -n gitops exec "${gitea_pod}" -- /usr/local/bin/gitea \
            --config /data/gitea/conf/app.ini admin user list 2>/dev/null >/dev/null; then
            schema_ready=1
            log "    → schema ready ($((i*2))s)"
            break
        fi
        sleep 2
    done
    [[ "${schema_ready}" == "1" ]] || warn "  schema 初期化が 60s 内に終わらず先に進む（後続 retry あり）"

    # admin user 作成（既存なら skip）。gitea 1.22 系の admin user create CLI を使用。
    # ADR-POL-002 P4 で発覚: --config を渡さないと "Unable to load config file" で失敗するため明示。
    kubectl -n gitops exec "${gitea_pod}" -- /usr/local/bin/gitea \
        --config /data/gitea/conf/app.ini admin user create \
        --username argocd --password 'ArgoCD123!' --email 'argocd@k1s0.local' --admin 2>/dev/null \
        || true

    # port-forward 経由で API + git push
    # ADR-POL-002 P4 finishing で発覚: port-forward の establish 待ちが固定 sleep だと
    # 環境差で取り損ねる。curl で health check が通るまで polling する。
    kubectl -n gitops port-forward svc/gitea 13000:3000 >/dev/null 2>&1 &
    local pf_pid=$!
    local pf_ready=0
    for i in {1..15}; do
        if curl -sf "http://localhost:13000/api/healthz" >/dev/null 2>&1; then
            pf_ready=1
            break
        fi
        sleep 1
    done
    if [[ "${pf_ready}" != "1" ]]; then
        warn "  gitea port-forward が establish せず bootstrap 継続不能"
        kill ${pf_pid} 2>/dev/null || true
        return
    fi

    # repo 作成: user "argocd" 直下に repo "k1s0" を作る（既存なら skip）。
    # ADR-POL-002 P4 で発覚: gitea で user 名と org 名は同 namespace のため、user "argocd" 作成後に
    # org "argocd" を作ろうとすると "user already exists" で衝突する。
    # argocd Application URL は `argocd/k1s0` のままで user/org どちらでも解決するため、
    # user 直下 repo として作成する。
    # ADR-POL-002 P4 finishing で発覚: API repo 作成失敗を |true で誤魔化すと
    # 後段 git push が "repository not found" 失敗する。明示的に成功確認 + retry する。
    local repo_ready=0
    for i in {1..5}; do
        local code
        code=$(curl -sS -o /dev/null -w "%{http_code}" -u "argocd:ArgoCD123!" \
            -X POST "http://localhost:13000/api/v1/user/repos" \
            -H "Content-Type: application/json" \
            -d '{"name":"k1s0","auto_init":false,"default_branch":"main","private":false}' 2>/dev/null)
        # 201 Created = 新規作成、409 Conflict = 既存（OK）、それ以外は retry
        if [[ "${code}" == "201" || "${code}" == "409" ]]; then
            repo_ready=1
            log "    → repo argocd/k1s0 ready (HTTP ${code}, attempt ${i})"
            break
        fi
        sleep 2
    done
    if [[ "${repo_ready}" != "1" ]]; then
        warn "  gitea repo 作成失敗（5 retry）。後段 push は失敗する見込み"
    fi

    # 現 working tree を push（commit 済み内容のみ）
    pushd "${REPO_ROOT}" >/dev/null || { warn "  pushd 失敗"; kill ${pf_pid} 2>/dev/null || true; return; }
    git remote remove gitea-local 2>/dev/null || true
    git remote add gitea-local "http://argocd:ArgoCD123!@localhost:13000/argocd/k1s0.git"
    if git push gitea-local HEAD:main --force 2>&1 | tail -3; then
        log "    → git push 成功"
    else
        warn "  gitea push 失敗（後段 argocd Application が source unreachable で OutOfSync する）"
    fi
    git remote remove gitea-local 2>/dev/null || true
    popd >/dev/null || true

    kill ${pf_pid} 2>/dev/null || true
    wait ${pf_pid} 2>/dev/null || true
    log "  → gitea bootstrap 完了"
}

# deploy/apps/application-sets/*.yaml を local gitea URL に変換して直接適用。
# 設計判断 (ADR-POL-002):
#   - canonical 定義は production の GitHub URL のままで保持（deploy/apps/ 配下を改変しない）。
#   - local-stack ではコピー→sed 変換→apply の 3 段で local gitea URL に切替える。
#   - app-of-apps は production の GitHub root から再帰展開する pattern のため local-stack では使わない
#     （local で app-of-apps を入れると gitea 内の ApplicationSet が再び GitHub URL を読みに行き循環するため）。
apply_argocd_appsets() {
    log "  AppProjects + ApplicationSets を Argo CD に適用 (GitHub URL → local gitea URL 変換)"
    local tmp
    tmp=$(mktemp -d)
    # AppProject を先に適用（ApplicationSet が project: k1s0-* を参照するため）
    cp "${REPO_ROOT}/deploy/apps/projects/"*.yaml "${tmp}/" 2>/dev/null || true
    cp "${REPO_ROOT}/deploy/apps/application-sets/"*.yaml "${tmp}/" 2>/dev/null || true
    find "${tmp}" -name "*.yaml" -exec sed -i \
        -e 's|https://github.com/k1s0/k1s0.git|http://gitea.gitops.svc.cluster.local:3000/argocd/k1s0.git|g' \
        -e 's|"https://github.com/k1s0/k1s0.git"|"http://gitea.gitops.svc.cluster.local:3000/argocd/k1s0.git"|g' \
        {} +
    kubectl apply -f "${tmp}/" 2>&1 | tail -15 || warn "  appset apply 失敗"
    rm -rf "${tmp}"
    local proj_count appset_count
    # shellcheck disable=SC2012
    proj_count=$(ls "${REPO_ROOT}/deploy/apps/projects/"*.yaml 2>/dev/null | wc -l)
    # shellcheck disable=SC2012
    appset_count=$(ls "${REPO_ROOT}/deploy/apps/application-sets/"*.yaml 2>/dev/null | wc -l)
    log "  → ${proj_count} AppProjects + ${appset_count} ApplicationSets 適用完了"
}

apply_argo_rollouts() {
    has_layer argo-rollouts || return 0
    log "Argo Rollouts install (${ARGO_ROLLOUTS_VERSION}, ADR-CICD-002)"
    ensure_helm_repo argo https://argoproj.github.io/argo-helm
    helm upgrade --install argo-rollouts argo/argo-rollouts \
        --namespace argo-rollouts --version "${ARGO_ROLLOUTS_VERSION}" \
        -f "${MANIFESTS}/56-argo-rollouts/values.yaml" --wait || true
}

apply_envoy_gateway() {
    has_layer envoy-gateway || return 0
    log "Envoy Gateway install (${ENVOY_GATEWAY_VERSION}, OCI chart)"
    kubectl create namespace envoy-gateway-system 2>/dev/null || true
    helm upgrade --install envoy-gateway oci://docker.io/envoyproxy/gateway-helm \
        --namespace envoy-gateway-system --version "${ENVOY_GATEWAY_VERSION}" \
        -f "${MANIFESTS}/57-envoy-gateway/values.yaml" --wait || true
}

apply_cnpg() {
    has_layer cnpg || return 0
    log "CloudNativePG operator install (${CNPG_VERSION})"
    ensure_helm_repo cnpg https://cloudnative-pg.github.io/charts
    helm upgrade --install cnpg cnpg/cloudnative-pg \
        --namespace cnpg-system --version "${CNPG_VERSION}" \
        -f "${MANIFESTS}/60-cnpg/values.yaml" --wait
    log "k1s0 Postgres cluster を作成"
    kubectl apply -f "${MANIFESTS}/60-cnpg/k1s0-cluster.yaml"
}

apply_kafka() {
    has_layer kafka || return 0
    log "Strimzi Kafka operator install (${STRIMZI_VERSION})"
    ensure_helm_repo strimzi https://strimzi.io/charts/
    helm upgrade --install strimzi strimzi/strimzi-kafka-operator \
        --namespace kafka --version "${STRIMZI_VERSION}" \
        -f "${MANIFESTS}/65-kafka/strimzi-values.yaml" --wait
    log "k1s0 Kafka cluster (KRaft 単一ノード)"
    kubectl apply -f "${MANIFESTS}/65-kafka/k1s0-kafka.yaml"
}

apply_temporal() {
    has_layer temporal || return 0
    log "Temporal install (${TEMPORAL_VERSION}, workflow engine)"
    ensure_helm_repo temporal https://go.temporal.io/helm-charts
    helm upgrade --install temporal temporal/temporal \
        --namespace temporal --version "${TEMPORAL_VERSION}" \
        -f "${MANIFESTS}/66-temporal/values.yaml" --timeout 10m || true
}

apply_minio() {
    has_layer minio || return 0
    log "MinIO install"
    ensure_helm_repo minio https://charts.min.io/
    helm upgrade --install minio minio/minio \
        --namespace minio \
        -f "${MANIFESTS}/70-minio/values.yaml" --wait || true
}

apply_valkey() {
    has_layer valkey || return 0
    log "Valkey install (${VALKEY_VERSION})"
    ensure_helm_repo bitnami https://charts.bitnami.com/bitnami
    helm upgrade --install valkey bitnami/valkey \
        --namespace valkey --version "${VALKEY_VERSION}" \
        -f "${MANIFESTS}/75-valkey/values.yaml" --wait || true
}

apply_openbao() {
    has_layer openbao || return 0
    log "OpenBao (dev mode) install (${OPENBAO_VERSION})"
    ensure_helm_repo openbao https://openbao.github.io/openbao-helm
    helm upgrade --install openbao openbao/openbao \
        --namespace openbao --version "${OPENBAO_VERSION}" \
        -f "${MANIFESTS}/80-openbao/values.yaml" || true
}

apply_backstage() {
    has_layer backstage || return 0
    log "Backstage install (${BACKSTAGE_VERSION}, NodePort 30700)"
    ensure_helm_repo backstage https://backstage.github.io/charts
    helm upgrade --install backstage backstage/backstage \
        --namespace backstage --version "${BACKSTAGE_VERSION}" \
        -f "${MANIFESTS}/85-backstage/values.yaml" || true
}

apply_observability() {
    has_layer observability || return 0
    log "Observability (Loki ${LOKI_CHART_VERSION} / Tempo ${TEMPO_CHART_VERSION} / Grafana ${GRAFANA_CHART_VERSION} / OTel ${OTEL_COLLECTOR_VERSION}) install"
    ensure_helm_repo grafana https://grafana.github.io/helm-charts
    ensure_helm_repo open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts
    helm upgrade --install loki grafana/loki \
        --namespace observability --version "${LOKI_CHART_VERSION}" \
        -f "${MANIFESTS}/90-observability/values-loki.yaml" || true
    helm upgrade --install tempo grafana/tempo \
        --namespace observability --version "${TEMPO_CHART_VERSION}" \
        -f "${MANIFESTS}/90-observability/values-tempo.yaml" || true
    helm upgrade --install grafana grafana/grafana \
        --namespace observability --version "${GRAFANA_CHART_VERSION}" \
        -f "${MANIFESTS}/90-observability/values-grafana.yaml" || true
    # OTel Collector + Prometheus (現状 drift で見つかったため SoT に取り込む)
    ensure_helm_repo prometheus-community https://prometheus-community.github.io/helm-charts
    helm upgrade --install prometheus prometheus-community/prometheus \
        --namespace observability --version "${PROMETHEUS_CHART_VERSION}" \
        -f "${MANIFESTS}/90-observability/values-prometheus.yaml" || true
    helm upgrade --install otel-collector open-telemetry/opentelemetry-collector \
        --namespace observability --version "${OTEL_COLLECTOR_VERSION}" \
        -f "${MANIFESTS}/90-observability/values-otel-collector.yaml" || true
}

apply_keycloak() {
    has_layer keycloak || return 0
    log "Keycloak install (${KEYCLOAK_VERSION}, CNPG 連携)"
    ensure_helm_repo bitnami https://charts.bitnami.com/bitnami
    helm upgrade --install keycloak bitnami/keycloak \
        --namespace keycloak --version "${KEYCLOAK_VERSION}" \
        -f "${MANIFESTS}/95-keycloak/values.yaml" || true
}
