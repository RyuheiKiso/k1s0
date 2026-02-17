package auth

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/lestrrat-go/jwx/v2/jwk"
	"github.com/lestrrat-go/jwx/v2/jwt"
)

// JWKSFetcher は JWKS エンドポイントからの鍵取得を抽象化するインターフェース。
// テスト時にモックに差し替え可能。
type JWKSFetcher interface {
	FetchKeys(ctx context.Context, jwksURL string) (jwk.Set, error)
}

// DefaultJWKSFetcher は HTTP 経由で JWKS を取得するデフォルト実装。
type DefaultJWKSFetcher struct{}

// FetchKeys は指定 URL から JWKS を HTTP GET で取得する。
func (f *DefaultJWKSFetcher) FetchKeys(ctx context.Context, jwksURL string) (jwk.Set, error) {
	return jwk.Fetch(ctx, jwksURL)
}

// JWKSVerifier は JWKS エンドポイントから公開鍵を取得し、JWT トークンを検証する。
type JWKSVerifier struct {
	jwksURL   string
	issuer    string
	audience  string
	cacheTTL  time.Duration
	mu        sync.RWMutex
	keySet    jwk.Set
	lastFetch time.Time
	fetcher   JWKSFetcher
}

// NewJWKSVerifier は JWKS 検証器を生成する。
func NewJWKSVerifier(jwksURL, issuer, audience string, cacheTTL time.Duration) *JWKSVerifier {
	return &JWKSVerifier{
		jwksURL:  jwksURL,
		issuer:   issuer,
		audience: audience,
		cacheTTL: cacheTTL,
		fetcher:  &DefaultJWKSFetcher{},
	}
}

// NewJWKSVerifierWithFetcher はカスタム JWKSFetcher を使う検証器を生成する（テスト用）。
func NewJWKSVerifierWithFetcher(jwksURL, issuer, audience string, cacheTTL time.Duration, fetcher JWKSFetcher) *JWKSVerifier {
	return &JWKSVerifier{
		jwksURL:  jwksURL,
		issuer:   issuer,
		audience: audience,
		cacheTTL: cacheTTL,
		fetcher:  fetcher,
	}
}

// VerifyToken は JWT トークン文字列を検証し、Claims を返す。
func (v *JWKSVerifier) VerifyToken(ctx context.Context, tokenString string) (*Claims, error) {
	keySet, err := v.getKeySet(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get JWKS: %w", err)
	}

	token, err := jwt.Parse([]byte(tokenString),
		jwt.WithKeySet(keySet),
		jwt.WithIssuer(v.issuer),
		jwt.WithAudience(v.audience),
		jwt.WithValidate(true),
	)
	if err != nil {
		return nil, fmt.Errorf("token validation failed: %w", err)
	}

	return extractClaims(token)
}

// getKeySet はキャッシュから鍵セットを取得する。TTL を超えている場合は再取得する。
func (v *JWKSVerifier) getKeySet(ctx context.Context) (jwk.Set, error) {
	v.mu.RLock()
	if v.keySet != nil && time.Since(v.lastFetch) < v.cacheTTL {
		defer v.mu.RUnlock()
		return v.keySet, nil
	}
	v.mu.RUnlock()

	v.mu.Lock()
	defer v.mu.Unlock()

	// ダブルチェック: 他のゴルーチンが先に更新した可能性
	if v.keySet != nil && time.Since(v.lastFetch) < v.cacheTTL {
		return v.keySet, nil
	}

	keySet, err := v.fetcher.FetchKeys(ctx, v.jwksURL)
	if err != nil {
		// キャッシュにフォールバック
		if v.keySet != nil {
			return v.keySet, nil
		}
		return nil, fmt.Errorf("JWKS fetch failed: %w", err)
	}

	v.keySet = keySet
	v.lastFetch = time.Now()
	return keySet, nil
}

// InvalidateCache はキャッシュを無効化する。鍵ローテーション時に使用。
func (v *JWKSVerifier) InvalidateCache() {
	v.mu.Lock()
	defer v.mu.Unlock()
	v.lastFetch = time.Time{}
}
