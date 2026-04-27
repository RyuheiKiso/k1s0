# infra/security/keycloak

ADR-SEC-001 に従い、認証 / 認可（OIDC IdP）を Keycloak で運用。

## ファイル

| ファイル | 内容 |
|---|---|
| `values.yaml` | Bitnami Keycloak Helm values（HA 3 replica + Infinispan + 外部 CNPG + production mode + ServiceMonitor） |

## ローカル開発との差分

| 観点 | dev | prod |
|---|---|---|
| replica | 1 | 3（Infinispan distributed cache） |
| auth | 平文 password | OpenBao 動的シークレット（plan 04-06） |
| production | false | true |
| proxy | edge | edge（Istio Ambient Gateway 終端） |
| 監視 | 無効 | ServiceMonitor 有効 |
| realm import | （手動） | ConfigMap `k1s0-keycloak-realm` を起動時 import |

## 用途

- Grafana / Argo CD / Backstage / portal-bff の OIDC client
- tier1 facade の JWT 検証（Policy Enforcer interceptor）
- ロールベースアクセス制御（RBAC グループ「k1s0-admin」「k1s0-developer」「k1s0-viewer」）

## 関連

- [ADR-SEC-001](../../../docs/02_構想設計/adr/ADR-SEC-001-keycloak-oidc.md)
