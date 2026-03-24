# Terraform State を S3+DynamoDB で管理する
# S3: バージョニング・暗号化・アクセスログを有効化し State を安全に保存
# DynamoDB: State ロックにより並行実行による競合を防止
# 旧 Consul backend からの移行: docs/infrastructure/terraform-state-migration.md 参照
terraform {
  backend "s3" {
    bucket         = "k1s0-terraform-state-prod"
    key            = "k1s0/prod/terraform.tfstate"
    region         = "ap-northeast-1"
    dynamodb_table = "k1s0-terraform-state-lock"
    encrypt        = true

    # [H-11] KMS CMK 未設定（TODO: 本番環境では特に優先対応すること）
    # 現状: SSE-S3（AES-256）による AWS 管理キーで暗号化されている
    # 問題: PCI DSS / SOC2 等のコンプライアンス要件では CMK（カスタマー管理キー）が必須の場合がある
    # 本番環境での対応手順:
    #   1. AWS KMS で Terraform State 用の CMK を作成する（自動ローテーション有効化推奨）
    #      aws kms create-key --description "k1s0 Terraform State Key (prod)" \
    #        --key-usage ENCRYPT_DECRYPT --key-spec SYMMETRIC_DEFAULT \
    #        --enable-key-rotation --region ap-northeast-1
    #   2. キーエイリアスを設定する
    #      aws kms create-alias --alias-name alias/k1s0-terraform-state-prod --target-key-id <key-id>
    #   3. 以下のコメントを外してキー ARN を設定する
    # kms_key_id = "arn:aws:kms:ap-northeast-1:<account-id>:alias/k1s0-terraform-state-prod"
    # 詳細: docs/architecture/adr/0025-terraform-state-s3.md「未対応事項」参照

    # [L-4] S3 バージョニング要件
    # backend.tf での宣言は不可（バージョニングは S3 バケット側の設定）。
    # バケット k1s0-terraform-state-prod では versioning: Enabled を確認・設定すること:
    #   aws s3api put-bucket-versioning \
    #     --bucket k1s0-terraform-state-prod \
    #     --versioning-configuration Status=Enabled
    # バージョニングにより誤った apply からの State ロールバックが可能になる。
    # ACL は bucket-owner-full-control を推奨
  }
}

# ── 旧 Consul backend 設定（ロールバック参考用・使用禁止）──────────────────
# 外部監査 H-08 対応により S3+DynamoDB backend へ移行済み
# Consul は SPOF リスクおよび at-rest encryption 非対応のため廃止
#
# terraform {
#   backend "consul" {
#     address = "consul.internal.example.com:8500"
#     scheme  = "https"
#     path    = "terraform/k1s0/prod"
#     lock    = true
#     # ACLトークン: 環境変数 CONSUL_HTTP_TOKEN から自動取得される
#   }
# }
