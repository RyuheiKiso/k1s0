# auth サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、auth サービスが本当に必要なパスのみに限定する。
# - auth 固有シークレット（JWT 署名鍵、Keycloak クレデンシャル等）
# - Transit シークレットエンジン（トークン署名・検証）
# - DB 動的クレデンシャル（auth DB のみ）
# system 共通シークレット（Kafka SASL 等）へのアクセスは auth サービスには不要なため除外する。

# auth サービス固有の KV v2 シークレット（JWT 鍵、Keycloak 設定等）
path "secret/data/k1s0/system/auth/*" {
  capabilities = ["read"]
}

# auth シークレットのメタデータ参照（バージョン管理に使用）
path "secret/metadata/k1s0/system/auth/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（auth DB のみ）
path "database/creds/system-auth" {
  capabilities = ["read"]
}

# Transit シークレットエンジン: JWT トークン署名
# auth サービスは ID トークン・アクセストークンの署名を担うため create/update が必要
path "transit/sign/auth-jwt" {
  capabilities = ["create", "update"]
}

# Transit シークレットエンジン: JWT トークン署名検証
path "transit/verify/auth-jwt" {
  capabilities = ["create", "update"]
}

# PKI: クライアント証明書の発行（mTLS 対応）
path "pki_int/issue/auth" {
  capabilities = ["create", "update"]
}
