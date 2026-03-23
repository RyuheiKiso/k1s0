# Terraform State Backend 移行手順: Consul → S3+DynamoDB

## 概要

外部監査 H-08 の指摘に基づき、Terraform State の管理 backend を Consul から S3+DynamoDB へ移行する。
本ドキュメントでは移行前の準備、移行手順、ロールバック手順を説明する。

## 背景と移行理由

| 項目 | Consul (旧) | S3+DynamoDB (新) |
|------|-------------|-----------------|
| 高可用性 | 単一クラスタが SPOF になりうる | S3 は AWS マネージドで 99.999999999% の耐久性 |
| at-rest 暗号化 | 非対応（ACL のみ） | SSE-S3 / SSE-KMS による暗号化を標準サポート |
| State ロック | consul lock（アドバイザリ） | DynamoDB による強整合性ロック |
| 監査ログ | Consul audit log（要設定） | S3 アクセスログ + CloudTrail で標準取得可能 |
| バージョニング | なし | S3 バージョニングで変更履歴を保持 |
| 業界標準 | 独自実装 | Terraform 公式推奨バックエンド |

## 前提条件

- AWS CLI が設定済みであること（`aws configure` または IAM ロール）
- 対象 AWS アカウントに S3 および DynamoDB の作成権限があること
- 既存の Consul backend に接続可能な状態で移行作業を開始すること

---

## Step 1: S3 バケットの作成

環境ごとにバケットを作成する。バケット名はグローバルに一意である必要がある。

```bash
# prod 環境用バケット作成
aws s3api create-bucket \
  --bucket k1s0-terraform-state-prod \
  --region ap-northeast-1 \
  --create-bucket-configuration LocationConstraint=ap-northeast-1

# staging 環境用バケット作成
aws s3api create-bucket \
  --bucket k1s0-terraform-state-staging \
  --region ap-northeast-1 \
  --create-bucket-configuration LocationConstraint=ap-northeast-1

# dev 環境用バケット作成
aws s3api create-bucket \
  --bucket k1s0-terraform-state-dev \
  --region ap-northeast-1 \
  --create-bucket-configuration LocationConstraint=ap-northeast-1
```

### バージョニングを有効化

```bash
# 全環境のバケットにバージョニングを設定
for ENV in prod staging dev; do
  aws s3api put-bucket-versioning \
    --bucket "k1s0-terraform-state-${ENV}" \
    --versioning-configuration Status=Enabled
done
```

### SSE-S3 暗号化を有効化

```bash
# 全環境のバケットにサーバーサイド暗号化を設定
for ENV in prod staging dev; do
  aws s3api put-bucket-encryption \
    --bucket "k1s0-terraform-state-${ENV}" \
    --server-side-encryption-configuration '{
      "Rules": [{
        "ApplyServerSideEncryptionByDefault": {
          "SSEAlgorithm": "AES256"
        },
        "BucketKeyEnabled": true
      }]
    }'
done
```

### パブリックアクセスをブロック

```bash
# 全環境のバケットのパブリックアクセスを完全遮断
for ENV in prod staging dev; do
  aws s3api put-public-access-block \
    --bucket "k1s0-terraform-state-${ENV}" \
    --public-access-block-configuration \
      BlockPublicAcls=true,IgnorePublicAcls=true,BlockPublicPolicy=true,RestrictPublicBuckets=true
done
```

### アクセスログを有効化

```bash
# ログ収集用バケットを作成（全環境共有）
aws s3api create-bucket \
  --bucket k1s0-terraform-state-logs \
  --region ap-northeast-1 \
  --create-bucket-configuration LocationConstraint=ap-northeast-1

# 全環境バケットのアクセスログを設定
for ENV in prod staging dev; do
  aws s3api put-bucket-logging \
    --bucket "k1s0-terraform-state-${ENV}" \
    --bucket-logging-status '{
      "LoggingEnabled": {
        "TargetBucket": "k1s0-terraform-state-logs",
        "TargetPrefix": "'"${ENV}"'/"
      }
    }'
done
```

