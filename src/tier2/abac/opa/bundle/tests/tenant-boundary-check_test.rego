# tier2 ABAC テナント境界チェックポリシーのテスト
# tenant-boundary-check.rego（package tier2.abac.tenant_boundary）が
# 同一テナントアクセスを許可し、クロステナントアクセスを拒否することを確認する

# テストパッケージ宣言
package tier2.abac.tenant_boundary_test

# テスト対象パッケージのインポート
import data.tier2.abac.tenant_boundary

# テスト: 同一テナントへのアクセスは拒否されないことを確認する
# deny セットが空であれば拒否なし（アクセス許可）を意味する
test_same_tenant_allowed {
    # コンテキストとリソースの tenant_id が同一の場合は deny セットが空になること
    count(tenant_boundary.deny) == 0 with input as {
        "context": {"tenant_id": "tenant_a"},
        "resource": {"tenant_id": "tenant_a"}
    }
}

# テスト: 異なるテナントへのアクセスは拒否されることを確認する
# deny セットに 1 件以上のエントリがあれば拒否を意味する
test_cross_tenant_denied {
    # コンテキストとリソースの tenant_id が異なる場合は deny セットに理由が格納されること
    count(tenant_boundary.deny) > 0 with input as {
        "context": {"tenant_id": "tenant_a"},
        "resource": {"tenant_id": "tenant_b"}
    }
}

# テスト: context.tenant_id が未設定の場合は拒否されることを確認する
test_missing_context_tenant_id_denied {
    # コンテキストに tenant_id がない場合は deny セットに理由が格納されること
    count(tenant_boundary.deny) > 0 with input as {
        "context": {},
        "resource": {"tenant_id": "tenant_a"}
    }
}

# テスト: resource.tenant_id が未設定の場合は拒否されることを確認する
test_missing_resource_tenant_id_denied {
    # リソースに tenant_id がない場合は deny セットに理由が格納されること
    count(tenant_boundary.deny) > 0 with input as {
        "context": {"tenant_id": "tenant_a"},
        "resource": {}
    }
}
