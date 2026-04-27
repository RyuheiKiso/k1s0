# infra/security/cert-manager

ADR-SEC-001 に従い、TLS 証明書発行を cert-manager で自動化。

## ファイル

| ファイル | 内容 |
|---|---|
| `values.yaml` | cert-manager Helm values（HA 3 replica × 3 component、Gateway API、ServiceMonitor） |
| `cluster-issuer.yaml` | Let's Encrypt prod/staging + 内部 CA の 3 ClusterIssuer |

## ローカル開発との差分

| 観点 | dev（local-stack） | prod |
|---|---|---|
| operator replica | 1 / 1 / 1 | 3 / 3 / 3（HA） |
| ClusterIssuer | self-signed のみ | Let's Encrypt prod + staging + 内部 CA |
| Gateway API | enable-gateway-api のみ | + ServerSideApply / ExperimentalGatewayAPISupport |
| 監視 | 無効 | ServiceMonitor 有効 |

## 関連

- [ADR-SEC-001](../../../docs/02_構想設計/adr/ADR-SEC-001-cert-manager.md)
- Cloudflare API token は OpenBao 動的シークレット経由で注入（plan 04-06）
