// 認可 middleware。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//   docs/03_要件定義/30_非機能要件/E_セキュリティ.md NFR-E-AC-001 / 003 / 005
//
// 役割:
//   tier3 BFF へのアクセスは Keycloak 発行の OIDC JWT を必須とする。
//   `Authorization: Bearer <jwt>` ヘッダから JWT を取り出し、env で指定された
//   検証 mode で署名・期限・テナント claim を検証する。tenant_id / sub / roles を
//   request context に attach し、後段の handler / k1s0Client に渡す。
//
// 検証モード（env BFF_AUTH_MODE で切替）:
//   - "off"  : 既定。デモ / 早期 dev 用途。"admin-token" は admin role を付与する後方互換。
//              tier1 への呼出には JWT が転送されないため、tier1 側 AuthInterceptor が off
//              でなければ呼出は失敗する（多層防御）。
//   - "hmac" : 共有秘密鍵で HS256/384/512 検証（CI / dev）。env BFF_AUTH_HMAC_SECRET。
//   - "jwks" : Keycloak の JWKS URL を参照して RS256/384/512 検証（production）。
//              env BFF_AUTH_JWKS_URL で endpoint を渡す。
//
// 元の middleware は token を 8 文字切り出して subject にする偽装実装で、
// "admin-token" 固定文字列が admin role を取得できる致命的バックドアがあった
// （署名 / 期限 / テナント claim を一切検証しない）。本ファイルでは:
//   - off mode: 後方互換のため "admin-token" は維持するが demo-tenant に固定し、
//     その他の token も demo-tenant に集約することで本番投入時の越境リスクを排除する
//   - hmac/jwks mode: 完全な JWT 検証で tier1 AuthInterceptor と同等のセキュリティ水準を提供

// Package auth は BFF の認可 middleware を提供する。
package auth

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/go-jose/go-jose/v4"
	"github.com/go-jose/go-jose/v4/jwt"

	bffErrors "github.com/k1s0/k1s0/src/tier3/bff/internal/shared/errors"
)

// contextKey は context 経由でユーザ識別を渡す際のキー。
type contextKey string

// SubjectKey は認証済 subject を context から取り出すキー。
const SubjectKey contextKey = "k1s0.subject"

// RolesKey は認証済 roles を context から取り出すキー。
const RolesKey contextKey = "k1s0.roles"

// TenantIDKey は認証済 tenant_id を context から取り出すキー。
// tier1 への呼出時に TenantContext.tenant_id へ詰めるために使う。
const TenantIDKey contextKey = "k1s0.tenant_id"

// TokenKey は受け取った生 JWT 文字列を context から取り出すキー。
// tier1 への gRPC 呼出時に Authorization メタデータに転送するために使う。
const TokenKey contextKey = "k1s0.bearer_token"

// AuthMode は middleware の動作モード。
type AuthMode string

const (
	// AuthModeOff は JWT 検証スキップ（dev / 早期 OSS 採用）。
	AuthModeOff AuthMode = "off"
	// AuthModeHMAC は HS256 共有秘密鍵で検証（CI / dev）。
	AuthModeHMAC AuthMode = "hmac"
	// AuthModeJWKS は JWKS URL から取得した RSA 公開鍵で RS256 検証（production / Keycloak）。
	AuthModeJWKS AuthMode = "jwks"
)

// AuthClaims は BFF が JWT から取り出すクレーム。
// Keycloak の標準クレーム配置（realm_access.roles）に追従する。
type AuthClaims struct {
	// テナント識別子（必須）。
	TenantID string `json:"tenant_id"`
	// Keycloak の realm_access.roles を取り出すための入れ子型。
	RealmAccess *struct {
		Roles []string `json:"roles"`
	} `json:"realm_access,omitempty"`
	// JWT 標準クレーム（exp / iat / nbf / sub）。
	jwt.Claims
}

// flattenedRoles は RealmAccess.Roles を平坦化して返す（Keycloak 互換）。
func (c *AuthClaims) flattenedRoles() []string {
	if c.RealmAccess == nil {
		return nil
	}
	out := make([]string, len(c.RealmAccess.Roles))
	copy(out, c.RealmAccess.Roles)
	return out
}

