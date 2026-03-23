# ADR-0025: Terraform State Backend を Consul から S3+DynamoDB に移行

## ステータス

承認済み

## コンテキスト

`infra/terraform/environments/` 配下の全環境（dev / staging / prod）において、Terraform State の管理 backend として Consul を使用していた。

```hcl
# 変更前の状態（問題のある設定）
terraform {
  backend "consul" {
    address = "consul.internal.example.com:8500"
    scheme  = "https"
    path    = "terraform/k1s0/prod"
    lock    = true
  }
}
```

この設定は外部監査 H-08 で以下の問題点が指摘された:

1. **SPOF（単一障害点）リスク**: Consul クラスタが停止すると全環境で `terraform plan` / `terraform apply` が実行不能になる。インフラ変更のデプロイが完全に停止するリスクがある。
2. **at-rest encryption の欠如**: Consul の KV ストアは標準で暗号化されておらず、State ファイルに含まれる機密情報（パスワード、API キー、証明書等）が平文で保存される。
3. **バージョン管理なし**: Consul KV は State の変更履歴を保持しないため、誤った `terraform apply` からのロールバックが困難。
4. **監査ログの不備**: Consul ACL audit log は別途設定が必要で、デフォルトでは誰がいつ State を変更したかの追跡ができない。
5. **業界標準からの乖離**: Terraform の公式ドキュメントおよび業界標準では S3+DynamoDB（AWS）または GCS+Cloud Storage（GCP）が推奨されており、Consul backend の採用事例は限定的である。

## 決定

全環境（dev / staging / prod）の `backend.tf` を Consul backend から S3+DynamoDB backend に変更する。

```hcl
# 変更後の状態（S3+DynamoDB backend）
terraform {
  backend "s3" {
    bucket         = "k1s0-terraform-state-${environment}"
    key            = "k1s0/${environment}/terraform.tfstate"
    region         = "ap-northeast-1"
    dynamodb_table = "k1s0-terraform-state-lock"
    encrypt        = true
  }
}
```

対象ファイル:
- `infra/terraform/environments/prod/backend.tf`
- `infra/terraform/environments/staging/backend.tf`
- `infra/terraform/environments/dev/backend.tf`

移行手順の詳細は `docs/infrastructure/terraform-state-migration.md` を参照すること。

## 理由

### S3+DynamoDB を選択した理由

1. **高可用性**: S3 は AWS によるマネージドサービスで、SLA 99.99%・耐久性 99.999999999%（イレブンナイン）を保証する。単一 Consul クラスタへの依存を排除し SPOF を解消できる。

2. **at-rest 暗号化**: `encrypt = true` を指定することで SSE-S3（AES-256）による透過的な暗号化が有効になる。KMS CMK（`kms_key_id`）を設定すれば追加の暗号化レイヤーも適用可能。

3. **強整合性のロック**: DynamoDB の条件付き書き込み（`PutItem` with `attribute_not_exists`）を利用した State ロックにより、並行実行による State 破損を防止できる。Consul のアドバイザリロックと比較して確実性が高い。

4. **バージョン管理**: S3 バージョニングにより State ファイルの変更履歴が自動保存される。誤った apply からの State ロールバックが容易になる。

5. **監査ログ**: S3 アクセスログおよび CloudTrail により、誰がいつ State を読み書きしたかを標準的に追跡できる。コンプライアンス要件への対応が容易。

6. **Terraform 公式推奨**: HashiCorp の公式ドキュメントおよびベストプラクティスガイドで S3+DynamoDB が推奨されており、コミュニティのサポートが充実している。

7. **既存 AWS インフラとの統合**: k1s0 は AWS 上で運用されており、S3 および DynamoDB は既に利用中のサービスである。追加のインフラ維持コストが最小限で済む。

### 環境分離の設計方針

環境ごとに S3 バケットを分離（`k1s0-terraform-state-{env}`）することで、環境間の State 混在を防ぐ。DynamoDB テーブルは全環境で共有するが、ロックキー（`LockID`）に環境名が含まれるため競合しない。

## 影響

**ポジティブな影響**:

- Consul クラスタ障害時でも Terraform 操作が継続可能になり、SPOF が解消される
- State ファイルの at-rest 暗号化により、機密情報の漏洩リスクが大幅に低下する
- S3 バージョニングにより State の変更履歴が保存され、障害時のロールバックが容易になる
- CloudTrail + S3 アクセスログにより、State への全アクセスが監査ログとして記録される
- 外部監査 H-08 の指摘事項が解消される
- Terraform 公式推奨構成に準拠し、運用ノウハウの習得・共有が容易になる

**ネガティブな影響・トレードオフ**:

- AWS アカウントに S3 バケット（3環境分 + ログ用）と DynamoDB テーブル（1件）の追加リソースが必要になる
- `terraform init -migrate-state` の実行が必要で、移行作業中は一時的に Terraform 操作を停止する必要がある
- Consul に依存していた既存の運用手順（監視、ダッシュボード等）の更新が必要になる
- S3 ストレージコストが発生するが、State ファイルのサイズは通常 MB 以下であり費用は軽微

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: Consul backend を継続 | 現状維持 | 外部監査 H-08 の要件を満たさない。SPOF・暗号化欠如の問題が継続する |
| 案 B: GCS+Cloud Storage | Google Cloud Storage を State backend に使用 | k1s0 は AWS 上で運用されており、GCP サービスの追加導入はインフラの複雑度を増す |
| 案 C: Terraform Cloud | HashiCorp の SaaS State 管理サービスを使用 | 外部 SaaS への依存が増え、オンプレ・プライベートクラウド要件と相容れない。ライセンスコストも発生する |
| 案 D: HTTP backend（独自実装） | カスタム HTTP サーバーで State を管理 | 独自実装のメンテナンスコストが高く、セキュリティ上のリスクも増える |
| 案 E: Azure Blob Storage | Azure Storage を State backend に使用 | 案 B と同様に AWS 以外のクラウドサービスへの依存が生じる |

## 参考

- [Terraform S3 Backend 公式ドキュメント](https://developer.hashicorp.com/terraform/language/backend/s3)
- [Terraform State 管理ベストプラクティス](https://developer.hashicorp.com/terraform/tutorials/aws-get-started/aws-remote)
- [移行手順書](../../infrastructure/terraform-state-migration.md)
- 外部監査報告書 H-08: Terraform State Backend の SPOF リスクと暗号化欠如
- [ADR-0019: Vault ドメイン別シークレット分離](0019-vault-domain-secret-isolation.md)
