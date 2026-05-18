# tier2 ABAC 委任ポリシーのテスト
# OPA の組み込みテストフレームワークを使用する

# テストパッケージ宣言
package tier2.abac.delegation_test

# テスト対象パッケージのインポート
import data.tier2.abac.delegation

# テスト: 正常な allow ケース
test_allow_same_tenant {
    # 同一テナントからの正常なリードリクエストは allow されることを確認する
    delegation.allow with input as {
        "tenant_id": "tenant-001",
        "auth_context": {"tenant_id": "tenant-001", "roles": ["user"]},
        "operation": "read",
        "resource_scope": "tenant_scoped"
    }
}

# テスト: cross-tenant deny ケース
test_deny_cross_tenant {
    # 異なるテナントへのアクセスは deny されることを確認する
    not delegation.allow with input as {
        "tenant_id": "tenant-002",
        "auth_context": {"tenant_id": "tenant-001", "roles": ["user"]},
        "operation": "read",
        "resource_scope": "tenant_scoped"
    }
}
