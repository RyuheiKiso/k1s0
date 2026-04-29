// 本ファイルは tier2 Go サービス共通の HTTP JWT 認証 middleware。
//
// docs 正典:
//   docs/03_要件定義/00_共通規約.md §「認証認可」
//   docs/03_要件定義/30_非機能要件/E_セキュリティ.md NFR-E-AC-001 / 003 / 005
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
//
// 役割:
//   tier2 Go の HTTP server（notification-hub / stock-reconciler 等）が共通で挿す
//   middleware。`Authorization: Bearer <jwt>` ヘッダから JWT を取り出し、
//   T2_AUTH_MODE 環境変数の値に応じて 3 通り検証する:
//     - off  : dev 限定。署名検証 skip、subject="dev" / tenant_id="demo-tenant" を context に積む
//     - hmac : T2_AUTH_HMAC_SECRET の HS256/384/512 で署名 + 期限 + テナント claim を検証
//     - jwks : T2_AUTH_JWKS_URL から JWKS を fetch しキャッシュ、RS256/384/512 で検証
//
//   検証成功時は subject / tenant_id / 生 token を request context に attach し、
//   後段の handler / k1s0 SDK 呼出で取り出して TenantContext に詰めて tier1 へ送る。
//
//   tier3 BFF の internal/auth/middleware.go と同型のロジックだが、bffErrors 依存を
//   外し標準的な JSON エラーを返す自己完結版（OSS quality 一貫性のため tier2 / 3 で
//   同等の検証強度を提供）。

// Package auth は tier2 Go サービス共通の HTTP JWT 認証 middleware を提供する。
package auth

// 標準 / 外部 import。
import (
	// context 伝搬。
	"context"
	// JSON エンコード（エラーレスポンス用）。
	"encoding/json"
	// 標準 errors。
	"errors"
	// エラー文字列整形。
	"fmt"
	// HTTP server。
	"net/http"
	// 環境変数読込。
	"os"
	// 文字列処理。
	"strings"
	// 排他制御（JWKS cache）。
	"sync"
	// 期限処理。
	"time"

	// JOSE 実装（RS256/HS256 双方をサポートし、tier3 BFF と同一の jwx 系列）。
	"github.com/go-jose/go-jose/v4"
	"github.com/go-jose/go-jose/v4/jwt"
)

// contextKey は context 経由で識別を渡す際のキー（衝突回避のため独自型）。
type contextKey string

// SubjectKey は認証済 subject を context から取り出すキー。
const SubjectKey contextKey = "k1s0.subject"

// TenantIDKey は認証済 tenant_id を context から取り出すキー。
const TenantIDKey contextKey = "k1s0.tenant_id"

// TokenKey は受け取った生 JWT 文字列を context から取り出すキー（tier1 への転送用）。
const TokenKey contextKey = "k1s0.bearer_token"

// AuthMode は middleware の動作モード。
type AuthMode string

const (
	// AuthModeOff は JWT 検証スキップ（dev 限定）。
	AuthModeOff AuthMode = "off"
	// AuthModeHMAC は HS256 共有秘密鍵で検証（CI / dev）。
	AuthModeHMAC AuthMode = "hmac"
	// AuthModeJWKS は JWKS URL から取得した RSA 公開鍵で RS256 検証（production / Keycloak）。
	AuthModeJWKS AuthMode = "jwks"
)

// authClaims は JWT から取り出すクレーム（tenant_id 必須、Keycloak 互換）。
type authClaims struct {
	// テナント識別子（必須）。
	TenantID string `json:"tenant_id"`
	// JWT 標準クレーム（exp / iat / nbf / sub）。
	jwt.Claims
}

