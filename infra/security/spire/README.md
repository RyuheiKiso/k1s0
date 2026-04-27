# infra/security/spire

ADR-SEC-003 に従い、Workload Identity を SPIRE / SPIFFE で運用。

## ファイル

| ファイル | 内容 |
|---|---|
| `values.yaml` | SPIRE umbrella chart values（spire-server HA 3 replica + 外部 CNPG + cert-manager upstream + CSI driver + OIDC Discovery） |

## ローカル開発との差分

| 観点 | dev | prod |
|---|---|---|
| trust domain | `k1s0.local` | `k1s0.example.com` |
| spire-server replica | 1 | 3（HA） |
| dataStore | sqlite3 | PostgreSQL（CNPG 共有） |
| upstream CA | （self-CA） | cert-manager `k1s0-internal-ca` |
| OIDC Discovery | 無効 | 有効（tier1 facade JWT-SVID 検証） |

## アーキテクチャ

- **spire-server**: SVID 発行 / Registration entry 管理。3 replica HA、外部 CNPG にデータ永続化
- **spire-agent**: 全 Node に DaemonSet、Workload Attestation（k8s pod 経由）
- **CSI driver**: Pod 起動時に SVID を `/run/spire/svid/` に mount（sidecar 不要、ADR-SEC-003 推奨）
- **OIDC Discovery**: `spire.k1s0.example.com/.well-known/openid-configuration` で JWT-SVID を Federate

## 関連

- [ADR-SEC-003](../../../docs/02_構想設計/adr/ADR-SEC-003-spire-spiffe.md)
- [ADR-SEC-002](../../../docs/02_構想設計/adr/ADR-SEC-002-mtls-strict.md) — Istio Ambient ztunnel が SPIRE SVID を使用
