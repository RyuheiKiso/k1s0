# ADR-0025: Terraform State Backend を Consul から Ceph RGW（S3互換）に移行

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

### ADR-0015 との関係

ADR-0015「S3/AWS SDK 依存の完全削除」は**アプリケーションレイヤー**（Rust/Go サーバー、クライアントライブラリ）を対象としており、インフラ管理ツール（Terraform）のバックエンドとしての S3 互換ストレージ利用は対象外である。Terraform State Backend は開発者がインフラデプロイ時にのみ使用するインフラ管理レイヤーであり、アプリケーションの実行時依存には含まれない。

この設定は外部監査 H-08 で以下の問題点が指摘された:

1. **SPOF（単一障害点）リスク**: Consul クラスタが停止すると全環境で `terraform plan` / `terraform apply` が実行不能になる。インフラ変更のデプロイが完全に停止するリスクがある。
2. **at-rest encryption の欠如**: Consul の KV ストアは標準で暗号化されておらず、State ファイルに含まれる機密情報（パスワード、API キー、証明書等）が平文で保存される。
3. **バージョン管理なし**: Consul KV は State の変更履歴を保持しないため、誤った `terraform apply` からのロールバックが困難。
4. **監査ログの不備**: Consul ACL audit log は別途設定が必要で、デフォルトでは誰がいつ State を変更したかの追跡ができない。
5. **業界標準からの乖離**: Terraform の公式ドキュメントおよび業界標準では S3 系のバックエンドが推奨されており、Consul backend の採用事例は限定的である。

## 決定

全環境（dev / staging / prod）の `backend.tf` を Consul backend から Ceph RGW（S3 互換）backend に変更する。

k1s0 は AWS を使用せず、Kubernetes 上の Ceph クラスタを使用している。Terraform の `backend "s3"` は S3 互換 API に対応しているため、Ceph RGW を State 保存先として使用する。State ロックは DynamoDB の代わりに Terraform 1.10 以降の `use_lockfile = true`（S3 上にロックファイルを作成）を使用する。

```hcl
# 変更後の状態（Ceph RGW backend）
terraform {
  backend "s3" {
    bucket         = "k1s0-terraform-state-${environment}"
    key            = "k1s0/${environment}/terraform.tfstate"
    region         = "dummy"        # Ceph RGW では使用しないが required フィールドのためダミー値を設定
    endpoint       = "https://rgw.internal.example.com"  # Ceph RGW エンドポイント
    use_path_style = true           # Ceph RGW はパススタイル URL を使用
    use_lockfile   = true           # S3 上にロックファイルを作成（Terraform 1.10+）
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

### Ceph RGW（S3互換）を選択した理由

1. **既存インフラとの統合**: k1s0 は Kubernetes + Ceph を基盤としており、Ceph RGW が S3 互換 API を提供している。追加のインフラコストなしに利用可能。

2. **高可用性**: Ceph は分散ストレージとして複数ノードに冗長化されており、単一 Consul クラスタへの依存を排除し SPOF を解消できる。

3. **at-rest 暗号化**: Ceph OSD レベルの暗号化、および `encrypt = true` による SSE を組み合わせることで機密情報を保護できる。

4. **バージョン管理**: Ceph RGW のバケットバージョニングにより State ファイルの変更履歴を保存できる。誤った apply からの State ロールバックが容易になる。

5. **State ロック**: Terraform 1.10 以降の `use_lockfile = true` により、S3 上にロックファイルを作成して並行実行による State 破損を防止できる。DynamoDB（AWS 固有サービス）は不要。

6. **Terraform 標準機能**: `backend "s3"` は S3 互換エンドポイントをサポートしており、Ceph RGW に対しても標準的に使用できる。

### 環境分離の設計方針

環境ごとにバケットを分離（`k1s0-terraform-state-{env}`）することで、環境間の State 混在を防ぐ。ロックキー（`{bucket}/{key}.tflock`）は環境ごとに異なるため競合しない。

## 影響

**ポジティブな影響**:

- Consul クラスタ障害時でも Terraform 操作が継続可能になり、SPOF が解消される
- State ファイルの at-rest 暗号化により、機密情報の漏洩リスクが大幅に低下する
- Ceph バケットバージョニングにより State の変更履歴が保存され、障害時のロールバックが容易になる
- Ceph RGW のアクセスログにより、State への全アクセスが記録される
- 外部監査 H-08 の指摘事項が解消される
- AWS に依存しない構成を維持できる

**ネガティブな影響・トレードオフ**:

- Ceph RGW に S3 バケット（3環境分）の追加リソースが必要になる
- `terraform init -migrate-state` の実行が必要で、移行作業中は一時的に Terraform 操作を停止する必要がある
- Consul に依存していた既存の運用手順（監視、ダッシュボード等）の更新が必要になる
- Terraform 1.10 以降が必要（`use_lockfile` 対応バージョン）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: Consul backend を継続 | 現状維持 | 外部監査 H-08 の要件を満たさない。SPOF・暗号化欠如の問題が継続する |
| 案 B: AWS S3+DynamoDB | AWS のマネージド S3 + DynamoDB ロックを使用 | k1s0 は AWS を使用しない方針（ADR-0015）に反する |
| 案 C: Terraform Cloud | HashiCorp の SaaS State 管理サービスを使用 | 外部 SaaS への依存が増え、オンプレ・プライベートクラウド要件と相容れない。ライセンスコストも発生する |
| 案 D: HTTP backend（独自実装） | カスタム HTTP サーバーで State を管理 | 独自実装のメンテナンスコストが高く、セキュリティ上のリスクも増える |
| 案 E: MinIO | MinIO サーバーを別途構築して使用 | Ceph RGW が既に稼働しており、追加インフラが不要 |

## 未対応事項

### KES（Key Encryption Service）による追加暗号化

**現状**: Ceph OSD レベルの at-rest encryption で暗号化済み（インフラチーム管理）。

**追加対応（推奨）**: MinIO KES + Vault 統合による暗号化レイヤーを追加することで、より強固な鍵管理が可能になる。

### Ceph RGW バケットバージョニング

S3 バージョニングはバケット設定で有効化する（`backend.tf` では設定不可）。
Ceph RGW の管理者がバケット作成後にバージョニングを有効化すること（手順は移行手順書参照）。

## 参考

- [Terraform S3 Backend 公式ドキュメント](https://developer.hashicorp.com/terraform/language/backend/s3)
- [Terraform S3 Backend use_lockfile](https://developer.hashicorp.com/terraform/language/backend/s3#use_lockfile)
- [移行手順書](../../infrastructure/terraform-state-migration.md)
- 外部監査報告書 H-08: Terraform State Backend の SPOF リスクと暗号化欠如
- [ADR-0015: S3/AWS SDK依存の完全削除](0015-remove-s3-dependency.md)
- [ADR-0019: Vault ドメイン別シークレット分離](0019-vault-domain-secret-isolation.md)
