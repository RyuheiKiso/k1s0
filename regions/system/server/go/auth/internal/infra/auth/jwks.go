package auth

import (
	"context"
	"time"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
)

// JWKSVerifier は JWKS エンドポイントを使った JWT 検証器。
type JWKSVerifier struct {
	jwksURI  string
	cacheTTL time.Duration
}

// NewJWKSVerifier は新しい JWKSVerifier を作成する。
func NewJWKSVerifier(jwksURI string, cacheTTL time.Duration) *JWKSVerifier {
	return &JWKSVerifier{
		jwksURI:  jwksURI,
		cacheTTL: cacheTTL,
	}
}

// VerifyToken は JWT トークンを検証し、Claims を返却する。
// 本番実装では lestrrat-go/jwx/v2 を使って署名検証を行う。
func (v *JWKSVerifier) VerifyToken(ctx context.Context, tokenString string) (*model.TokenClaims, error) {
	// 本番実装:
	// 1. JWKS エンドポイントから公開鍵を取得（キャッシュ付き）
	// 2. JWT の署名検証
	// 3. 有効期限検証
	// 4. Claims パース
	return nil, nil
}
