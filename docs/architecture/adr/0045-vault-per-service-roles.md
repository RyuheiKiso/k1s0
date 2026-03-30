# ADR-0045: Vault サービス個別 Kubernetes Auth ロール実装計画

## ステータス

実装完了（2026-03-29）

## コンテキスト

ADR-0019（Vault ポリシーのドメイン単位シークレット分離）において、各サービスが自ドメインのシークレットにのみアクセスできるよう個別ポリシーを設ける方針が決定された。

外部技術監査（L-14）で以下の問題が指摘された。

- `infra/terraform/modules/vault/auth.tf` では system tier の全 27 サービスが単一 `system` ロールに集約されている
- `infra/vault/policies/` 配下には各サービス用の個別 HCL ポリシーファイルが既に存在するが、Kubernetes auth ロールとの紐付けがされていない
- 1 サービスが侵害された場合、同 tier 内の全サービスのシークレットにアクセス可能なリスクがある

現在の単一 `system` ロールの構成:

```hcl
# 全 27 SA が単一ロールに集約（問題のある現状）
resource "vault_kubernetes_auth_backend_role" "system" {
  role_name               = "system"
  bound_service_account_names = [
    "auth-rust", "bff-proxy-sa", "config-rust", "dlq-manager", ...（27 SA）
  ]
  token_policies = ["system"]  # 単一の粗粒度ポリシー
}
```

## 決定

Phase 5 にて system tier の全 27 サービスに対して個別の Kubernetes auth ロールを作成し、
既存の個別 HCL ポリシーファイル（`infra/vault/policies/{service}.hcl`）と紐付ける。

### 実装方針

各サービスに対して以下の形式でロールを作成する:

```hcl
# サービス個別ロールの例（auth-rust）
resource "vault_kubernetes_auth_backend_role" "auth_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "auth-rust"
  bound_service_account_names      = ["auth-rust"]
  bound_service_account_namespaces = ["k1s0-system"]
  token_policies                   = ["auth-rust"]  # 個別ポリシーを参照
  token_ttl                        = 3600
  token_max_ttl                    = 14400
}
```

### 対象サービス（27 SA）

| SA 名 | ポリシーファイル |
|-------|---------------|
| auth-rust | auth-server.hcl |
| bff-proxy-sa | bff-proxy.hcl |
| config-rust | config-server.hcl |
| dlq-manager | dlq-manager.hcl |
| event-store-rust | event-store.hcl |
| featureflag-rust | featureflag.hcl |
| file-rust | file.hcl |
| graphql-gateway | graphql-gateway.hcl |
| master-maintenance | master-maintenance.hcl |
| navigation-rust | navigation.hcl |
| notification-rust | notification.hcl |
| policy-rust | policy.hcl |
| quota-rust | quota.hcl |
| ratelimit-rust | ratelimit.hcl |
| rule-engine-rust | rule-engine.hcl |
| saga-rust | saga-server.hcl |
| scheduler-rust | scheduler.hcl |
| search-rust | search.hcl |
| service-catalog | service-catalog.hcl |
| session-rust | session.hcl |
| tenant-rust | tenant.hcl |
| vault-rust | vault-server.hcl |
| workflow-rust | workflow.hcl |
| event-monitor-rust | event-monitor.hcl |
| app-registry | app-registry.hcl |
| api-registry-rust | api-registry.hcl |
| ai-agent（将来予定） | ai-agent.hcl |

### 移行手順

1. 個別ロールを `auth.tf` に追記する（既存の `system` ロールは残したまま）
2. 各サービスの Vault ロール設定（`values.yaml` の `vault.role`）を個別ロール名に変更する
3. 動作確認後、既存の単一 `system` ロールを削除する

## 理由

1. **最小権限の原則**: 各サービスは自サービスのシークレットにのみアクセスを限定できる
2. **侵害影響範囲の局所化**: あるサービスが侵害されても影響を1サービスのシークレットに限定できる
3. **ADR-0019 との整合**: business/service tier で実施済みのドメイン分離方針を system tier にも適用する
4. **ポリシーファイルの活用**: 既存の個別 HCL ポリシーファイルが未使用のまま放置されている状態を解消する

## 影響

**ポジティブな影響**:

- 侵害時の爆発半径（Blast Radius）をサービス単位に限定できる
- Vault 監査ログでどのサービスがどのシークレットにアクセスしたかを追跡しやすくなる
- ADR-0019 の方針が system tier にも完全に適用される

**ネガティブな影響・トレードオフ**:

- 27 サービス分の `vault_kubernetes_auth_backend_role` リソースを追加するため、`auth.tf` が大幅に増大する
- 各サービスの `values.yaml` で `vault.role` の変更が必要になる（27 ファイルの変更）
- 移行期間中は旧 `system` ロールと新個別ロールが並行するため Terraform の管理が複雑になる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 現状維持（単一 system ロール） | 現在の集約ロールを継続 | 最小権限の原則に反する。L-14 監査指摘事項が未対応のままとなる |
| Vault ネームスペース分離 | tier ごとに Vault Namespace を分割 | Vault Enterprise 機能が必要。OSS 版では利用不可 |
| SA グループによる中間集約 | 関連サービスをグループ単位（4〜6 SA）でロール集約 | 依然として侵害影響がグループ全体に及ぶため本質的解決にならない |

## 参考

- [ADR-0019: Vault ポリシーのドメイン単位シークレット分離](./0019-vault-domain-secret-isolation.md) — 本 ADR の前提となる分離方針
- [外部監査対応 2026-03-29](../../memory/project_audit_response_2026_03_29.md) — L-14 監査指摘事項
- 個別ポリシーファイル: `infra/vault/policies/`（27 ファイル）
- 現行認証設定: `infra/terraform/modules/vault/auth.tf`

## 後続対応（未実装）

以下のサービスはポリシーファイルが存在するが、現時点で system tier の SA として登録されておらず個別ロールを作成していない。
将来サービスが追加された場合は `auth.tf` にロールを追記すること。

| ポリシーファイル | 備考 |
|---------------|------|
| ai-agent.hcl | AI エージェントサービス（将来追加予定） |
| ai-gateway.hcl | AI ゲートウェイサービス（将来追加予定） |
| k1s0-system.hcl | tier 共通ポリシー（個別ロールには不要） |

~~各サービスの `values.yaml` の `vault.role` を個別ロール名に変更し、動作確認後に既存の単一 `system` ロールを削除すること。~~

H-03 対応（2026-03-30）にて、全 system tier サービス（24 サービス）の `values.yaml` を個別ロールに移行完了。
残作業: 全サービスのローリングアップデート完了後、Vault 監査ログで "system" ロールへのアクセスが 0 件になったことを確認してから `auth.tf` の単一 `system` ロール（H-04）を削除すること。

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成（L-14 監査対応） | - |
| 2026-03-29 | auth.tf に 26 サービス分の個別ロールを実装、variables.tf に k8s_namespace 変数を追加（H-02/L-14 対応完了） | - |
| 2026-03-30 | 全 system tier サービス（24 SA）の Helm values.yaml を個別 Vault ロールに移行（H-03 対応）。auth.tf の旧 "system" ロールに削除予定コメントを追加（H-04）。 | - |
