package repository

import (
	"context"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
)

// UserRepository はユーザー情報取得のインターフェース。
// Keycloak Admin API をバックエンドとして想定する。
type UserRepository interface {
	// GetUser はユーザー ID からユーザー情報を取得する。
	GetUser(ctx context.Context, userID string) (*model.User, error)

	// ListUsers はユーザー一覧をページネーション付きで取得する。
	ListUsers(ctx context.Context, params UserListParams) ([]*model.User, int, error)

	// GetUserRoles はユーザーに割り当てられたロール一覧を取得する。
	GetUserRoles(ctx context.Context, userID string) ([]*model.Role, map[string][]*model.Role, error)

	// Healthy は接続確認を行う。
	Healthy(ctx context.Context) error
}

// UserListParams はユーザー一覧取得のパラメータ。
type UserListParams struct {
	Page     int
	PageSize int
	Search   string
	Enabled  *bool
}
