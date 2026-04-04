# Keycloak Realm 同期手順書

**対象**: k1s0-system Keycloak レルム（k1s0）  
**正本**: `infra/keycloak/realm-k1s0.json`（Docker 版、リポジトリ管理）

---

## 概要

Keycloak の Realm 設定は `infra/keycloak/realm-k1s0.json` を正本として管理する。
Docker Compose 環境では起動時に自動インポートされるが、Kubernetes 環境では手動または
ConfigMap 経由での適用が必要である。

---

## 1. 正本の場所と更新

| 環境 | ファイル | 管理方法 |
|------|---------|---------|
| Docker Compose | `infra/keycloak/realm-k1s0.json` | **正本**（リポジトリ管理） |
| Kubernetes | `infra/kubernetes/verify/keycloak.yaml`（ConfigMap） | Docker 版から同期 |

**Realm 設定を変更した場合**:
1. Docker Compose で変更を加えた後、Keycloak Admin Console からエクスポートする
2. `infra/keycloak/realm-k1s0.json` を更新する
3. `infra/kubernetes/verify/keycloak.yaml` の ConfigMap データも同期する

---

## 2. Docker Compose 環境（自動適用）

Docker Compose 起動時、Keycloak は `/opt/keycloak/data/import/` に配置された
`realm-k1s0.json` を自動的にインポートする。

```bash
# 起動コマンド（infra プロファイル）
docker compose --env-file .env.dev \
  -f docker-compose.yaml -f docker-compose.dev.yaml \
  --profile infra up -d keycloak

# インポート確認（Keycloak 起動ログ）
docker logs keycloak 2>&1 | grep -i "import\|realm"
```

---

## 3. Kubernetes 環境（手動適用）

### 3.1 ConfigMap 経由での適用（推奨）

`infra/kubernetes/verify/keycloak.yaml` の ConfigMap にレルム設定を含め、
Keycloak Pod の `/opt/keycloak/data/import/` にマウントする。

```bash
# ConfigMap を適用する
kubectl apply -f infra/kubernetes/verify/keycloak.yaml -n k1s0-system

# Keycloak Pod を再起動してインポートをトリガーする
kubectl rollout restart deployment/keycloak -n k1s0-system

# インポート確認
kubectl logs -n k1s0-system -l app=keycloak --tail=50 | grep -i "import\|realm"
```

### 3.2 kubectl exec でのオンライン適用（緊急時）

```bash
# Keycloak Pod 名を取得する
KC_POD=$(kubectl get pod -n k1s0-system -l app=keycloak -o jsonpath='{.items[0].metadata.name}')

# レルム設定ファイルをコピーする
kubectl cp infra/keycloak/realm-k1s0.json \
  k1s0-system/${KC_POD}:/tmp/realm-k1s0.json

# kcadm.sh でインポートする（既存レルムの更新）
kubectl exec -n k1s0-system ${KC_POD} -- \
  /opt/keycloak/bin/kcadm.sh update realms/k1s0 \
  -f /tmp/realm-k1s0.json \
  --server http://localhost:8080 \
  --realm master \
  --user admin \
  --password "${KEYCLOAK_ADMIN_PASSWORD}"
```

---

## 4. 差分確認（Docker 版 vs K8s 版）

Docker 版と Kubernetes 版 ConfigMap の差分を確認する手順:

```bash
# K8s ConfigMap からレルム設定を取得する
kubectl get configmap keycloak-realm -n k1s0-system \
  -o jsonpath='{.data.realm-k1s0\.json}' > /tmp/k8s-realm.json

# 差分確認
diff infra/keycloak/realm-k1s0.json /tmp/k8s-realm.json
```

差分がある場合は Docker 版を正本として K8s 版を更新すること。

---

## 5. 将来の改善計画

- **CI 自動同期チェック**: `scripts/check-keycloak-realm-sync.sh` を実装し、Docker 版と K8s ConfigMap 版の差分を検出して CI を fail させる
- **GitOps 統合**: ArgoCD や Flux を使った Realm 設定の自動適用を検討

---

## 関連ドキュメント

- [Keycloak SSL 設定](../infrastructure/security/keycloak-ssl-configuration.md)
- [インフラ設計](../infrastructure/kubernetes/kubernetes設計.md)
- [ADR-0081](../architecture/adr/0081-external-audit-remediation-2026-04-04-v4.md)