// Config は middleware の挙動を制御する。
type Config struct {
	Mode         AuthMode
	HMACSecret   []byte
	JWKSURL      string
	JWKSCacheTTL time.Duration
	HTTPClient   *http.Client
}

// LoadConfigFromEnv は env から Config を構築する。
// 既定は AuthModeOff（既存 dev / demo の "admin-token" 後方互換を維持）。
func LoadConfigFromEnv() Config {
	mode := AuthMode(os.Getenv("BFF_AUTH_MODE"))
	if mode == "" {
		mode = AuthModeOff
	}
	return Config{
		Mode:         mode,
		HMACSecret:   []byte(os.Getenv("BFF_AUTH_HMAC_SECRET")),
		JWKSURL:      os.Getenv("BFF_AUTH_JWKS_URL"),
		JWKSCacheTTL: 10 * time.Minute,
		HTTPClient:   http.DefaultClient,
	}
}

// jwksCache は JWKS の TTL 付き cache。
type jwksCache struct {
	mu        sync.RWMutex
	jwks      *jose.JSONWebKeySet
	expiresAt time.Time
	url       string
	ttl       time.Duration
	client    *http.Client
}

// fetch は JWKS を URL から取得する（cache miss / expire 時）。
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

// requiredWithConfig は cfg を使う Required 内部実装。test で cfg を差し替えるために分離する。
func requiredWithConfig(cfg Config, requireRole string) func(http.Handler) http.Handler {
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

			subject, tenantID, roles, err := authenticate(r.Context(), cfg, jwks, token)
			if err != nil {
				writeUnauthorized(w, err.Error())
				return
			}
			if requireRole != "" {
				ok := false
				for _, role := range roles {
					if role == requireRole {
						ok = true
						break
					}
				}
				if !ok {
					writeForbidden(w, "missing role: "+requireRole)
					return
				}
			}
			ctx := context.WithValue(r.Context(), SubjectKey, subject)
			ctx = context.WithValue(ctx, RolesKey, roles)
			ctx = context.WithValue(ctx, TenantIDKey, tenantID)
			ctx = context.WithValue(ctx, TokenKey, token)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

// Required はトークン必須の middleware を返す。requireRole が空文字なら role チェック skip。
// env で BFF_AUTH_MODE / BFF_AUTH_HMAC_SECRET / BFF_AUTH_JWKS_URL を読んで動作モードを決める。
func Required(requireRole string) func(http.Handler) http.Handler {
	return requiredWithConfig(LoadConfigFromEnv(), requireRole)
}

// authenticate は token を mode に応じて検証し、subject / tenant_id / roles を返す。
func authenticate(ctx context.Context, cfg Config, jwks *jwksCache, token string) (string, string, []string, error) {
	switch cfg.Mode {
	case AuthModeOff:
		return authenticateOff(token)
	case AuthModeHMAC:
		return authenticateHMAC(token, cfg.HMACSecret)
	case AuthModeJWKS:
		return authenticateJWKS(ctx, token, jwks)
	default:
		return "", "", nil, fmt.Errorf("unsupported BFF_AUTH_MODE: %s", cfg.Mode)
	}
}

// authenticateOff は dev / demo の後方互換ロジック。
// "admin-token" は admin role を付与する以外、token 内容を検証せず仮 subject を返す。
// production で本モードを使うことは禁止（tier1 が JWT を要求するため、tier1 呼出が失敗する）。
func authenticateOff(token string) (string, string, []string, error) {
	subject := "user-" + token[:min(8, len(token))]
	tenantID := "demo-tenant"
	roles := []string{"user"}
	if token == "admin-token" {
		subject = "admin-user"
		roles = []string{"admin", "user"}
	}
	return subject, tenantID, roles, nil
}

// authenticateHMAC は HS256/384/512 で JWT を検証し、必須クレームを取り出す。
func authenticateHMAC(token string, secret []byte) (string, string, []string, error) {
	if len(secret) == 0 {
		return "", "", nil, errors.New("hmac secret not set")
	}
	parsed, err := jwt.ParseSigned(token, []jose.SignatureAlgorithm{jose.HS256, jose.HS384, jose.HS512})
	if err != nil {
		return "", "", nil, fmt.Errorf("parse: %w", err)
	}
	var claims AuthClaims
	if err := parsed.Claims(secret, &claims); err != nil {
		return "", "", nil, fmt.Errorf("verify: %w", err)
	}
	return finalizeClaims(&claims)
}

// authenticateJWKS は JWKS から RSA 公開鍵を取り出して RS256 検証する。
func authenticateJWKS(ctx context.Context, token string, jwks *jwksCache) (string, string, []string, error) {
	if jwks == nil {
		return "", "", nil, errors.New("jwks not configured")
	}
	keys, err := jwks.fetch(ctx)
	if err != nil {
		return "", "", nil, err
	}
	parsed, err := jwt.ParseSigned(token, []jose.SignatureAlgorithm{jose.RS256, jose.RS384, jose.RS512})
	if err != nil {
		return "", "", nil, fmt.Errorf("parse: %w", err)
	}
	if len(parsed.Headers) == 0 {
		return "", "", nil, errors.New("jwt has no header")
	}
	kid := parsed.Headers[0].KeyID
	matches := keys.Key(kid)
	if len(matches) == 0 {
		return "", "", nil, fmt.Errorf("kid %q not found in jwks", kid)
	}
	var claims AuthClaims
	if err := parsed.Claims(matches[0].Key, &claims); err != nil {
		return "", "", nil, fmt.Errorf("verify: %w", err)
	}
	return finalizeClaims(&claims)
}

// finalizeClaims は標準クレームを検証し、必須フィールドを返却する。
func finalizeClaims(claims *AuthClaims) (string, string, []string, error) {
	if err := claims.Claims.ValidateWithLeeway(jwt.Expected{Time: time.Now()}, 30*time.Second); err != nil {
		return "", "", nil, fmt.Errorf("standard claims: %w", err)
	}
	if claims.TenantID == "" {
		return "", "", nil, errors.New("missing tenant_id claim")
	}
	if claims.Subject == "" {
		return "", "", nil, errors.New("missing sub claim")
	}
	roles := claims.flattenedRoles()
	if len(roles) == 0 {
		// roles 不在は最小権限 "user" のみ付与。
		roles = []string{"user"}
	}
	return claims.Subject, claims.TenantID, roles, nil
}

// SubjectFromContext は middleware が context にセットした subject を取り出す。
func SubjectFromContext(ctx context.Context) string {
	v, ok := ctx.Value(SubjectKey).(string)
	if !ok {
		return ""
	}
	return v
}

// RolesFromContext は middleware が context にセットした roles を取り出す。
func RolesFromContext(ctx context.Context) []string {
	v, ok := ctx.Value(RolesKey).([]string)
	if !ok {
		return nil
	}
	return v
}

// TenantIDFromContext は middleware が context にセットした tenant_id を取り出す。
// tier1 呼出時に TenantContext.tenant_id へ詰めるために使う。
func TenantIDFromContext(ctx context.Context) string {
	v, ok := ctx.Value(TenantIDKey).(string)
	if !ok {
		return ""
	}
	return v
}

// TokenFromContext は middleware が context にセットした生 Bearer token を取り出す。
// tier1 gRPC 呼出時に Authorization メタデータへ転送するために使う。
func TokenFromContext(ctx context.Context) string {
	v, ok := ctx.Value(TokenKey).(string)
	if !ok {
		return ""
	}
	return v
}

func writeUnauthorized(w http.ResponseWriter, msg string) {
	writeJSONError(w, bffErrors.New(bffErrors.CategoryUnauthorized, "E-T3-BFF-AUTH-001", msg))
}

func writeForbidden(w http.ResponseWriter, msg string) {
	writeJSONError(w, bffErrors.New(bffErrors.CategoryForbidden, "E-T3-BFF-AUTH-002", msg))
}

func writeJSONError(w http.ResponseWriter, de *bffErrors.DomainError) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(de.Category.HTTPStatus())
	_ = json.NewEncoder(w).Encode(map[string]any{
		"error": map[string]any{
			"code":     de.Code,
			"message":  de.Message,
			"category": string(de.Category),
		},
	})
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
