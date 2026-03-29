# ADR-0047: Vault シークレットパス命名規則統一（auth-server）

## ステータス
承認済み

## コンテキスト
`infra/helm/services/system/auth/values.yaml` の Vault シークレットパスが `secret/data/k1s0/system/auth/database` を参照していたが、以下の全ての設定ファイルは `auth-server` を使用している:
- `infra/vault/policies/auth-server.hcl`: `secret/data/k1s0/system/auth-server/*`
- `infra/vault/secret-provider-class/auth-secrets.yaml`: `secret/data/k1s0/system/auth-server/database`
- `infra/docker/vault/init-vault.sh`: `secret/k1s0/system/auth-server/database`
- `infra/helm/charts/k1s0-common/templates/_vault-annotations.tpl`: `secret/data/k1s0/system/auth-server/database`

この不一致により、Vault Agent Injector が `auth/database` への 403 Permission Denied を返し、auth サービスが K8s 上で起動に失敗する。

## 決定
Vault シークレットパスは `secret/data/k1s0/{tier}/{service-name}/*` の形式に統一する。auth サービスのパスは `auth-server` とする（サービス名とコンテナ名の一貫性を保つため）。

## 理由
- `auth-server.hcl` ポリシー、SecretProviderClass、init-vault.sh の全てが `auth-server` を使用しており、変更コストが最小
- ADR-0045（Vault per-service role分離）の命名規約 `{service-name}-server` に準拠

## 影響

**ポジティブな影響**:
- auth サービスが K8s 上で正常起動できるようになる
- Vault ポリシー、SecretProviderClass、Helm values の三者間の一貫性が確保される

**ネガティブな影響・トレードオフ**:
- 既存の Vault KV ストアに保存済みのシークレットが `auth/database` パスに存在する場合、`auth-server/database` への移行が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 全ファイルを `auth` に統一 | values.yaml はそのまま、HCL/SPC を修正 | 変更ファイル数が多く、既存の `auth-server.hcl` 命名を変更する必要がある |
| 両パスを許可 | HCL で `auth/*` と `auth-server/*` 両方を許可 | 最小権限の原則に反する |

## 参考
- [ADR-0045](./0045-vault-per-service-roles.md) - Vault per-service role 分離
- `infra/vault/policies/auth-server.hcl`

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成（外部監査 CRIT-10 対応） | k1s0-team |