// Config は middleware の挙動を制御する。
type Config struct {
	// 動作モード（off / hmac / jwks）。
	Mode AuthMode
	// HS* 用共有秘密鍵（mode=hmac のみ使用）。
	HMACSecret []byte
	// JWKS endpoint URL（mode=jwks のみ使用）。
	JWKSURL string
	// JWKS cache TTL。0 で 10 分既定。
	JWKSCacheTTL time.Duration
	// HTTP client（test 注入可能）。
	HTTPClient *http.Client
}

// LoadConfigFromEnv は環境変数から Config を構築する。
//
// 既定 Mode は off（dev 既定）。production では T2_AUTH_MODE=jwks を必ず設定する。
func LoadConfigFromEnv() Config {
	mode := AuthMode(os.Getenv("T2_AUTH_MODE"))
	if mode == "" {
		mode = AuthModeOff
	}
	return Config{
		Mode:         mode,
		HMACSecret:   []byte(os.Getenv("T2_AUTH_HMAC_SECRET")),
		JWKSURL:      os.Getenv("T2_AUTH_JWKS_URL"),
		JWKSCacheTTL: 10 * time.Minute,
		HTTPClient:   http.DefaultClient,
	}
}

// jwksCache は JWKS の TTL 付き cache（複数 goroutine 安全）。
type jwksCache struct {
	mu        sync.RWMutex
	jwks      *jose.JSONWebKeySet
	expiresAt time.Time
	url       string
	ttl       time.Duration
	client    *http.Client
}

// fetch は JWKS を URL から取得する（cache miss / expire 時のみ）。
func (c *jwksCache) fetch(ctx context.Context) (*jose.JSONWebKeySet, error) {
	c.mu.RLock()
	if c.jwks != nil && time.Now().Before(c.expiresAt) {
		j := c.jwks
		c.mu.RUnlock()
		return j, nil
	}
	c.mu.RUnlock()
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.jwks != nil && time.Now().Before(c.expiresAt) {
		return c.jwks, nil
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.url, nil)
	if err != nil {
		return nil, fmt.Errorf("jwks fetch: %w", err)
	}
	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("jwks fetch: %w", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("jwks fetch: HTTP %d", resp.StatusCode)
	}
	var keys jose.JSONWebKeySet
	if err := json.NewDecoder(resp.Body).Decode(&keys); err != nil {
		return nil, fmt.Errorf("jwks decode: %w", err)
	}
	c.jwks = &keys
	c.expiresAt = time.Now().Add(c.ttl)
	return c.jwks, nil
}

