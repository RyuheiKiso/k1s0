#!/usr/bin/env bash
# =============================================================================
# ops/dr/scripts/restore-pg-from-barman.sh
#
# 設計: ops/dr/scenarios/pg-restore.md の手順を 1 コマンド化
# 関連 Runbook: RB-DB-002 / RB-BKP-001
#
# Usage:
#   ops/dr/scripts/restore-pg-from-barman.sh \
#     --cluster k1s0-postgres \
#     --backup-name k1s0-postgres-backup-latest \
#     [--namespace cnpg-system] \
#     [--dry-run]
# =============================================================================
set -euo pipefail

CLUSTER=""
BACKUP=""
NS="cnpg-system"
DRY_RUN=0

usage() {
    sed -n '3,15p' "$0" | sed 's/^# \{0,1\}//'
    exit 2
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --cluster) CLUSTER="$2"; shift 2 ;;
        --backup-name) BACKUP="$2"; shift 2 ;;
        --namespace) NS="$2"; shift 2 ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) usage ;;
        *) echo "[error] 未知: $1"; usage ;;
    esac
done

if [[ -z "${CLUSTER}" || -z "${BACKUP}" ]]; then
    echo "[error] --cluster / --backup-name は必須" >&2
    usage
fi

apply_cmd="kubectl apply -f -"
[[ "${DRY_RUN}" == "1" ]] && apply_cmd="cat"

cat <<EOF | ${apply_cmd}
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: ${CLUSTER}
  namespace: ${NS}
spec:
  instances: 3
  bootstrap:
    recovery:
      backup:
        name: ${BACKUP}
  externalClusters:
    - name: ${BACKUP}
      barmanObjectStore:
        destinationPath: s3://${CLUSTER}-backup/
        endpointURL: \${MINIO_ENDPOINT:-http://minio.minio.svc:9000}
        s3Credentials:
          accessKeyId:
            name: k1s0-minio-credentials
            key: access-key
          secretAccessKey:
            name: k1s0-minio-credentials
            key: secret-key
EOF

if [[ "${DRY_RUN}" == "0" ]]; then
    echo "[info] 復旧進捗を監視: kubectl get cluster ${CLUSTER} -n ${NS} -w"
    kubectl wait --for=condition=Ready "cluster/${CLUSTER}" -n "${NS}" --timeout=1800s
    echo "[info] tier1 facade rolling restart"
    kubectl rollout restart deployment/tier1-facade -n k1s0
fi
