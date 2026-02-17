package usecase

import "fmt"

// CheckPermissionInput はパーミッション確認の入力。
type CheckPermissionInput struct {
	Roles      []string `json:"roles"`
	Permission string   `json:"permission" validate:"required"`
	Resource   string   `json:"resource" validate:"required"`
}

// CheckPermissionOutput はパーミッション確認の出力。
type CheckPermissionOutput struct {
	Allowed bool   `json:"allowed"`
	Reason  string `json:"reason,omitempty"`
}

// CheckPermissionUseCase はパーミッション確認ユースケース。
type CheckPermissionUseCase struct{}

// NewCheckPermissionUseCase は新しい CheckPermissionUseCase を作成する。
func NewCheckPermissionUseCase() *CheckPermissionUseCase {
	return &CheckPermissionUseCase{}
}

// Execute はロールベースのアクセス制御ロジックを実行する。
// sys_admin: 全権限
// sys_operator: read, write
// sys_auditor: read のみ
func (uc *CheckPermissionUseCase) Execute(input CheckPermissionInput) *CheckPermissionOutput {
	for _, role := range input.Roles {
		switch role {
		case "sys_admin":
			return &CheckPermissionOutput{Allowed: true}
		case "sys_operator":
			if input.Permission == "read" || input.Permission == "write" {
				return &CheckPermissionOutput{Allowed: true}
			}
		case "sys_auditor":
			if input.Permission == "read" {
				return &CheckPermissionOutput{Allowed: true}
			}
		}
	}
	return &CheckPermissionOutput{
		Allowed: false,
		Reason:  fmt.Sprintf("insufficient permissions: role(s) %v do not grant '%s' on '%s'", input.Roles, input.Permission, input.Resource),
	}
}
