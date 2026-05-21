# tier2 ABAC root 権限禁止ポリシーのテスト
# deny_root.rego（package tier2.abac.deny_root）が root ロールによる
# テナントデータアクセスを拒否することを確認する

# テストパッケージ宣言
package tier2.abac.deny_root_test

# テスト対象パッケージのインポート
import data.tier2.abac.deny_root

# テスト: root ロールが tenant_data にアクセスしようとした場合に拒否されることを確認する
test_deny_root_role_tenant_data {
    # root ロールで tenant_data にアクセスする場合は deny が true になること
    deny_root.deny with input as {
        "auth_context": {"roles": ["root"]},
        "resource_type": "tenant_data"
    }
}

# テスト: root ロールが pii_data にアクセスしようとした場合に拒否されることを確認する
test_deny_root_role_pii_data {
    # root ロールで pii_data にアクセスする場合は deny が true になること
    deny_root.deny with input as {
        "auth_context": {"roles": ["root"]},
        "resource_type": "pii_data"
    }
}

# テスト: root ロール以外のユーザーは拒否されないことを確認する
test_non_root_role_allowed {
    # user ロールで tenant_data にアクセスする場合は deny が false になること
    not deny_root.deny with input as {
        "auth_context": {"roles": ["user"]},
        "resource_type": "tenant_data"
    }
}

# テスト: root ロールでも tenant_data / pii_data 以外のリソースは拒否されないことを確認する
test_root_role_non_protected_resource_allowed {
    # root ロールで tenant_data / pii_data 以外のリソースにアクセスする場合は deny が false になること
    not deny_root.deny with input as {
        "auth_context": {"roles": ["root"]},
        "resource_type": "public_config"
    }
}
