package auth

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

// HasPermission は Claims に指定の権限があるかを判定する。
// realm_access と resource_access の両方をチェックする。
// admin ロールを持つ場合は全権限を付与する。
func HasPermission(claims *Claims, resource, action string) bool {
	// sys_admin は全権限
	if HasRole(claims, "sys_admin") {
		return true
	}

	// realm_access に admin ロールがある場合
	if HasRole(claims, "admin") {
		return true
	}

	// resource_access のチェック（指定リソースのロールを確認）
	if access, ok := claims.ResourceAccess[resource]; ok {
		for _, role := range access.Roles {
			if role == action || role == "admin" {
				return true
			}
		}
	}

	return false
}

// HasTierAccess は Claims で指定 Tier へのアクセスが許可されているかを判定する。
func HasTierAccess(claims *Claims, tier string) bool {
	for _, t := range claims.TierAccess {
		if strings.EqualFold(t, tier) {
			return true
		}
	}
	return false
}

