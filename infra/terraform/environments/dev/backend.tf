# Terraform State を Ceph RGW（S3互換）で管理する
# S3互換: Ceph RGW の S3 API を使用して State を保存・バージョニング
# ロック: use_lockfile = true により S3 上にロックファイルを作成して並行実行を防止（Terraform 1.10+）
# 旧 Consul backend からの移行: docs/infrastructure/terraform-state-migration.md 参照
terraform {
  backend "s3" {
    bucket         = "k1s0-terraform-state-dev"
    key            = "k1s0/dev/terraform.tfstate"
    region         = "dummy"        # Ceph RGW では使用しないが required フィールドのためダミー値を設定
    endpoint       = "https://rgw.internal.example.com"  # Ceph RGW エンドポイント
    use_path_style = true           # Ceph RGW はパススタイル URL を使用
    use_lockfile   = true           # S3 上にロックファイルを作成（DynamoDB 不要）
    encrypt        = true
    # encrypt = true: Ceph RGW の SSE (Server-Side Encryption) を有効化する
    # Ceph OSD レベルの at-rest encryption はインフラチームが管理・確認する
    # S3 バージョニングはバケット設定で有効化すること（backend.tf では設定不可）
    # 認証情報は環境変数 AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY で設定すること
  }
}

# ── 旧 Consul backend 設定（ロールバック参考用・使用禁止）──────────────────
# 外部監査 H-08 対応により Ceph RGW backend へ移行済み
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
