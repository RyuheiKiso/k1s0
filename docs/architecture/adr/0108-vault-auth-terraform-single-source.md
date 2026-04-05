# ADR-0108: Vault 認証設定の Terraform 一元管理（ConfigMap 廃止）

## ステータス

承認済み

## コンテキスト

Vault 認証設定は2か所で管理されており、不整合が生じていた:

### 現状

1. **`infra/vault/auth/auth.tf`**: Terraform による Vault Kubernetes auth メソッドの設定。
   Phase 5（サービス個別ロール実装）として、全33サービスの個別ポリシー・TTL 設定が完了済み。

2. **`infra/vault/auth/` ディレクトリの ConfigMap（6ファイル）**:
   - `k1s0-system-auth.yaml`
   - `k1s0-system-config.yaml`
   - `k1s0-system-dlq.yaml`
   - `k1s0-system-saga.yaml`
   - `k1s0-system-ai-gateway.yaml`
   - `k1s0-system-ai-agent.yaml`

これらの ConfigMap は過去の手動適用フェーズ（Phase 1〜4）の参照用として作成されたものであり、
現在は `auth.tf` による Terraform 管理が完了している。

### 問題点

`auth.tf` と ConfigMap の間に以下の不整合が確認された:

- **ポリシー名の不一致**: ConfigMap では `k1s0-config-policy` と記載されているが、
  `auth.tf` では `k1s0-config-server-policy` として定義されている
- **TTL の不一致**: ConfigMap では `token_ttl = "1h"` と記載されているが、
  `auth.tf` では `token_ttl = "30m"` と定義されているサービスがある
- **対象サービスの漏れ**: ConfigMap には6サービス分しか存在しないが、
  `auth.tf` では33サービスすべてが管理されている

ConfigMap が「正」として扱われると、`auth.tf` との乖離を修正する際の混乱を招く。

## 決定

**Terraform（`auth.tf`）を Vault 認証設定の唯一の権威ソース（Single Source of Truth）とする。**

`infra/vault/auth/` ディレクトリ内の以下6ファイルを削除する:

- `k1s0-system-auth.yaml`
- `k1s0-system-config.yaml`
- `k1s0-system-dlq.yaml`
- `k1s0-system-saga.yaml`
- `k1s0-system-ai-gateway.yaml`
- `k1s0-system-ai-agent.yaml`

今後、Vault 認証の設定変更はすべて `auth.tf` を通じて行い、`terraform apply` で適用する。

## 理由

- **Phase 5 完了済み**: `auth.tf` によるサービス個別ロール実装は既に完了しており、
  ConfigMap は古いリファレンスに過ぎない

- **不整合の根本除去**: 2つのソースが存在する限り、どちらが「正」かについての混乱が継続する。
  ConfigMap を削除することで、唯一の正のソースが明確になる

- **Infrastructure as Code の原則**: Terraform による設定管理はバージョン管理・レビュー・
  ロールバックが可能であり、手動 YAML 適用より信頼性が高い

- **CI/CD との統合**: `auth.tf` は `terraform plan` / `terraform apply` でドリフト検出・修正が可能だが、
  ConfigMap は `kubectl apply` の実行有無が追跡困難

## 影響

**ポジティブな影響**:

- Vault 認証設定の権威ソースが一元化され、不整合が解消される
- `terraform plan` によりドリフト（手動変更）を自動検出できる
- `auth.tf` だけを参照すれば全33サービスの設定が確認できる

**ネガティブな影響・トレードオフ**:

- ConfigMap を参照していた運用手順書・ドキュメントの更新が必要
- `infra/vault/auth/` ディレクトリが空になるため、README などで Terraform への誘導が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: ConfigMap を更新して auth.tf と整合させる | ポリシー名・TTL を auth.tf に合わせて修正する | 二重管理の問題が解消されない。将来的に再び乖離が発生するリスクが残る |
| 案 B: auth.tf を削除し ConfigMap のみを維持する | kubectl apply による手動管理に戻す | Terraform による宣言的管理の利点（ドリフト検出・レビュー・CI 統合）が失われる |
| 案 C: ConfigMap を Terraform の tfvars として再活用する | ConfigMap の内容を Terraform 変数ファイルとして変換する | 不要な変換作業が生じる。auth.tf が既に完全な情報を持っているため冗長 |

## 参考

- `infra/vault/auth/auth.tf` — Terraform による Vault 認証設定
- [ADR-0045: Vault サービス個別ロール分離](0045-vault-per-service-role-isolation.md)
- [ADR-0068: Vault Phase 5 個別ロール実装](0068-vault-phase5-individual-roles.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-05 | 初版作成 | @k1s0-team |
