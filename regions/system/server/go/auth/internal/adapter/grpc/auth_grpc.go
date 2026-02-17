package grpc

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/usecase"
)

// ValidateTokenExecutor は ValidateTokenUseCase の実行インターフェース。
type ValidateTokenExecutor interface {
	Execute(ctx context.Context, token string) (*model.TokenClaims, error)
}

// GetUserExecutor は GetUserUseCase + GetUserRoles の実行インターフェース。
type GetUserExecutor interface {
	Execute(ctx context.Context, userID string) (*model.User, error)
	GetUserRoles(ctx context.Context, userID string) ([]*model.Role, map[string][]*model.Role, error)
}

// ListUsersExecutor は ListUsersUseCase の実行インターフェース。
type ListUsersExecutor interface {
	Execute(ctx context.Context, input usecase.ListUsersInput) (*usecase.ListUsersOutput, error)
}

// AuthGRPCService は gRPC AuthService の実装。
type AuthGRPCService struct {
	validateTokenUC ValidateTokenExecutor
	getUserUC       GetUserExecutor
	listUsersUC     ListUsersExecutor
}

// NewAuthGRPCService は AuthGRPCService のコンストラクタ。
func NewAuthGRPCService(
	validateTokenUC ValidateTokenExecutor,
	getUserUC GetUserExecutor,
	listUsersUC ListUsersExecutor,
) *AuthGRPCService {
	return &AuthGRPCService{
		validateTokenUC: validateTokenUC,
		getUserUC:       getUserUC,
		listUsersUC:     listUsersUC,
	}
}

// ValidateToken は JWT トークンを検証する。
func (s *AuthGRPCService) ValidateToken(ctx context.Context, req *ValidateTokenRequest) (*ValidateTokenResponse, error) {
	claims, err := s.validateTokenUC.Execute(ctx, req.Token)
	if err != nil {
		return &ValidateTokenResponse{
			Valid:        false,
			ErrorMessage: err.Error(),
		}, nil
	}

	return &ValidateTokenResponse{
		Valid:  true,
		Claims: domainClaimsToPb(claims),
	}, nil
}

// GetUser はユーザー情報を取得する。
func (s *AuthGRPCService) GetUser(ctx context.Context, req *GetUserRequest) (*GetUserResponse, error) {
	user, err := s.getUserUC.Execute(ctx, req.UserId)
	if err != nil {
		if errors.Is(err, usecase.ErrUserNotFound) {
			return nil, fmt.Errorf("rpc error: code = NotFound desc = user not found: %s", req.UserId)
		}
		return nil, fmt.Errorf("rpc error: code = Internal desc = %v", err)
	}

	return &GetUserResponse{
		User: domainUserToPb(user),
	}, nil
}

// ListUsers はユーザー一覧を取得する。
func (s *AuthGRPCService) ListUsers(ctx context.Context, req *ListUsersRequest) (*ListUsersResponse, error) {
	input := usecase.ListUsersInput{
		Page:     1,
		PageSize: 20,
	}
	if req.Pagination != nil {
		input.Page = int(req.Pagination.Page)
		input.PageSize = int(req.Pagination.PageSize)
	}
	if req.Search != "" {
		input.Search = req.Search
	}
	if req.Enabled != nil {
		input.Enabled = req.Enabled
	}

	output, err := s.listUsersUC.Execute(ctx, input)
	if err != nil {
		return nil, fmt.Errorf("rpc error: code = Internal desc = %v", err)
	}

	pbUsers := make([]*PbUser, 0, len(output.Users))
	for _, u := range output.Users {
		pbUsers = append(pbUsers, domainUserToPb(u))
	}

	return &ListUsersResponse{
		Users: pbUsers,
		Pagination: &PaginationResult{
			TotalCount: int32(output.TotalCount),
			Page:       int32(output.Page),
			PageSize:   int32(output.PageSize),
			HasNext:    output.HasNext,
		},
	}, nil
}

