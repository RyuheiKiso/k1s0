# Terraform State を S3+DynamoDB で管理する
# S3: バージョニング・暗号化・アクセスログを有効化し State を安全に保存
# DynamoDB: State ロックにより並行実行による競合を防止
# 旧 Consul backend からの移行: docs/infrastructure/terraform-state-migration.md 参照
terraform {
  backend "s3" {
    bucket         = "k1s0-terraform-state-dev"
    key            = "k1s0/dev/terraform.tfstate"
    region         = "ap-northeast-1"
    dynamodb_table = "k1s0-terraform-state-lock"
    encrypt        = true
    # encrypt = true: SSE (Server-Side Encryption) を有効化する
    # 本プロジェクトは Ceph S3 互換ストレージを使用しているため AWS KMS (kms_key_id) は適用不可
    # Ceph OSD レベルの at-rest encryption はインフラチームが管理・確認する
    # AWS S3 へ移行する場合は kms_key_id の追加が必須（セキュリティ要件）
    # S3 バージョニングはバケット設定で有効化すること（backend.tf では設定不可）
    # S3 互換ストレージ（MinIO 等）を使用する場合、kms_key_id は適用不可
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
#     path    = "terraform/k1s0/dev"
#     lock    = true
#     # ACLトークン: 環境変数 CONSUL_HTTP_TOKEN から自動取得される
#   }
# }
