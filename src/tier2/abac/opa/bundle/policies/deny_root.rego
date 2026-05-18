# tier2 ABAC root 権限禁止ポリシー
# システム管理者権限 (root) によるテナントデータアクセスを禁止する

# パッケージ宣言
package tier2.abac.deny_root

# デフォルトは許可 (deny ルールが true を返す場合のみ拒否)
default deny = false

# deny ルール: root 権限ユーザーによる cross-tenant アクセスを拒否する
deny {
    # ユーザーが root ロールを持つ場合は拒否する
    input.auth_context.roles[_] == "root"
    # かつ tenant データにアクセスしようとしている場合
    input.resource_type in ["tenant_data", "pii_data"]
}
