# CRIT-006 監査対応: bff-proxy サービス専用 Vault ポリシー
# OAuth2/OIDC ブラウザ認証フローを処理する BFF プロキシサービス。
# セッション暗号化キー・OIDC クライアントシークレットのみアクセス可能とする。

# bff-proxy サービス固有の KV v2 シークレット（OIDC クライアント ID/Secret 等）
path "secret/data/k1s0/system/bff-proxy/*" {
  capabilities = ["read"]
}

# bff-proxy シークレットのメタデータ参照
path "secret/metadata/k1s0/system/bff-proxy/*" {
  capabilities = ["read", "list"]
}
