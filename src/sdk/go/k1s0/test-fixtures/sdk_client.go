// src/sdk/go/k1s0/test-fixtures/sdk_client.go
//
// k1s0 Go SDK test-fixtures: SDK client init helper。
// 利用者が fixture から認証済 SDK client を取得する経路（領域 2、ADR-TEST-010 §3）。
package testfixtures

import (
	// testing は t.Helper / t.Fatal で test framework と統合
	"testing"
)

// SDKClient は test 用に context (tenant) を inject した薄い client wrapper。
// 採用初期で src/sdk/go/k1s0/client.Client を内包する形に拡張する。
type SDKClient struct {
	// Tenant はこの client が代表する tenant ID
	Tenant string
	// fixture は Setup 経由の親 Fixture（MockBuilder アクセス用）
	fixture *Fixture
}

// NewSDKClient は fixture 経由で test 用 SDK client を生成する。
// 採用初期で k1s0.NewClient(...) + JWT 注入の wrapper として実装する。
func (f *Fixture) NewSDKClient(t *testing.T, tenant string) *SDKClient {
	t.Helper()
	// tenant 未指定時は Options の Tenant を使う
	if tenant == "" {
		tenant = f.Options.Tenant
	}
	return &SDKClient{
		Tenant:  tenant,
		fixture: f,
	}
}

// SetState は State.Set RPC を叩く（採用初期で k1s0.Client.State().Set への wrapper として実装）
func (c *SDKClient) SetState(t *testing.T, key string, value any) error {
	t.Helper()
	// 採用初期で SDK client.State().Set(ctx, key, value) を呼ぶ
	t.Skipf("SetState 未実装 (tenant=%s, key=%s) - 採用初期 (ADR-TEST-010)", c.Tenant, key)
	return nil
}

// GetState は State.Get RPC を叩く
func (c *SDKClient) GetState(t *testing.T, key string) (any, error) {
	t.Helper()
	t.Skipf("GetState 未実装 (tenant=%s, key=%s) - 採用初期", c.Tenant, key)
	return nil, nil
}
