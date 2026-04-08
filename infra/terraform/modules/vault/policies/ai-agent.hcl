# CRIT-006 監査対応: ai-agent サービス専用 Vault ポリシー
# AI エージェント実行・タスク管理を担う system ティアのサービス。
# エージェント設定・DB クレデンシャルのみアクセス可能とする。

# ai-agent サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/ai-agent/*" {
  capabilities = ["read"]
}

# ai-agent シークレットのメタデータ参照
path "secret/metadata/k1s0/system/ai-agent/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（ai-agent DB のみ）
path "database/creds/system-ai-agent" {
  capabilities = ["read"]
}
