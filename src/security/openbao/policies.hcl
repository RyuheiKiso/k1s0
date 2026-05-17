# OpenBao HCL ポリシー定義ファイル
# dev / ops / security / formal の 4 ロールに対するアクセス制御を定義する
# このファイルは OpenBao サーバーに bao policy write コマンドで適用する

# dev ロールポリシー: 開発者向けアクセス権限
# アプリケーション開発に必要な最小限の権限を付与する
path "transit/encrypt/short_lived_key" {
  # 短期キーによる暗号化のみ許可する
  capabilities = ["create", "update"]
}

# dev ロール: 短期キーによる復号を許可する
path "transit/decrypt/short_lived_key" {
  # 短期キーによる復号のみ許可する
  capabilities = ["create", "update"]
}

# dev ロール: セッションキーによる暗号化を許可する
path "transit/encrypt/session_key" {
  # セッションキーによる暗号化を許可する
  capabilities = ["create", "update"]
}

# dev ロール: セッションキーによる復号を許可する
path "transit/decrypt/session_key" {
  # セッションキーによる復号を許可する
  capabilities = ["create", "update"]
}

# dev ロール: 一時暗号化キーによる暗号化を許可する
path "transit/encrypt/ephemeral_key" {
  # 一時キーによる暗号化を許可する
  capabilities = ["create", "update"]
}

# dev ロール: 一時暗号化キーによる復号を許可する
path "transit/decrypt/ephemeral_key" {
  # 一時キーによる復号を許可する
  capabilities = ["create", "update"]
}

# dev ロール: 自身のキーメタデータの参照を許可する
path "transit/keys/short_lived_key" {
  # キーメタデータの読み取りのみ許可する（削除・更新は禁止）
  capabilities = ["read"]
}

# ops ロールポリシー: 運用者向けアクセス権限
# 運用業務に必要なシークレット管理権限を付与する
path "transit/keys/*" {
  # 全 Transit キーのメタデータ参照を許可する
  capabilities = ["read", "list"]
}

# ops ロール: Transit エンジンの全暗号化操作を許可する
path "transit/encrypt/*" {
  # 全キーによる暗号化を許可する
  capabilities = ["create", "update"]
}

# ops ロール: Transit エンジンの全復号操作を許可する
path "transit/decrypt/*" {
  # 全キーによる復号を許可する（PII KEK を除く）
  capabilities = ["create", "update"]
}

# ops ロール: キーローテーションを許可する
path "transit/keys/+/rotate" {
  # キーのローテーションを許可する
  capabilities = ["create", "update"]
}

# ops ロール: 監査ログの参照を許可する
path "sys/audit" {
  # 監査設定の読み取りを許可する
  capabilities = ["read", "list"]
}

# security ロールポリシー: セキュリティ担当者向けアクセス権限
# セキュリティ審査・インシデント対応に必要な権限を付与する
path "transit/*" {
  # Transit エンジン全体への完全アクセスを許可する
  capabilities = ["create", "read", "update", "delete", "list"]
}

# security ロール: PII KEK の管理権限を付与する
path "transit/keys/pii_kek" {
  # PII キーへの完全アクセスを許可する（最高機密）
  capabilities = ["create", "read", "update", "delete"]
}

# security ロール: 監査ログの完全アクセスを許可する
path "sys/audit/*" {
  # 監査設定の管理を許可する
  capabilities = ["create", "read", "update", "delete", "list", "sudo"]
}

# security ロール: ポリシー管理権限を付与する
path "sys/policies/*" {
  # ポリシーの読み取りを許可する（変更は sudo が別途必要）
  capabilities = ["read", "list"]
}

# security ロール: シークレットエンジンの一覧参照を許可する
path "sys/mounts" {
  # マウント一覧の読み取りを許可する
  capabilities = ["read"]
}

# formal ロールポリシー: 形式検証者向けアクセス権限
# 形式検証・proof 生成に必要な最小限の読み取り権限を付与する
path "transit/keys/short_lived_key" {
  # formal 検証のためにキーメタデータの読み取りのみ許可する
  capabilities = ["read"]
}

# formal ロール: audit キーのメタデータ参照を許可する
path "transit/keys/audit_key" {
  # formal 検証のために監査キーメタデータの読み取りのみ許可する
  capabilities = ["read"]
}

# formal ロール: システム健全性情報の参照を許可する
path "sys/health" {
  # システム健全性の読み取りを許可する
  capabilities = ["read", "sudo"]
}