// RequiredWithConfig は cfg を使う Required 内部実装。test で cfg を差し替えるために分離する。
func RequiredWithConfig(cfg Config) func(http.Handler) http.Handler {
	var jwks *jwksCache
	if cfg.Mode == AuthModeJWKS && cfg.JWKSURL != "" {
		ttl := cfg.JWKSCacheTTL
		if ttl <= 0 {
			ttl = 10 * time.Minute
		}
		client := cfg.HTTPClient
		if client == nil {
			client = http.DefaultClient
		}
		jwks = &jwksCache{url: cfg.JWKSURL, ttl: ttl, client: client}
	}
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authHeader := r.Header.Get("Authorization")
			if !strings.HasPrefix(authHeader, "Bearer ") {
				writeUnauthorized(w, "missing bearer token")
				return
			}
			token := strings.TrimPrefix(authHeader, "Bearer ")
			if strings.TrimSpace(token) == "" {
				writeUnauthorized(w, "empty token")
				return
			}
			subject, tenantID, err := authenticate(r.Context(), cfg, jwks, token)
			if err != nil {
				writeUnauthorized(w, err.Error())
				return
			}
			ctx := context.WithValue(r.Context(), SubjectKey, subject)
			ctx = context.WithValue(ctx, TenantIDKey, tenantID)
			ctx = context.WithValue(ctx, TokenKey, token)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

// Required はトークン必須の middleware を返す。env で T2_AUTH_MODE 等を読む。
func Required() func(http.Handler) http.Handler {
	return RequiredWithConfig(LoadConfigFromEnv())
}

// authenticate は token を mode に応じて検証し、subject / tenant_id を返す。
func authenticate(ctx context.Context, cfg Config, jwks *jwksCache, token string) (string, string, error) {
	switch cfg.Mode {
	case AuthModeOff:
		// dev 既定: token 内容を見ず demo-tenant に固定する（tier3 BFF off mode と同等）。
		return "dev", "demo-tenant", nil
	case AuthModeHMAC:
		if len(cfg.HMACSecret) == 0 {
			return "", "", errors.New("T2_AUTH_HMAC_SECRET not set")
		}
		parsed, err := jwt.ParseSigned(token, []jose.SignatureAlgorithm{jose.HS256, jose.HS384, jose.HS512})
		if err != nil {
			return "", "", fmt.Errorf("parse: %w", err)
		}
		var claims authClaims
		if err := parsed.Claims(cfg.HMACSecret, &claims); err != nil {
			return "", "", fmt.Errorf("verify: %w", err)
		}
		return finalizeClaims(&claims)
	case AuthModeJWKS:
		if jwks == nil {
			return "", "", errors.New("jwks not configured")
		}
		keys, err := jwks.fetch(ctx)
		if err != nil {
			return "", "", err
		}
		parsed, err := jwt.ParseSigned(token, []jose.SignatureAlgorithm{jose.RS256, jose.RS384, jose.RS512})
		if err != nil {
			return "", "", fmt.Errorf("parse: %w", err)
		}
		if len(parsed.Headers) == 0 {
			return "", "", errors.New("jwt has no header")
		}
		matches := keys.Key(parsed.Headers[0].KeyID)
		if len(matches) == 0 {
			return "", "", fmt.Errorf("kid %q not found in jwks", parsed.Headers[0].KeyID)
		}
		var claims authClaims
		if err := parsed.Claims(matches[0].Key, &claims); err != nil {
			return "", "", fmt.Errorf("verify: %w", err)
		}
		return finalizeClaims(&claims)
	default:
		return "", "", fmt.Errorf("unsupported T2_AUTH_MODE: %s", cfg.Mode)
	}
}

// finalizeClaims は標準クレームを検証し、必須フィールドを返す。
func finalizeClaims(claims *authClaims) (string, string, error) {
	if err := claims.Claims.ValidateWithLeeway(jwt.Expected{Time: time.Now()}, 30*time.Second); err != nil {
		return "", "", fmt.Errorf("standard claims: %w", err)
	}
	if claims.TenantID == "" {
		return "", "", errors.New("missing tenant_id claim")
	}
	if claims.Subject == "" {
		return "", "", errors.New("missing sub claim")
	}
	return claims.Subject, claims.TenantID, nil
}

// SubjectFromContext は middleware が attach した subject を取り出す。
func SubjectFromContext(ctx context.Context) string {
	v, ok := ctx.Value(SubjectKey).(string)
	if !ok {
		return ""
	}
	return v
}

// TenantIDFromContext は middleware が attach した tenant_id を取り出す。
// k1s0 SDK 呼出時に WithTenant で per-request 上書きするために使う。
func TenantIDFromContext(ctx context.Context) string {
	v, ok := ctx.Value(TenantIDKey).(string)
	if !ok {
		return ""
	}
	return v
}

// TokenFromContext は middleware が attach した生 Bearer token を取り出す。
func TokenFromContext(ctx context.Context) string {
	v, ok := ctx.Value(TokenKey).(string)
	if !ok {
		return ""
	}
	return v
}

// writeUnauthorized は 401 + JSON error を返す。
func writeUnauthorized(w http.ResponseWriter, msg string) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(http.StatusUnauthorized)
	_ = json.NewEncoder(w).Encode(map[string]any{
		"error": map[string]any{
			"code":     "E-T2-AUTH-001",
			"message":  msg,
			"category": "UNAUTHORIZED",
		},
	})
}
