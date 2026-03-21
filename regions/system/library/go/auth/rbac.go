package authlib

import "strings"

// HasRole は Claims に指定のレルムロールが含まれるかを判定する。
func HasRole(claims *Claims, role string) bool {
	for _, r := range claims.RealmAccess.Roles {
		if r == role {
			return true
		}
	}
	return false
}

// HasResourceRole は Claims に指定のリソースロールが含まれるかを判定する。
func HasResourceRole(claims *Claims, resource, role string) bool {
	access, ok := claims.ResourceAccess[resource]
	if !ok {
		return false
	}
	for _, r := range access.Roles {
		if r == role {
			return true
		}
	}
	return false
}

// CheckPermission は Claims に指定の権限があるかを判定する。
// realm_access と resource_access の両方をチェックする。
// sys_admin のみ全リソース全アクションの権限を持つ（最小権限原則）。
// admin は resource_access で明示的に付与されたリソースのみ全アクションを許可する通常ロールとして扱う。
func CheckPermission(claims *Claims, resource, action string) bool {
	// sys_admin のみ全権限を付与する（スーパーユーザー）。
	// admin ロールは realm_access に存在しても全権限を付与しない（最小権限原則）。
	if HasRole(claims, "sys_admin") {
		return true
	}

	// resource_access のチェック（指定リソースのロールを確認）。
	// resource_access に admin ロールがある場合はそのリソース内の全アクションを許可する。
	// realm_access の admin は通常ロールとして扱い、ここでは判定しない。
	if access, ok := claims.ResourceAccess[resource]; ok {
		for _, role := range access.Roles {
			if role == action || role == "admin" {
				return true
			}
		}
	}

	return false
}

// HasPermission is kept for backward compatibility.
func HasPermission(claims *Claims, resource, action string) bool {
	return CheckPermission(claims, resource, action)
}

// tierLevel は Tier の階層レベルを返す。
// system(0) > business(1) > service(2) の順で上位 Tier ほど小さい値を返す。
// 不明な Tier は -1 を返す。
func tierLevel(tier string) int {
	switch strings.ToLower(tier) {
	case "system":
		return 0
	case "business":
		return 1
	case "service":
		return 2
	default:
		return -1
	}
}

// HasTierAccess は Claims で指定 Tier へのアクセスが許可されているかを判定する。
//
// Tier 階層ルール:
//   - system tier を持つユーザーは全 Tier (system, business, service) にアクセス可能
//   - business tier を持つユーザーは business と service にアクセス可能
//   - service tier を持つユーザーは service のみにアクセス可能
func HasTierAccess(claims *Claims, requiredTier string) bool {
	requiredLevel := tierLevel(requiredTier)
	if requiredLevel == -1 {
		return false
	}

	for _, userTier := range claims.TierAccess {
		userLevel := tierLevel(userTier)
		if userLevel != -1 && userLevel <= requiredLevel {
			return true
		}
	}
	return false
}
