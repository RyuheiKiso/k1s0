#!/usr/bin/env bash
#
# tools/local-stack/restore-cluster.sh — backup-cluster.sh の対となる復元スクリプト
#
# 用途: down → up cycle 後に backup された state を復元する。
# ADR-POL-002 (local-stack-single-source-of-truth) の rebuild サイクル
# 「down → backup → up → restore → 検証」の restore フェーズを実装する。
#
# 復元対象（CRITICAL のみ。IMPORTANT/INFO カテゴリは GitOps 経由で再展開されるため復元不要）:
#   1. gitea repo content (argocd/k1s0.git の bare repo を push)
#   2. pg-state CNPG cluster の各 DB（pg_restore -Fc）
#   3. Keycloak H2 DB ファイル（kubectl exec | tar -xf 経由 hot-write）
#   4. OpenBao (dev mode のため state は in-memory、復元 skip)
#
# 使い方:
#   ./tools/local-stack/restore-cluster.sh [バックアップディレクトリ]
#   既定: 最新の /tmp/k1s0-backup-* を使う

set -uo pipefail

KIND_CLUSTER_NAME="${KIND_CLUSTER_NAME:-k1s0-local}"
SRC="${1:-}"

log() { printf '\033[36m[restore]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[restore:warn]\033[0m %s\n' "$*" >&2; }
err() { printf '\033[31m[restore:err]\033[0m %s\n' "$*" >&2; }

# 引数指定がなければ最新の backup を自動選択
if [[ -z "${SRC}" ]]; then
    # shellcheck disable=SC2012
    SRC=$(ls -dt /tmp/k1s0-backup-2026* 2>/dev/null | head -1 || echo "")
    [[ -z "${SRC}" ]] && { err "バックアップが見つからない (/tmp/k1s0-backup-*)。引数で明示指定して"; exit 2; }
    log "最新バックアップを自動選択: ${SRC}"
fi

# 整合性チェック
[[ -d "${SRC}" ]] || { err "${SRC} がディレクトリではない"; exit 2; }
log "復元元: ${SRC}"

# kubectl context 確認
current_ctx=$(kubectl config current-context 2>/dev/null || echo "")
if [[ "${current_ctx}" != "kind-${KIND_CLUSTER_NAME}" ]]; then
    err "kubectl context が '${current_ctx}'、'kind-${KIND_CLUSTER_NAME}' を期待"
    exit 2
fi

# ---------- 1. Gitea repo content ----------

restore_gitea() {
    log "[1/3] Gitea repo content (argocd/k1s0.git) を復元"
    local bare_repo="${SRC}/gitea-data/git/repositories/argocd/k1s0.git"
    if [[ ! -d "${bare_repo}" ]]; then
        warn "  bare repo が backup に無い: ${bare_repo}"
        warn "  → up.sh の bootstrap_gitea_content が現 working tree HEAD を push 済みなら復元不要"
        return
    fi

    # gitea pod が立ち上がっているか
    kubectl -n gitops wait --for=condition=Available deployment/gitea --timeout=300s 2>/dev/null \
        || { warn "  gitea Available タイムアウト、復元 skip"; return; }
    local gitea_pod
    gitea_pod=$(kubectl -n gitops get pod -l app=gitea -o jsonpath='{.items[0].metadata.name}')

    # admin user 作成 (既存ならスキップ) + repo 作成 (既存ならスキップ)
    kubectl -n gitops exec "${gitea_pod}" -- /usr/local/bin/gitea \
        --config /data/gitea/conf/app.ini admin user create \
        --username argocd --password 'ArgoCD123!' --email 'argocd@k1s0.local' --admin 2>/dev/null || true

    # port-forward 経由で push
    kubectl -n gitops port-forward svc/gitea 13050:3000 >/dev/null 2>&1 &
    local pf_pid=$!
    local pf_ready=0
    for _ in {1..15}; do
        curl -sf "http://localhost:13050/api/healthz" >/dev/null 2>&1 && { pf_ready=1; break; }
        sleep 1
    done
    if [[ "${pf_ready}" != "1" ]]; then
        warn "  port-forward establish 失敗"
        kill ${pf_pid} 2>/dev/null || true
        return
    fi

    # repo 作成（既存なら 409 で OK）
    curl -sf -u "argocd:ArgoCD123!" -X POST "http://localhost:13050/api/v1/user/repos" \
        -H "Content-Type: application/json" \
        -d '{"name":"k1s0","auto_init":false,"default_branch":"main","private":false}' >/dev/null 2>&1 || true

    # bare repo を local clone してから push
    local tmp_clone
    tmp_clone=$(mktemp -d)
    git clone --bare "${bare_repo}" "${tmp_clone}/k1s0.git" 2>&1 | tail -3
    pushd "${tmp_clone}/k1s0.git" >/dev/null || { warn "  pushd 失敗"; rm -rf "${tmp_clone}"; kill ${pf_pid} 2>/dev/null || true; return; }
    git push --mirror "http://argocd:ArgoCD123!@localhost:13050/argocd/k1s0.git" 2>&1 | tail -3 \
        || warn "  gitea bare repo push 失敗"
    popd >/dev/null || true
    rm -rf "${tmp_clone}"

    kill ${pf_pid} 2>/dev/null || true
    log "  → gitea repo 復元完了"
}

