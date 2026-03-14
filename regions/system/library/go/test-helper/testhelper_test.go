package testhelper_test

import (
	"testing"
	"time"

	testhelper "github.com/k1s0-platform/system-library-go-test-helper"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// JwtTestHelperのCreateAdminTokenが管理者ロールを持つJWTトークンを生成することを確認する。
func TestJwtTestHelper_CreateAdminToken(t *testing.T) {
	h := testhelper.NewJwtTestHelper("test-secret")
	token := h.CreateAdminToken()
	claims, err := h.DecodeClaims(token)
	require.NoError(t, err)
	assert.Equal(t, "admin", claims.Sub)
	assert.Equal(t, []string{"admin"}, claims.Roles)
}

// JwtTestHelperのCreateUserTokenが指定したユーザーIDとロールを持つJWTトークンを生成することを確認する。
func TestJwtTestHelper_CreateUserToken(t *testing.T) {
	h := testhelper.NewJwtTestHelper("test-secret")
	token := h.CreateUserToken("user-123", []string{"user", "reader"})
	claims, err := h.DecodeClaims(token)
	require.NoError(t, err)
	assert.Equal(t, "user-123", claims.Sub)
	assert.Equal(t, []string{"user", "reader"}, claims.Roles)
}

// JwtTestHelperのCreateTokenがテナントIDを含むJWTトークンを正しく生成することを確認する。
func TestJwtTestHelper_CreateTokenWithTenant(t *testing.T) {
	h := testhelper.NewJwtTestHelper("secret")
	token := h.CreateToken(testhelper.TestClaims{
		Sub:      "svc",
		Roles:    []string{"service"},
		TenantID: "t-1",
		Expiry:   30 * time.Minute,
	})
	claims, err := h.DecodeClaims(token)
	require.NoError(t, err)
	assert.Equal(t, "t-1", claims.TenantID)
	assert.Greater(t, claims.Exp, claims.Iat)
}

// 無効なJWTトークンに対してDecodeClaimsがエラーを返すことを確認する。
func TestJwtTestHelper_DecodeInvalidToken(t *testing.T) {
	h := testhelper.NewJwtTestHelper("s")
	_, err := h.DecodeClaims("invalid")
	assert.Error(t, err)
}

// MockServerBuilderが通知サーバーのモックを正しくビルドしてリクエストを処理することを確認する。
func TestMockServerBuilder_Notification(t *testing.T) {
	server := testhelper.NewMockServerBuilder().
		NotificationServer().
		WithHealthOK().
		WithSuccessResponse("/send", `{"id":"1","status":"sent"}`).
		Build()

	status, body, ok := server.Handle("GET", "/health")
	assert.True(t, ok)
	assert.Equal(t, 200, status)
	assert.Contains(t, body, "ok")

	status, _, ok = server.Handle("POST", "/send")
	assert.True(t, ok)
	assert.Equal(t, 200, status)

	assert.Equal(t, 2, server.RequestCount())
}

// ContainerBuilderが複数のコンテナ設定を正しく構築することを確認する。
func TestContainerBuilder(t *testing.T) {
	containers := testhelper.NewContainerBuilder().
		WithPostgres().
		WithRedis().
		WithKafka().
		WithKeycloak().
		Build()

	assert.True(t, containers.Postgres)
	assert.True(t, containers.Redis)
	assert.True(t, containers.Kafka)
	assert.True(t, containers.Keycloak)
}

// 登録されていないパスへのリクエストに対してモックサーバーがfalseを返すことを確認する。
func TestMockServerBuilder_NotFound(t *testing.T) {
	server := testhelper.NewRatelimitServerMock().WithHealthOK().Build()
	_, _, ok := server.Handle("GET", "/nonexistent")
	assert.False(t, ok)
}

// FixtureBuilderのUUIDが正しい形式のUUID文字列を生成することを確認する。
func TestFixtureBuilder_UUID(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	id := fb.UUID()
	assert.Len(t, id, 36)
	assert.Contains(t, id, "-")
}

// FixtureBuilderのEmailがexample.comドメインのメールアドレスを生成することを確認する。
func TestFixtureBuilder_Email(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	email := fb.Email()
	assert.Contains(t, email, "@example.com")
}

// FixtureBuilderのNameがuser-プレフィックスを持つ名前を生成することを確認する。
func TestFixtureBuilder_Name(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	name := fb.Name()
	assert.Contains(t, name, "user-")
}

// FixtureBuilderのIntが指定した範囲内の整数を生成することを確認する。
func TestFixtureBuilder_Int(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	for i := 0; i < 100; i++ {
		val := fb.Int(10, 20)
		assert.GreaterOrEqual(t, val, 10)
		assert.Less(t, val, 20)
	}
}

// FixtureBuilderのIntで最小値と最大値が同じ場合にその値を返すことを確認する。
func TestFixtureBuilder_IntSameMinMax(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	assert.Equal(t, 5, fb.Int(5, 5))
}

// FixtureBuilderのTenantIDがtenant-プレフィックスを持つIDを生成することを確認する。
func TestFixtureBuilder_TenantID(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	tid := fb.TenantID()
	assert.Contains(t, tid, "tenant-")
}

// AssertionHelperのJSONContainsが期待するJSONキーと値が含まれる場合にエラーを返さないことを確認する。
func TestAssertionHelper_JSONContains(t *testing.T) {
	ah := testhelper.AssertionHelper{}
	err := ah.JSONContains(
		`{"id":"1","status":"ok","extra":"ignored"}`,
		`{"id":"1","status":"ok"}`,
	)
	assert.NoError(t, err)
}

// AssertionHelperのJSONContainsが値が一致しない場合にエラーを返すことを確認する。
func TestAssertionHelper_JSONContainsMismatch(t *testing.T) {
	ah := testhelper.AssertionHelper{}
	err := ah.JSONContains(`{"id":"1"}`, `{"id":"2"}`)
	assert.Error(t, err)
}

// AssertionHelperのJSONContainsがネストされたJSONの部分一致を正しく検証することを確認する。
func TestAssertionHelper_JSONContainsNested(t *testing.T) {
	ah := testhelper.AssertionHelper{}
	err := ah.JSONContains(
		`{"user":{"id":"1","name":"test"},"status":"ok"}`,
		`{"user":{"id":"1"}}`,
	)
	assert.NoError(t, err)
}

// AssertionHelperのEventEmittedが指定したイベントタイプの存在有無を正しく検証することを確認する。
func TestAssertionHelper_EventEmitted(t *testing.T) {
	ah := testhelper.AssertionHelper{}
	events := []map[string]interface{}{
		{"type": "created", "id": "1"},
		{"type": "updated", "id": "2"},
	}
	assert.NoError(t, ah.EventEmitted(events, "created"))
	assert.Error(t, ah.EventEmitted(events, "deleted"))
}
