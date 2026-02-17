package usecase

import (
	"context"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
)

// JWTConfig は JWT 検証に必要な設定。
type JWTConfig struct {
	Issuer   string `yaml:"issuer"`
	Audience string `yaml:"audience"`
}

// ValidateTokenUseCase は JWT トークン検証ユースケース。
type ValidateTokenUseCase struct {
	verifier  TokenVerifier
	jwtConfig JWTConfig
}

// NewValidateTokenUseCase は新しい ValidateTokenUseCase を作成する。
func NewValidateTokenUseCase(verifier TokenVerifier, jwtConfig JWTConfig) *ValidateTokenUseCase {
	return &ValidateTokenUseCase{
		verifier:  verifier,
		jwtConfig: jwtConfig,
	}
}

// Execute はトークンを検証し、Claims を返却する。
func (uc *ValidateTokenUseCase) Execute(ctx context.Context, tokenString string) (*model.TokenClaims, error) {
	if tokenString == "" {
		return nil, ErrInvalidToken
	}

	claims, err := uc.verifier.VerifyToken(ctx, tokenString)
	if err != nil {
		return nil, err
	}

	// issuer の追加検証
	if claims.Iss != uc.jwtConfig.Issuer {
		return nil, ErrInvalidIssuer
	}

	// audience の追加検証
	if claims.Aud != uc.jwtConfig.Audience {
		return nil, ErrInvalidAudience
	}

	return claims, nil
}
