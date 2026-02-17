package service

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestAuthDomainService_CheckPermission(t *testing.T) {
	svc := NewAuthDomainService()

	tests := []struct {
		name       string
		permission string
		resource   string
		roles      []string
		wantAllow  bool
		wantReason string
	}{
		{
			name:       "sys_admin は全リソースに対して全権限を持つ",
			permission: "admin",
			resource:   "users",
			roles:      []string{"sys_admin"},
			wantAllow:  true,
		},
		{
			name:       "sys_admin は任意のリソースに read 可能",
			permission: "read",
			resource:   "anything",
			roles:      []string{"sys_admin"},
			wantAllow:  true,
		},
		{
			name:       "sys_operator は users を read 可能",
			permission: "read",
			resource:   "users",
			roles:      []string{"sys_operator"},
			wantAllow:  true,
		},
		{
			name:       "sys_operator は audit_logs を write 可能",
			permission: "write",
			resource:   "audit_logs",
			roles:      []string{"sys_operator"},
			wantAllow:  true,
		},
		{
			name:       "sys_operator は users を write 不可",
			permission: "write",
			resource:   "users",
			roles:      []string{"sys_operator"},
			wantAllow:  false,
			wantReason: "insufficient permissions: none of the assigned roles grant write access to users",
		},
		{
			name:       "sys_auditor は users を read 可能",
			permission: "read",
			resource:   "users",
			roles:      []string{"sys_auditor"},
			wantAllow:  true,
		},
		{
			name:       "sys_auditor は audit_logs を read 可能",
			permission: "read",
			resource:   "audit_logs",
			roles:      []string{"sys_auditor"},
			wantAllow:  true,
		},
		{
			name:       "sys_auditor は audit_logs を write 不可",
			permission: "write",
			resource:   "audit_logs",
			roles:      []string{"sys_auditor"},
			wantAllow:  false,
			wantReason: "insufficient permissions: none of the assigned roles grant write access to audit_logs",
		},
		{
			name:       "未知のロールは全て拒否",
			permission: "read",
			resource:   "users",
			roles:      []string{"unknown_role"},
			wantAllow:  false,
			wantReason: "insufficient permissions: none of the assigned roles grant read access to users",
		},
		{
			name:       "空のロール一覧は拒否",
			permission: "read",
			resource:   "users",
			roles:      []string{},
			wantAllow:  false,
			wantReason: "insufficient permissions: none of the assigned roles grant read access to users",
		},
		{
			name:       "複数ロールのうち1つが許可すれば通る",
			permission: "read",
			resource:   "users",
			roles:      []string{"unknown_role", "sys_auditor"},
			wantAllow:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			allowed, reason := svc.CheckPermission(tt.permission, tt.resource, tt.roles)
			assert.Equal(t, tt.wantAllow, allowed)
			if !tt.wantAllow {
				assert.Equal(t, tt.wantReason, reason)
			}
		})
	}
}
