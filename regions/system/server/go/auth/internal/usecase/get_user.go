package usecase

import (
	"context"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
)

// GetUserUseCase はユーザー情報取得ユースケース。
type GetUserUseCase struct {
	userRepo repository.UserRepository
}

// NewGetUserUseCase は新しい GetUserUseCase を作成する。
func NewGetUserUseCase(userRepo repository.UserRepository) *GetUserUseCase {
	return &GetUserUseCase{
		userRepo: userRepo,
	}
}

// Execute はユーザー ID からユーザー情報を取得する。
func (uc *GetUserUseCase) Execute(ctx context.Context, userID string) (*model.User, error) {
	if userID == "" {
		return nil, ErrUserNotFound
	}

	user, err := uc.userRepo.GetUser(ctx, userID)
	if err != nil {
		return nil, err
	}

	return user, nil
}

// GetUserRoles はユーザーに割り当てられたロール一覧を取得する。
func (uc *GetUserUseCase) GetUserRoles(ctx context.Context, userID string) ([]*model.Role, map[string][]*model.Role, error) {
	if userID == "" {
		return nil, nil, ErrUserNotFound
	}
	return uc.userRepo.GetUserRoles(ctx, userID)
}
