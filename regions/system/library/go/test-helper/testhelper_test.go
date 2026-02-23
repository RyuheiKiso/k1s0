package testhelper_test

import (
	"testing"

	testhelper "github.com/k1s0-platform/system-library-go-test-helper"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestJwtTestHelper_CreateAdminToken(t *testing.T) {
	h := testhelper.NewJwtTestHelper("test-secret")
	token := h.CreateAdminToken()
	claims, err := h.DecodeClaims(token)
	require.NoError(t, err)
	assert.Equal(t, "admin", claims.Sub)
	assert.Equal(t, []string{"admin"}, claims.Roles)
}

func TestJwtTestHelper_CreateUserToken(t *testing.T) {
	h := testhelper.NewJwtTestHelper("test-secret")
	token := h.CreateUserToken("user-123", []string{"user", "reader"})
	claims, err := h.DecodeClaims(token)
	require.NoError(t, err)
	assert.Equal(t, "user-123", claims.Sub)
	assert.Equal(t, []string{"user", "reader"}, claims.Roles)
}

func TestJwtTestHelper_CreateTokenWithTenant(t *testing.T) {
	h := testhelper.NewJwtTestHelper("secret")
	token := h.CreateToken(testhelper.TestClaims{
		Sub:      "svc",
		Roles:    []string{"service"},
		TenantID: "t-1",
		Iat:      1000,
		Exp:      2000,
	})
	claims, err := h.DecodeClaims(token)
	require.NoError(t, err)
	assert.Equal(t, "t-1", claims.TenantID)
}

func TestJwtTestHelper_DecodeInvalidToken(t *testing.T) {
	h := testhelper.NewJwtTestHelper("s")
	_, err := h.DecodeClaims("invalid")
	assert.Error(t, err)
}

func TestMockServerBuilder_Notification(t *testing.T) {
	server := testhelper.NewNotificationServerMock().
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

func TestMockServerBuilder_NotFound(t *testing.T) {
	server := testhelper.NewRatelimitServerMock().WithHealthOK().Build()
	_, _, ok := server.Handle("GET", "/nonexistent")
	assert.False(t, ok)
}

func TestFixtureBuilder_UUID(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	id := fb.UUID()
	assert.Len(t, id, 36)
	assert.Contains(t, id, "-")
}

func TestFixtureBuilder_Email(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	email := fb.Email()
	assert.Contains(t, email, "@example.com")
}

func TestFixtureBuilder_Name(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	name := fb.Name()
	assert.Contains(t, name, "user-")
}

func TestFixtureBuilder_Int(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	for i := 0; i < 100; i++ {
		val := fb.Int(10, 20)
		assert.GreaterOrEqual(t, val, 10)
		assert.Less(t, val, 20)
	}
}

func TestFixtureBuilder_IntSameMinMax(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	assert.Equal(t, 5, fb.Int(5, 5))
}

func TestFixtureBuilder_TenantID(t *testing.T) {
	fb := testhelper.FixtureBuilder{}
	tid := fb.TenantID()
	assert.Contains(t, tid, "tenant-")
}

func TestAssertionHelper_JSONContains(t *testing.T) {
	ah := testhelper.AssertionHelper{}
	err := ah.JSONContains(
		`{"id":"1","status":"ok","extra":"ignored"}`,
		`{"id":"1","status":"ok"}`,
	)
	assert.NoError(t, err)
}

func TestAssertionHelper_JSONContainsMismatch(t *testing.T) {
	ah := testhelper.AssertionHelper{}
	err := ah.JSONContains(`{"id":"1"}`, `{"id":"2"}`)
	assert.Error(t, err)
}

func TestAssertionHelper_JSONContainsNested(t *testing.T) {
	ah := testhelper.AssertionHelper{}
	err := ah.JSONContains(
		`{"user":{"id":"1","name":"test"},"status":"ok"}`,
		`{"user":{"id":"1"}}`,
	)
	assert.NoError(t, err)
}

func TestAssertionHelper_EventEmitted(t *testing.T) {
	ah := testhelper.AssertionHelper{}
	events := []map[string]interface{}{
		{"type": "created", "id": "1"},
		{"type": "updated", "id": "2"},
	}
	assert.NoError(t, ah.EventEmitted(events, "created"))
	assert.Error(t, ah.EventEmitted(events, "deleted"))
}
