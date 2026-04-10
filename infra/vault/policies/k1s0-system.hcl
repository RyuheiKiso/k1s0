# k1s0 システム共通シークレット用 Vault ポリシー
# Tier: system
# 説明: system tier の共通シークレットのみに読み取りアクセスを提供する
# 注意: 各サービス固有のシークレットは個別ポリシー（auth-server.hcl 等）で管理する

# 共通シークレットへの読み取りアクセス（共有設定・証明書等）
path "secret/data/k1s0/system/common/*" {
  capabilities = ["read"]
}

# 共通シークレットのメタデータ（一覧取得・存在確認用）
path "secret/metadata/k1s0/system/common/*" {
  capabilities = ["read", "list"]
}

# MEDIUM-INFRA-001 監査対応: システムリース更新権限を追加する
# 長時間稼働するサービスでリース期限切れによる接続断を防止する
path "sys/leases/renew" {
  capabilities = ["update"]
}

# 自身のトークン情報確認と更新のための権限
path "auth/token/lookup-self" {
  capabilities = ["read"]
}

path "auth/token/renew-self" {
  capabilities = ["update"]
}
