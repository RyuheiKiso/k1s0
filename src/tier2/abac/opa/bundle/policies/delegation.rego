# tier2 ABAC 委任ポリシー
# テナント内でのリソース委任権限を制御する

# パッケージ宣言: tier2 abac 委任ポリシー
package tier2.abac.delegation

# デフォルトは拒否 (allow ルールが明示的に true を返す場合のみ許可)
default allow = false

# allow ルール: tenant_id が一致し、かつ operation が許可リストにある場合に true を返す
allow {
    # 入力の tenant_id が auth context の tenant_id と一致することを確認する
    input.tenant_id == input.auth_context.tenant_id
    # operation が許可されている操作リストに含まれることを確認する
    input.operation in ["read", "write", "create", "delete"]
    # resource_scope が tenant_scoped または platform_global であることを確認する
    input.resource_scope in ["tenant_scoped", "tenant_master", "platform_global"]
}

# cross-tenant アクセスは常に拒否する
deny {
    # 入力の tenant_id と auth context の tenant_id が異なる場合は拒否
    input.tenant_id != input.auth_context.tenant_id
}