---

## Step 2: DynamoDB テーブルの作成（State ロック用）

全環境で共通の DynamoDB テーブルを使用する。ロックキーに環境名が含まれるため1テーブルで共存可能。

```bash
# State ロック用 DynamoDB テーブルを作成
aws dynamodb create-table \
  --table-name k1s0-terraform-state-lock \
  --attribute-definitions AttributeName=LockID,AttributeType=S \
  --key-schema AttributeName=LockID,KeyType=HASH \
  --billing-mode PAY_PER_REQUEST \
  --region ap-northeast-1

# テーブルが ACTIVE になるまで待機
aws dynamodb wait table-exists \
  --table-name k1s0-terraform-state-lock \
  --region ap-northeast-1
```

---

## Step 3: IAM ポリシーの設定

Terraform を実行する IAM ロール / ユーザーに以下の権限を付与する。

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "TerraformStateS3Access",
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::k1s0-terraform-state-*",
        "arn:aws:s3:::k1s0-terraform-state-*/*"
      ]
    },
    {
      "Sid": "TerraformStateLockDynamoDB",
      "Effect": "Allow",
      "Action": [
        "dynamodb:GetItem",
        "dynamodb:PutItem",
        "dynamodb:DeleteItem"
      ],
      "Resource": "arn:aws:dynamodb:ap-northeast-1:*:table/k1s0-terraform-state-lock"
    }
  ]
}
```

---

## Step 4: terraform init -migrate-state の実行

各環境のディレクトリで以下を実行する。`-migrate-state` フラグにより Consul から S3 へ State が自動コピーされる。

```bash
# prod 環境の移行
cd infra/terraform/environments/prod
terraform init -migrate-state

# staging 環境の移行
cd infra/terraform/environments/staging
terraform init -migrate-state

# dev 環境の移行
cd infra/terraform/environments/dev
terraform init -migrate-state
```

実行時に以下のプロンプトが表示されるので `yes` を入力する:

```
Do you want to copy existing state to the new backend?
  Pre-existing state was found while migrating the previous "consul" backend to the
  newly configured "s3" backend. No existing state was found in the newly configured
  "s3" backend. Do you want to copy this state to the new backend?

  Enter a value: yes
```

---

## Step 5: 移行後の検証

```bash
# S3 に State ファイルが存在することを確認
aws s3 ls s3://k1s0-terraform-state-prod/k1s0/prod/
aws s3 ls s3://k1s0-terraform-state-staging/k1s0/staging/
aws s3 ls s3://k1s0-terraform-state-dev/k1s0/dev/

# terraform plan でドリフトがないことを確認（変更なしが期待値）
cd infra/terraform/environments/prod && terraform plan
cd infra/terraform/environments/staging && terraform plan
cd infra/terraform/environments/dev && terraform plan
```

---

## ロールバック手順

移行後に問題が発生した場合、以下の手順で Consul backend に戻す。

> **注意**: ロールバック前に必ず S3 上の State をバックアップしておくこと。

### 1. backend.tf のコメントアウトを解除

各環境の `backend.tf` で、Consul 設定のコメントアウトを解除し、S3 設定をコメントアウトする。

### 2. terraform init -migrate-state を再実行

```bash
# S3 → Consul に State を戻す
cd infra/terraform/environments/prod
terraform init -migrate-state
# プロンプトで yes を入力
```

### 3. Consul の State を確認

```bash
# Consul 上に State が存在することを確認
consul kv get terraform/k1s0/prod
```

---

## 参考

- [Terraform S3 Backend ドキュメント](https://developer.hashicorp.com/terraform/language/backend/s3)
- [AWS S3 バージョニング](https://docs.aws.amazon.com/AmazonS3/latest/userguide/Versioning.html)
- [ADR-0025: Terraform State Backend を S3+DynamoDB に移行](../architecture/adr/0025-terraform-state-s3.md)
- 外部監査報告書 H-08: Terraform State Backend の SPOF リスク
