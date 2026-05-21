# tier2 ABAC テナント境界チェックポリシー
# spec 10 §整合 6「cross-tenant integration test で tenant 越境取得ゼロを property test green」に準拠する
# 10_テナント分離適合仕様.md §tenant boundary enforcement の物理 policy として機能する

# パッケージ宣言: tier2.abac.tenant_boundary
package tier2.abac.tenant_boundary

# デフォルトは拒否 (deny ルールが true を返す場合のみ拒否し、呼び出し元が空でないことを確認する)
default deny = false

# deny ルール: input.context.tenant_id が input.resource.tenant_id と一致しない場合にアクセスを拒否する
# spec 10 §整合 6 property: cross-tenant アクセスは全経路で拒否されなければならない
deny[msg] {
    # input.context.tenant_id が存在することを確認する（未設定の場合は string 型エラーを防ぐ）
    input.context.tenant_id
    # input.resource.tenant_id が存在することを確認する（未設定の場合は string 型エラーを防ぐ）
    input.resource.tenant_id
    # コンテキストの tenant_id とリソースの tenant_id が一致しない場合に拒否する
    input.context.tenant_id != input.resource.tenant_id
    # 拒否理由を msg に格納する（audit_event の reason フィールドに書込む）
    msg := sprintf(
        "tenant boundary violation: context.tenant_id=%v does not match resource.tenant_id=%v",
        [input.context.tenant_id, input.resource.tenant_id],
    )
}

# deny ルール: input.context.tenant_id が未設定の場合にアクセスを拒否する
# 認証コンテキストなしのアクセスは全て tenant boundary 違反として扱う
deny[msg] {
    # context フィールド自体が存在しない場合は越境拒否する
    not input.context.tenant_id
    # 拒否理由を msg に格納する
    msg := "tenant boundary violation: context.tenant_id is not set"
}

# deny ルール: input.resource.tenant_id が未設定の場合にアクセスを拒否する
# テナントスコープのないリソースへの無制限アクセスを防ぐ
deny[msg] {
    # context tenant_id が存在することは確認する（上の deny ルールと重複しないようにする）
    input.context.tenant_id
    # resource フィールド自体が存在しない場合は越境拒否する
    not input.resource.tenant_id
    # 拒否理由を msg に格納する
    msg := "tenant boundary violation: resource.tenant_id is not set"
}
