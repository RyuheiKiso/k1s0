# ADR-0077: Vault business/service ティアのロール個別化

## ステータス

承認済み

## コンテキスト

ADR-0045 で system ティアの全26サービスは Vault Kubernetes auth ロールをサービス個別に分離済みである。
しかし business ティア（project-master-rust）と service ティア（task-rust, board-rust, activity-rust）は依然として共有ロール（"business" / "service"）を使用している。

共有ロールの問題点:
- 1サービスが侵害された場合、同ティア内の全シークレットにアクセス可能になる
- secret rotation を個別サービス単位で実施できない
- インシデント発生時のブラスト半径が同ティア全体に及ぶ

`infra/terraform/modules/vault/auth.tf` の現状:
- `vault_kubernetes_auth_backend_role.business`: project-master-rust が使用、`token_policies = ["business"]`
- `vault_kubernetes_auth_backend_role.service`: task-rust, board-rust, activity-rust が使用、`token_policies = ["service"]`

## 決定

business/service ティアも system ティア（ADR-0045 実装済み）と同様に、各サービス個別の Vault Kubernetes auth ロールとポリシーに分離する。

### Terraform auth.tf の修正計画

#### business ティア

現在の共有ロール `business` を廃止し、以下の個別ロールに移行する:

```hcl
# project-master-rust 個別 Vault ロール
resource "vault_kubernetes_auth_backend_role" "project_master_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "project-master-rust"
  bound_service_account_names      = ["project-master-rust"]
  bound_service_account_namespaces = ["k1s0-business"]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["business", "project-master"]
}
```

#### service ティア

現在の共有ロール `service` を廃止し、以下の個別ロールに移行する:

```hcl
# task-rust 個別 Vault ロール
resource "vault_kubernetes_auth_backend_role" "task_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "task-rust"
  bound_service_account_names      = ["task-rust"]
  bound_service_account_namespaces = ["k1s0-service"]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["service", "task"]
}

# board-rust 個別 Vault ロール
resource "vault_kubernetes_auth_backend_role" "board_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "board-rust"
  bound_service_account_names      = ["board-rust"]
  bound_service_account_namespaces = ["k1s0-service"]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["service", "board"]
}

# activity-rust 個別 Vault ロール
resource "vault_kubernetes_auth_backend_role" "activity_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "activity-rust"
  bound_service_account_names      = ["activity-rust"]
  bound_service_account_namespaces = ["k1s0-service"]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["service", "activity"]
}
```

### Vault ポリシーファイルの追加

`infra/vault/policies/` に以下のポリシーファイルを追加する:
- `project-master.hcl`: project-master 専用シークレットへのアクセス定義
- `task.hcl`: task 専用シークレットへのアクセス定義
- `board.hcl`: board 専用シークレットへのアクセス定義
- `activity.hcl`: activity 専用シークレットへのアクセス定義

### Helm values.yaml の更新

各サービスの Helm chart の `vault.role` を個別ロール名に変更する:
- `infra/helm/services/business/project-master/values.yaml`: `vault.role: "project-master-rust"`
- `infra/helm/services/service/task/values.yaml`: `vault.role: "task-rust"`
- `infra/helm/services/service/board/values.yaml`: `vault.role: "board-rust"`
- `infra/helm/services/service/activity/values.yaml`: `vault.role: "activity-rust"`

### 移行手順

1. 新しい個別ロールと個別ポリシーを Terraform で作成する（既存の共有ロールは残す）
2. 各サービスの Helm values.yaml の `vault.role` を個別ロール名に変更してデプロイする
3. 全サービスが個別ロールで正常に動作することを確認する
4. 古い共有ロール（"business", "service"）を Terraform から削除する

## 理由

- system ティアで ADR-0045 の実装により個別ロール化が完了しており、同じパターンを business/service ティアにも適用する
- 1サービスの認証情報漏洩が同ティア全体のシークレット漏洩に繋がるリスクを排除する
- インシデント発生時の爆発半径（blast radius）を単一サービスに限定できる
- サービス個別のシークレットローテーションが可能になる

## 影響

**ポジティブな影響**:

- 最小権限の原則（Principle of Least Privilege）に完全準拠する
- 1サービス侵害時の被害範囲を同サービスのシークレットのみに限定できる
- サービス個別の secret rotation が可能になり、セキュリティインシデント対応が迅速化する
- セキュリティ監査レポートの INFRA-HIGH-002 指摘が解消される

**ネガティブな影響・トレードオフ**:

- Terraform リソース数が増加する（共有4ロール → 個別4ロール）
- 移行作業中は新旧ロール両方が存在するため Terraform state が一時的に複雑になる
- 各サービスの Helm values.yaml の更新が必要になる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 現状維持 | business/service を共有ロールのまま維持 | ブラスト半径が大きく、セキュリティ要件を満たさない |
| ティア共通ポリシーの細分化 | 共有ロールを維持しつつポリシーでアクセス制御を絞る | ロール単位のシークレットローテーションができない |

## 参考

- [ADR-0045: Vault サービス個別ロール分離](0045-vault-per-service-roles.md)
- [infra/terraform/modules/vault/auth.tf](../../../../infra/terraform/modules/vault/auth.tf)
- [Vault Kubernetes Auth Method](https://developer.hashicorp.com/vault/docs/auth/kubernetes)
- [INFRA-HIGH-002 監査指摘: business/service ティア Vault ロール共有問題]

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-03 | 初版作成（INFRA-HIGH-002 対応） | @k1s0 |
