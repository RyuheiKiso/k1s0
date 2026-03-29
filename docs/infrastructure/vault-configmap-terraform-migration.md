# Vault ConfigMap と Terraform ロール設定の乖離

## VAULT-02 監査対応

### 現状
`infra/vault/auth/k1s0-system-auth.yaml` の ConfigMap は `role_name: "auth-rust"` を参照しているが、
`infra/terraform/modules/vault/auth.tf` は単一共有ロール `system` を使用している。

### 原因
ADR-0045（Vault per-service ロール分割）で計画している per-service ロール移行が段階的移行中のため、
ConfigMap（移行後を先取りした設定）と Terraform（現在の実態）が乖離している。

### 解決策（ADR-0045 に基づく移行計画）
1. Terraform で per-service ロールを定義（`auth-rust`, `session-rust` 等）
2. 各サービスの SecretProviderClass の `roleName` を per-service ロールに更新
3. 旧 `system` 共有ロールを廃止

移行完了まで、ConfigMap の `role_name` は Terraform の実態（`system`）と一致しないことに注意。
K8s 上での動作は実際の Vault ロールに依存するため、デプロイ前に `infra/terraform/` の設定を確認すること。

### 参照
- `docs/architecture/adr/0045-vault-per-service-role.md`
