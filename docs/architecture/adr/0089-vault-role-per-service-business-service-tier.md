# ADR-0089: Vault ロール個別化の business/service tier 拡張

## ステータス

承認済み

## コンテキスト

ADR-0045 において Vault ロールのサービス単位個別化を決定した。しかし 2026-04-04 の外部技術監査（K8S-HIGH-002〜004）にて、以下の 6 サービスが依然として tier 共通ロール（"system" / "business" / "service"）を使用していることが指摘された。

| サービス | tier | 旧ロール | 新ロール |
|---------|------|--------|--------|
| api-registry | system | system | api-registry-rust |
| app-registry | system | system | app-registry |
| project-master | business | business | project-master-rust |
| task | service | service | task-rust |
| board | service | service | board-rust |
| activity | service | service | activity-rust |

Terraform の `infra/terraform/vault/auth.tf` には既に各サービス個別のロール定義が存在しており、Helm values.yaml 側の `vault.role` 値のみが旧来の共通ロール名のまま残っていた。これにより、あるサービスが侵害された場合に同一 tier の他サービスのシークレットにアクセスできるという最小権限原則違反が生じていた。

## 決定

ADR-0045 で決定した Vault ロールのサービス単位個別化を business tier（project-master）と service tier（task / board / activity）にも適用し、system tier の残り 2 サービス（api-registry / app-registry）についても同様に修正する。

各サービスの `infra/helm/services/{tier}/{service}/values.yaml` における `vault.role` を、Terraform で定義済みのサービス個別ロール名に変更する。

## 理由

- Terraform 側には個別ロール定義が既に存在するため、Helm values.yaml の変更のみで対応完了できる。
- tier 共通ロールは最小権限原則に違反し、横断的な侵害リスクを拡大させる。
- ADR-0045 の決定を一貫して適用することでセキュリティポリシーの統一性が保たれる。

## 影響

**ポジティブな影響**:

- 各サービスが自身のシークレットのみにアクセス可能となり、最小権限原則が徹底される。
- サービス侵害時の横断的シークレット漏洩リスクが排除される。
- ADR-0045 の適用範囲が全 tier に拡大され、ポリシーの一貫性が向上する。

**ネガティブな影響・トレードオフ**:

- Terraform の Vault ロール定義（auth.tf）が既に存在していることが前提であり、存在しない場合はデプロイ時に Vault 認証エラーとなる。
- ロール名変更による一時的なサービス再起動が必要になる場合がある。

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| tier 共通ロール継続 | 現状維持 | 最小権限原則違反・外部監査指摘事項のため採用不可 |
| Terraform 側を共通ロールに戻す | Helm 変更なしで対応 | セキュリティ後退であり ADR-0045 の精神に反する |

## 参考

- [ADR-0045: Vault per-service ロール分離](0045-vault-per-service-role.md)
- K8S-HIGH-002〜004: 2026-04-04 外部技術監査指摘事項
- `infra/terraform/vault/auth.tf`: サービス個別 Vault ロール定義

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（外部監査 K8S-HIGH-002〜004 対応） | @k1s0-team |
