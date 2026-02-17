package usecase

import (
	"context"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
)

// TokenVerifier は JWT トークン検証のインターフェース。
// infra 層の JWKS 検証器がこのインターフェースを実装する。
type TokenVerifier interface {
	VerifyToken(ctx context.Context, tokenString string) (*model.TokenClaims, error)
}
