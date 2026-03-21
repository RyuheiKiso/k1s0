package authlib

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

// testClaims はテスト用の Claims を構築するヘルパー。
func testClaims(roles []string, resourceAccess map[string]RoleSet, tierAccess []string) *Claims {
	return &Claims{
		Sub:            "user-123",
		RealmAccess:    RealmAccess{Roles: roles},
		ResourceAccess: resourceAccess,
		TierAccess:     tierAccess,
	}
}

// HasRole のテスト

// 存在するロールは true を返す
func TestHasRole_ExistingRole_ReturnsTrue(t *testing.T) {
	claims := testClaims([]string{"sys_operator", "viewer"}, nil, nil)
	assert.True(t, HasRole(claims, "sys_operator"))
}

// 存在しないロールは false を返す
func TestHasRole_MissingRole_ReturnsFalse(t *testing.T) {
	claims := testClaims([]string{"viewer"}, nil, nil)
	assert.False(t, HasRole(claims, "sys_admin"))
}

// ロールが空の場合は false を返す
func TestHasRole_EmptyRoles_ReturnsFalse(t *testing.T) {
	claims := testClaims(nil, nil, nil)
	assert.False(t, HasRole(claims, "any"))
}

// HasResourceRole のテスト

// 存在するリソースロールは true を返す
func TestHasResourceRole_ExistingRole_ReturnsTrue(t *testing.T) {
	claims := testClaims(nil, map[string]RoleSet{
		"my-service": {Roles: []string{"read", "write"}},
	}, nil)
	assert.True(t, HasResourceRole(claims, "my-service", "write"))
}

// 存在しないリソースは false を返す
func TestHasResourceRole_MissingResource_ReturnsFalse(t *testing.T) {
	claims := testClaims(nil, map[string]RoleSet{}, nil)
	assert.False(t, HasResourceRole(claims, "unknown-service", "read"))
}

// CheckPermission のテスト

// sys_admin ロールを持つユーザーは全権限を持つ
func TestCheckPermission_SysAdmin_GrantsAll(t *testing.T) {
	claims := testClaims([]string{"sys_admin"}, nil, nil)
	assert.True(t, CheckPermission(claims, "any-resource", "any-action"))
}

// admin ロールを持つユーザーは全権限を持つ
func TestCheckPermission_Admin_GrantsAll(t *testing.T) {
	claims := testClaims([]string{"admin"}, nil, nil)
	assert.True(t, CheckPermission(claims, "any-resource", "any-action"))
}

// resource_access に対応するアクションがある場合は true を返す
func TestCheckPermission_ResourceAction_ReturnsTrue(t *testing.T) {
	claims := testClaims(nil, map[string]RoleSet{
		"quotas": {Roles: []string{"read"}},
	}, nil)
	assert.True(t, CheckPermission(claims, "quotas", "read"))
}

// resource_access にアクションが無い場合は false を返す
func TestCheckPermission_NoMatchingAction_ReturnsFalse(t *testing.T) {
	claims := testClaims(nil, map[string]RoleSet{
		"quotas": {Roles: []string{"read"}},
	}, nil)
	assert.False(t, CheckPermission(claims, "quotas", "write"))
}

// HasPermission は CheckPermission の互換エイリアスとして動作する
func TestHasPermission_IsSameAsCheckPermission(t *testing.T) {
	claims := testClaims([]string{"sys_admin"}, nil, nil)
	assert.Equal(t, CheckPermission(claims, "res", "act"), HasPermission(claims, "res", "act"))
}

// HasTierAccess のテスト

// system tier は system, business, service すべてにアクセス可能
func TestHasTierAccess_SystemTier_GrantsAll(t *testing.T) {
	claims := testClaims(nil, nil, []string{"system"})
	assert.True(t, HasTierAccess(claims, "system"))
	assert.True(t, HasTierAccess(claims, "business"))
	assert.True(t, HasTierAccess(claims, "service"))
}

// business tier は business と service にアクセス可能
func TestHasTierAccess_BusinessTier_DeniesSystem(t *testing.T) {
	claims := testClaims(nil, nil, []string{"business"})
	assert.False(t, HasTierAccess(claims, "system"))
	assert.True(t, HasTierAccess(claims, "business"))
	assert.True(t, HasTierAccess(claims, "service"))
}

// service tier は service のみアクセス可能
func TestHasTierAccess_ServiceTier_ServiceOnly(t *testing.T) {
	claims := testClaims(nil, nil, []string{"service"})
	assert.False(t, HasTierAccess(claims, "system"))
	assert.False(t, HasTierAccess(claims, "business"))
	assert.True(t, HasTierAccess(claims, "service"))
}

// 不明な required tier は false を返す
func TestHasTierAccess_UnknownRequiredTier_ReturnsFalse(t *testing.T) {
	claims := testClaims(nil, nil, []string{"system"})
	assert.False(t, HasTierAccess(claims, "unknown"))
}

// tier_access が空の場合は false を返す
func TestHasTierAccess_EmptyTierAccess_ReturnsFalse(t *testing.T) {
	claims := testClaims(nil, nil, nil)
	assert.False(t, HasTierAccess(claims, "service"))
}