// GetUserRoles はユーザーのロール一覧を取得する。
func (s *AuthGRPCService) GetUserRoles(ctx context.Context, req *GetUserRolesRequest) (*GetUserRolesResponse, error) {
	realmRoles, clientRoles, err := s.getUserUC.GetUserRoles(ctx, req.UserId)
	if err != nil {
		if errors.Is(err, usecase.ErrUserNotFound) {
			return nil, fmt.Errorf("rpc error: code = NotFound desc = user not found: %s", req.UserId)
		}
		return nil, fmt.Errorf("rpc error: code = Internal desc = %v", err)
	}

	pbRealmRoles := make([]*PbRole, 0, len(realmRoles))
	for _, r := range realmRoles {
		pbRealmRoles = append(pbRealmRoles, &PbRole{
			Id:          r.ID,
			Name:        r.Name,
			Description: r.Description,
		})
	}

	pbClientRoles := make(map[string]*PbRoleList)
	for clientID, roles := range clientRoles {
		pbRoles := make([]*PbRole, 0, len(roles))
		for _, r := range roles {
			pbRoles = append(pbRoles, &PbRole{
				Id:          r.ID,
				Name:        r.Name,
				Description: r.Description,
			})
		}
		pbClientRoles[clientID] = &PbRoleList{Roles: pbRoles}
	}

	return &GetUserRolesResponse{
		UserId:      req.UserId,
		RealmRoles:  pbRealmRoles,
		ClientRoles: pbClientRoles,
	}, nil
}

// CheckPermission はロールベースのパーミッション確認を行う。
// sys_admin は全リソースに対して全権限を持つ。
// sys_operator は read, write 権限を持つ。
// sys_auditor は read 権限のみ。
// それ以外は権限なし。
func (s *AuthGRPCService) CheckPermission(ctx context.Context, req *CheckPermissionRequest) (*CheckPermissionResponse, error) {
	allowed := checkRolePermission(req.Roles, req.Permission, req.Resource)
	if allowed {
		return &CheckPermissionResponse{Allowed: true}, nil
	}

	return &CheckPermissionResponse{
		Allowed: false,
		Reason:  fmt.Sprintf("insufficient permissions: role(s) %v do not grant '%s' on '%s'", req.Roles, req.Permission, req.Resource),
	}, nil
}

// checkRolePermission はロールベースのアクセス制御ロジック。
func checkRolePermission(roles []string, permission, resource string) bool {
	for _, role := range roles {
		switch role {
		case "sys_admin":
			return true
		case "sys_operator":
			if permission == "read" || permission == "write" {
				return true
			}
		case "sys_auditor":
			if permission == "read" {
				return true
			}
		}
	}
	return false
}

// --- 変換ヘルパー ---

func domainClaimsToPb(c *model.TokenClaims) *PbTokenClaims {
	pb := &PbTokenClaims{
		Sub:              c.Sub,
		Iss:              c.Iss,
		Aud:              c.Aud,
		Exp:              c.Exp,
		Iat:              c.Iat,
		Jti:              c.Jti,
		PreferredUsername: c.PreferredUsername,
		Email:            c.Email,
		TierAccess:       c.TierAccess,
	}

	pb.RealmAccess = &PbRealmAccess{
		Roles: c.RealmAccess.Roles,
	}

	if len(c.ResourceAccess) > 0 {
		pb.ResourceAccess = make(map[string]*PbClientRoles)
		for k, v := range c.ResourceAccess {
			pb.ResourceAccess[k] = &PbClientRoles{Roles: v.Roles}
		}
	}

	return pb
}

func domainUserToPb(u *model.User) *PbUser {
	pb := &PbUser{
		Id:            u.ID,
		Username:      u.Username,
		Email:         u.Email,
		FirstName:     u.FirstName,
		LastName:      u.LastName,
		Enabled:       u.Enabled,
		EmailVerified: u.EmailVerified,
		CreatedAt: &Timestamp{
			Seconds: u.CreatedAt.Unix(),
			Nanos:   int32(u.CreatedAt.Nanosecond()),
		},
	}

	if len(u.Attributes) > 0 {
		pb.Attributes = make(map[string]*PbStringList)
		for k, v := range u.Attributes {
			pb.Attributes[k] = &PbStringList{Values: v}
		}
	}

	return pb
}

func timeToTimestamp(t time.Time) *Timestamp {
	return &Timestamp{
		Seconds: t.Unix(),
		Nanos:   int32(t.Nanosecond()),
	}
}
