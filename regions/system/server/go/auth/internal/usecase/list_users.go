package usecase

import (
	"context"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
)

// ListUsersUseCase はユーザー一覧取得ユースケース。
type ListUsersUseCase struct {
	userRepo repository.UserRepository
}

// NewListUsersUseCase は新しい ListUsersUseCase を作成する。
func NewListUsersUseCase(userRepo repository.UserRepository) *ListUsersUseCase {
	return &ListUsersUseCase{
		userRepo: userRepo,
	}
}

// ListUsersInput はユーザー一覧取得の入力パラメータ。
type ListUsersInput struct {
	Page     int
	PageSize int
	Search   string
	Enabled  *bool
}

// ListUsersOutput はユーザー一覧取得の出力。
type ListUsersOutput struct {
	Users      []*model.User
	TotalCount int
	Page       int
	PageSize   int
	HasNext    bool
}

// Execute はユーザー一覧をページネーション付きで取得する。
func (uc *ListUsersUseCase) Execute(ctx context.Context, input ListUsersInput) (*ListUsersOutput, error) {
	// デフォルト値の設定
	if input.Page <= 0 {
		input.Page = 1
	}
	if input.PageSize <= 0 {
		input.PageSize = 20
	}
	if input.PageSize > 100 {
		input.PageSize = 100
	}

	params := repository.UserListParams{
		Page:     input.Page,
		PageSize: input.PageSize,
		Search:   input.Search,
		Enabled:  input.Enabled,
	}

	users, totalCount, err := uc.userRepo.ListUsers(ctx, params)
	if err != nil {
		return nil, err
	}

	hasNext := input.Page*input.PageSize < totalCount

	return &ListUsersOutput{
		Users:      users,
		TotalCount: totalCount,
		Page:       input.Page,
		PageSize:   input.PageSize,
		HasNext:    hasNext,
	}, nil
}