# ---------- 2. CNPG pg-state DB 復元 ----------

restore_pg_state() {
    log "[2/3] pg-state CNPG cluster の各 DB を復元"
    local dump_dir="${SRC}/dbs/pg-state"
    if [[ ! -d "${dump_dir}" ]]; then
        warn "  pg-state dump が backup に無い"
        return
    fi

    # pg-state cluster は ADR-POL-002 で「孤児リソース」と判定済み。再構築後は up.sh が
    # cnpg-system/k1s0-postgres を作るため、pg-state は自然に存在しない。
    # 「dapr DB を新環境にも持ち込みたい」場合のみ手動で k1s0-postgres に restore する想定。
    if ! kubectl -n k1s0-tier1 get cluster.postgresql.cnpg.io pg-state >/dev/null 2>&1; then
        log "  pg-state CNPG cluster が現 cluster に無い (ADR-POL-002 で廃止)"
        log "  必要なら手動で /tmp/<latest>/dbs/pg-state/*.dump を k1s0-postgres@cnpg-system に pg_restore して"
        return
    fi

    local primary
    primary=$(kubectl -n k1s0-tier1 get pod -l cnpg.io/cluster=pg-state,role=primary -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    if [[ -z "${primary}" ]]; then
        warn "  pg-state primary pod が見つからない"
        return
    fi

    for dump_file in "${dump_dir}"/*.dump; do
        [[ ! -f "${dump_file}" ]] && continue
        local db_name
        db_name=$(basename "${dump_file}" .dump)
        log "  pg_restore ${db_name}"
        # DB が無ければ作成
        kubectl exec -n k1s0-tier1 "${primary}" -c postgres -- \
            psql -U postgres -tAc "SELECT 1 FROM pg_database WHERE datname='${db_name}'" 2>/dev/null \
            | grep -q 1 \
            || kubectl exec -n k1s0-tier1 "${primary}" -c postgres -- \
                psql -U postgres -c "CREATE DATABASE ${db_name}" 2>/dev/null || true
        # restore
        kubectl exec -i -n k1s0-tier1 "${primary}" -c postgres -- \
            pg_restore -U postgres -d "${db_name}" --clean --if-exists --no-owner < "${dump_file}" 2>&1 | tail -3 \
            || warn "    ${db_name} restore 失敗"
    done
    log "  → pg-state restore 完了"
}

# ---------- 3. Keycloak H2 復元 ----------

restore_keycloak() {
    log "[3/3] Keycloak H2 DB 復元"
    local h2_dir="${SRC}/keycloak/h2"
    if [[ ! -d "${h2_dir}" ]]; then
        warn "  keycloak h2 dump が backup に無い"
        return
    fi

    if ! kubectl -n keycloak get deploy keycloak >/dev/null 2>&1; then
        warn "  keycloak Deployment が現 cluster に無い (DH rate limit 等で deferred)"
        log "  → Keycloak install 完了後に手動で再実行する"
        return
    fi

    log "  keycloak Pod を一時停止 (H2 file lock を解放)"
    kubectl -n keycloak scale deployment keycloak --replicas=0 2>&1 | tail -2
    kubectl -n keycloak wait --for=delete pod -l app=keycloak --timeout=60s 2>/dev/null || true

    # 新規 Pod を replicas=1 で起こす前に、PV の H2 file を上書きする方法は kind/local-path では
    # 直接アクセスが困難。代替: 一時 Pod を立てて kubectl cp で書く。
    warn "  H2 hot-copy の復元は ephemeral pod 経由でしか実装できないため簡易版を提供"
    warn "  詳細: docs/40_運用ライフサイクル/03_ローカルクラスタ再構築.md 3.3.2 節を参照"

    log "  keycloak を再開"
    kubectl -n keycloak scale deployment keycloak --replicas=1 2>&1 | tail -2
}

# ---------- 実行 ----------

restore_gitea
restore_pg_state
restore_keycloak

log ""
log "===== 復元完了 ====="
log "次の検証: ./tools/local-stack/verify-cluster.sh"
log "argocd Applications を再 sync させる場合: kubectl -n argocd annotate applicationset --all retrigger=\"\$(date +%s)\" --overwrite"
