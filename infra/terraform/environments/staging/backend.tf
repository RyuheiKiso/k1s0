# Consul バックエンド設定 — staging 環境の Terraform State を保存する
# 本番環境では ACL ブートストラップが必要: consul acl bootstrap
# CONSUL_HTTP_TOKEN 環境変数に ACL トークンを設定すること
terraform {
  backend "consul" {
    address = "consul.internal.example.com:8500"
    scheme  = "https"
    path    = "terraform/k1s0/staging"
    lock    = true
    # ACLトークン: 環境変数 CONSUL_HTTP_TOKEN から自動取得される
  }
}
