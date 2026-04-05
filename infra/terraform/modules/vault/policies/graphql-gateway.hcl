# graphql-gateway サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、graphql-gateway サービスが必要なパスのみに限定する。
# graphql-gateway は GraphQL クエリの集約・ルーティングを担う BFF サービス。
# 各バックエンドサービスへの接続設定のみ必要で、DB 直接アクセスは不要。

# graphql-gateway サービス固有の KV v2 シークレット（バックエンド URL・API キー等）
path "secret/data/k1s0/system/graphql-gateway/*" {
  capabilities = ["read"]
}

# graphql-gateway シークレットのメタデータ参照
path "secret/metadata/k1s0/system/graphql-gateway/*" {
  capabilities = ["read", "list"]
}

# PKI: クライアント証明書の発行（バックエンドサービスへの mTLS 接続用）
path "pki_int/issue/graphql-gateway" {
  capabilities = ["create", "update"]
}
