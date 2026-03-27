# Terraform State Backend 移行手順: Consul → Ceph RGW（S3互換）

## 概要

外部監査 H-08 の指摘に基づき、Terraform State の管理 backend を Consul から Ceph RGW（S3互換）へ移行する。
k1s0 は AWS を使用しないため、既存の Ceph クラスタの RGW（RADOS Gateway）を S3 互換ストレージとして使用する。
本ドキュメントでは移行前の準備、移行手順、ロールバック手順を説明する。

## 背景と移行理由

| 項目 | Consul (旧) | Ceph RGW (新) |
|------|-------------|----------------|
| 高可用性 | 単一クラスタが SPOF になりうる | Ceph の分散アーキテクチャで冗長化済み |
| at-rest 暗号化 | 非対応（ACL のみ） | Ceph OSD レベルの暗号化 + SSE をサポート |
| State ロック | consul lock（アドバイザリ） | S3 ロックファイル（Terraform 1.10+）で競合防止 |
| 監査ログ | Consul audit log（要設定） | Ceph RGW アクセスログで標準取得可能 |
| バージョニング | なし | Ceph バケットバージョニングで変更履歴を保持 |
| AWS 依存 | なし | なし（既存 Ceph インフラを活用） |

## 前提条件

- Ceph クラスタが稼働済みで RGW が有効になっていること
- `radosgw-admin` コマンドが実行可能なこと（Ceph 管理ノード上）
- Terraform 1.10 以降がインストール済みであること（`use_lockfile` 対応）
- 既存の Consul backend に接続可能な状態で移行作業を開始すること

---

## Step 1: Ceph RGW ユーザーとバケットの作成

### RGW ユーザーを作成する

```bash
# Terraform 専用の RGW ユーザーを作成
radosgw-admin user create \
  --uid="terraform-state" \
  --display-name="Terraform State Backend" \
  --caps="buckets=*"

# 認証情報を取得（access_key と secret_key を記録しておく）
radosgw-admin user info --uid="terraform-state" | jq '.keys[0]'
```

### 環境ごとのバケットを作成する

```bash
# Ceph 管理ノードで実行、または mc（MinIO Client）を使用
for ENV in prod staging dev; do
  # RGW にバケットを作成
  radosgw-admin bucket create \
    --bucket="k1s0-terraform-state-${ENV}" \
    --uid="terraform-state"
done
```

または MinIO Client（mc）を使用する場合:

```bash
# Ceph RGW エンドポイントを登録
mc alias set ceph-rgw https://rgw.internal.example.com \
  <access_key> <secret_key>

# バケットを作成
for ENV in prod staging dev; do
  mc mb "ceph-rgw/k1s0-terraform-state-${ENV}"
done
```

### バージョニングを有効化する

```bash
for ENV in prod staging dev; do
  mc version enable "ceph-rgw/k1s0-terraform-state-${ENV}"
done
```

### アクセスログを有効化する

```bash
# ログ収集用バケットを作成（全環境共有）
mc mb ceph-rgw/k1s0-terraform-state-logs

# 各バケットのアクセスログを設定
for ENV in prod staging dev; do
  mc admin trace ceph-rgw 2>&1 | tee "/var/log/k1s0-terraform-${ENV}.log" &
done
```

---

## Step 2: RGW ユーザーのアクセス権限を設定する

Terraform を実行するユーザー・サービスアカウントに以下の権限を付与する。

```bash
# Terraform 実行ユーザーにバケットアクセスを付与
radosgw-admin caps add \
  --uid="terraform-state" \
  --caps="buckets=read,write"
```

またはバケットポリシーで制御する場合:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "TerraformStateAccess",
      "Effect": "Allow",
      "Principal": {"AWS": ["arn:aws:iam:::user/terraform-state"]},
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
    }
  ]
}
```

---

## Step 3: backend.tf を更新する

各環境の `backend.tf` を以下のように更新する。
認証情報（`access_key`, `secret_key`）は環境変数 `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` で渡すこと（ファイルにハードコードしない）。

```hcl
# prod 環境の backend.tf 例
terraform {
  backend "s3" {
    bucket         = "k1s0-terraform-state-prod"
    key            = "k1s0/prod/terraform.tfstate"
    region         = "dummy"        # Ceph RGW では使用しないが required フィールドのためダミー値を設定
    endpoint       = "https://rgw.internal.example.com"  # Ceph RGW エンドポイント
    use_path_style = true           # Ceph RGW はパススタイル URL を使用
    use_lockfile   = true           # S3 上にロックファイルを作成（Terraform 1.10+）
    encrypt        = true
  }
}
```

---

## Step 4: 環境変数を設定する

```bash
# Ceph RGW の認証情報を環境変数で設定（Step 1 で取得した値を使用）
export AWS_ACCESS_KEY_ID="<access_key>"
export AWS_SECRET_ACCESS_KEY="<secret_key>"
```

---

## Step 5: terraform init -migrate-state の実行

各環境のディレクトリで以下を実行する。`-migrate-state` フラグにより Consul から Ceph RGW へ State が自動コピーされる。

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

## Step 6: 移行後の検証

```bash
# Ceph RGW に State ファイルが存在することを確認
mc ls ceph-rgw/k1s0-terraform-state-prod/k1s0/prod/
mc ls ceph-rgw/k1s0-terraform-state-staging/k1s0/staging/
mc ls ceph-rgw/k1s0-terraform-state-dev/k1s0/dev/

# terraform plan でドリフトがないことを確認（変更なしが期待値）
cd infra/terraform/environments/prod && terraform plan
cd infra/terraform/environments/staging && terraform plan
cd infra/terraform/environments/dev && terraform plan
```

---

## ロールバック手順

移行後に問題が発生した場合、以下の手順で Consul backend に戻す。

> **注意**: ロールバック前に必ず Ceph RGW 上の State をバックアップしておくこと。

### 1. backend.tf のコメントアウトを解除

各環境の `backend.tf` で、Consul 設定のコメントアウトを解除し、S3 設定をコメントアウトする。

### 2. terraform init -migrate-state を再実行

```bash
# Ceph RGW → Consul に State を戻す
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
- [Terraform S3 Backend use_lockfile](https://developer.hashicorp.com/terraform/language/backend/s3#use_lockfile)
- [Ceph RGW S3 互換 API ドキュメント](https://docs.ceph.com/en/latest/radosgw/s3/)
- [ADR-0025: Terraform State Backend を Ceph RGW に移行](../architecture/adr/0025-terraform-state-s3.md)
- 外部監査報告書 H-08: Terraform State Backend の SPOF リスク
