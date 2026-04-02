# Vault ConfigMap と Terraform ロール設定の乖離

## VAULT-02 監査対応

> **【実装完了】** 2026-03-30 にて ADR-0045 の per-service ロール移行が完全完了した。
> 以下は経緯・背景記録として残す。

### 完了状況（I-5 / Phase 5 対応）

ADR-0045（Vault per-service ロール分割）の全フェーズが完了済み。

| フェーズ | 内容 | 完了日 |
|---------|------|--------|
| H-02 | `auth.tf` に 26 サービス分の個別ロールを実装 | 2026-03-29 |
| H-03 | 全 system tier サービス（24 SA）の `values.yaml` を個別ロールに移行 | 2026-03-30 |
| H-04 | `auth.tf` の旧モノリシック `system` ロールを削除 | 2026-03-30 |

現在、`infra/vault/auth/` 配下の各 ConfigMap（`role_name: "auth-rust"` 等）は、
`infra/terraform/modules/vault/auth.tf` の個別ロール定義と一致している。
乖離は解消済み。

### 旧状況（参考記録）

移行前は以下の乖離があった:
- `infra/vault/auth/k1s0-system-auth.yaml` の ConfigMap: `role_name: "auth-rust"`（個別ロール）
- `infra/terraform/modules/vault/auth.tf`: 単一共有ロール `system`（27 SA を集約）

この乖離は ADR-0045 の per-service ロール移行を ConfigMap 側が先取りしたことで発生していた。
移行完了後は両者が一致しており、デプロイ前の手動確認は不要になった。

### 参照
- `docs/architecture/adr/0045-vault-per-service-roles.md` — 移行計画・完了履歴
