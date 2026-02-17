package usecase

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestCheckPermission_SysAdmin_AllAllowed(t *testing.T) {
	uc := NewCheckPermissionUseCase()

	output := uc.Execute(CheckPermissionInput{
		Roles:      []string{"sys_admin"},
		Permission: "admin",
		Resource:   "users",
	})

	assert.True(t, output.Allowed)
	assert.Empty(t, output.Reason)
}

func TestCheckPermission_SysOperator_ReadUsers(t *testing.T) {
	uc := NewCheckPermissionUseCase()

	output := uc.Execute(CheckPermissionInput{
		Roles:      []string{"sys_operator"},
		Permission: "read",
		Resource:   "users",
	})

	assert.True(t, output.Allowed)
	assert.Empty(t, output.Reason)
}

func TestCheckPermission_SysOperator_WriteUsers_Denied(t *testing.T) {
	uc := NewCheckPermissionUseCase()

	output := uc.Execute(CheckPermissionInput{
		Roles:      []string{"sys_operator"},
		Permission: "admin",
		Resource:   "users",
	})

	assert.False(t, output.Allowed)
	assert.Contains(t, output.Reason, "insufficient permissions")
	assert.Contains(t, output.Reason, "sys_operator")
}

func TestCheckPermission_SysAuditor_ReadAuditLogs(t *testing.T) {
	uc := NewCheckPermissionUseCase()

	output := uc.Execute(CheckPermissionInput{
		Roles:      []string{"sys_auditor"},
		Permission: "read",
		Resource:   "audit_logs",
	})

	assert.True(t, output.Allowed)
	assert.Empty(t, output.Reason)
}

func TestCheckPermission_EmptyRoles_Denied(t *testing.T) {
	uc := NewCheckPermissionUseCase()

	output := uc.Execute(CheckPermissionInput{
		Roles:      []string{},
		Permission: "read",
		Resource:   "users",
	})

	assert.False(t, output.Allowed)
	assert.Contains(t, output.Reason, "insufficient permissions")
}

func TestCheckPermission_UnknownRole_Denied(t *testing.T) {
	uc := NewCheckPermissionUseCase()

	output := uc.Execute(CheckPermissionInput{
		Roles:      []string{"unknown_role"},
		Permission: "read",
		Resource:   "users",
	})

	assert.False(t, output.Allowed)
	assert.Contains(t, output.Reason, "insufficient permissions")
	assert.Contains(t, output.Reason, "unknown_role")
}
