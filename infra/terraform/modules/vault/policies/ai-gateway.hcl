# CRIT-006 監査対応: ai-gateway サービス専用 Vault ポリシー
# AI モデルルーティング・使用量追跡を担う system ティアのゲートウェイサービス。
# AI プロバイダー API キー・DB クレデンシャルのみアクセス可能とする。

# ai-gateway サービス固有の KV v2 シークレット（AI プロバイダー API キー等）
path "secret/data/k1s0/system/ai-gateway/*" {
  capabilities = ["read"]
}

# ai-gateway シークレットのメタデータ参照
path "secret/metadata/k1s0/system/ai-gateway/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（ai-gateway DB のみ）
path "database/creds/system-ai-gateway" {
  capabilities = ["read"]
}
