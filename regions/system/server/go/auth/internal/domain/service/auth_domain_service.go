package service

// AuthDomainService はパーミッション解決ロジックを提供するドメインサービス。
type AuthDomainService struct {
	// rolePermissions はロールごとのパーミッションマッピング。
	rolePermissions map[string]map[string][]string
}

// NewAuthDomainService は新しい AuthDomainService を作成する。
func NewAuthDomainService() *AuthDomainService {
	return &AuthDomainService{
		rolePermissions: defaultRolePermissions(),
	}
}

// CheckPermission はロール一覧に基づいてパーミッションを確認する。
// allowed が true の場合は許可、false の場合は reason に拒否理由が入る。
func (s *AuthDomainService) CheckPermission(
	permission string, resource string, roles []string,
) (allowed bool, reason string) {
	for _, role := range roles {
		perms, ok := s.rolePermissions[role]
		if !ok {
			continue
		}
		resources, ok := perms[permission]
		if !ok {
			continue
		}
		for _, r := range resources {
			if r == "*" || r == resource {
				return true, ""
			}
		}
	}
	return false, "insufficient permissions: none of the assigned roles grant " + permission + " access to " + resource
}

// defaultRolePermissions はデフォルトのロール・パーミッションマッピングを返す。
func defaultRolePermissions() map[string]map[string][]string {
	return map[string]map[string][]string{
		"sys_admin": {
			"read":   {"*"},
			"write":  {"*"},
			"delete": {"*"},
			"admin":  {"*"},
		},
		"sys_operator": {
			"read":  {"users", "auth_config", "audit_logs"},
			"write": {"auth_config", "audit_logs"},
		},
		"sys_auditor": {
			"read": {"users", "audit_logs"},
		},
	}
}
