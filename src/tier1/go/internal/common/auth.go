// 本ファイルは tier1 facade の認証 / テナント上書き interceptor。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「認証と認可」:
//       - tier1 API への匿名アクセスは禁止する（NFR-E-AC-001）
//       - エンドユーザー文脈: Keycloak 発行 JWT を gRPC メタデータ
//         "authorization: Bearer <jwt>" に付与。`sub` / `tenant_id` / `roles` を tier1 が検証
//       - TenantContext の tenant_id は呼出側自己宣言ではなく、tier1 が JWT / SPIFFE ID から
//         導出して上書きする。クライアントから渡された値と不一致な場合は K1s0Error.PermissionDenied で即拒否
//     §「マルチテナント分離」L1:
//       JWT / SPIFFE ID から tenant_id を導出し、リクエストの TenantContext を上書き
//   docs/03_要件定義/30_非機能要件/E_セキュリティ.md NFR-E-AC-003
//
// 実装方針:
//   - JWT 検証は env で 3 mode 切替（OSS 採用時の運用裁量を確保）:
//       1. TIER1_AUTH_MODE=jwks  : JWKS URL から取得した RSA 公開鍵で RS256 検証（production）
//       2. TIER1_AUTH_MODE=hmac  : 環境変数 TIER1_AUTH_HMAC_SECRET（HS256） で検証（dev / CI）
//       3. TIER1_AUTH_MODE=off   : 認証スキップ。TenantContext を信頼（test / 早期 dev、既定）
//   - mode != off では JWT 不在を Unauthenticated、検証失敗を Unauthenticated、
//     tenant_id mismatch を PermissionDenied で返す
//   - 検証成功時は AuthInfo を context に attach（後段 handler / audit emitter で参照）
//
// 制限事項（意図的）:
//   - SPIFFE ID 経路（mTLS）は別 interceptor で処理する（peer.AuthInfo 経路）
//   - JWKS の自動更新（rotation）は最小実装: TTL 付き sync.Map cache、TTL 失効で再取得
//   - JWT クレーム名は OIDC 標準（sub / tenant_id）と Keycloak 互換。"tenant_id" claim を MUST とする

package common

import (
	// 全 RPC で context を伝搬する。
	"context"
	// JWKS の HTTP 取得。
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	// 環境変数読出。
	"os"
	// JWKS cache の sync 制御。
	"sync"
	"time"

	// JWT 検証ライブラリ（既に transitive dep）。
	"github.com/go-jose/go-jose/v4"
	"github.com/go-jose/go-jose/v4/jwt"
	// gRPC server / metadata / status。
	"google.golang.org/grpc"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// AuthMode は認証 interceptor の動作モード。
type AuthMode string

const (
	// AuthModeOff は認証スキップ（test / 早期 dev、既定）。
	AuthModeOff AuthMode = "off"
	// AuthModeHMAC は HS256 + 共有秘密鍵で JWT 検証（dev / CI）。
	AuthModeHMAC AuthMode = "hmac"
	// AuthModeJWKS は RS256 + JWKS URL で JWT 検証（production / Keycloak）。
	AuthModeJWKS AuthMode = "jwks"
)

// AuthClaims は tier1 が JWT から抽出する必須クレーム。
// Keycloak realm_access.roles を Roles に展開し、handler が RBAC 判定（NFR-E-AC-002）に使う。
type AuthClaims struct {
	// テナント識別子（必須、TenantContext.tenant_id と突き合わせる）。
	TenantID string `json:"tenant_id"`
	// 呼出主体（user_id / workload_id、TenantContext.subject に上書き）。
	Subject string `json:"sub"`
	// Keycloak Realm Role 集合（NFR-E-AC-002）。`realm_access.roles` から展開。
	RealmAccess RealmAccess `json:"realm_access,omitempty"`
	// JWT 標準クレーム（exp / iat / nbf 検証用）。
	jwt.Claims
}

// RealmAccess は Keycloak `realm_access` クレームの roles 配列。
type RealmAccess struct {
	// Realm Role 一覧。
	Roles []string `json:"roles,omitempty"`
}

// HasRole は claims が指定 role を含むかを判定する。
// Keycloak Realm Role の単純包含チェック。
func (c *AuthClaims) HasRole(role string) bool {
	// nil 防御。
	if c == nil {
		// false を返す。
		return false
	}
	// 線形探索（roles は通常数件のため十分高速）。
	for _, r := range c.RealmAccess.Roles {
		// 完全一致のみ許容する。
		if r == role {
			// 一致を返す。
			return true
		}
	}
	// 不在を返す。
	return false
}

// authInfoKey は context に attach する AuthInfo の key 型。
type authInfoKey struct{}

// AuthInfo は検証成功後に context に attach する型。handler / audit emitter は
// AuthFromContext で取得する。
type AuthInfo struct {
	TenantID string
	Subject  string
	Mode     AuthMode
	// Realm Role 一覧（NFR-E-AC-002 RBAC 判定）。off mode では空 slice。
	Roles []string
}

// HasRole は AuthInfo が指定 role を含むかを判定する（NFR-E-AC-002）。
func (a *AuthInfo) HasRole(role string) bool {
	// nil 防御。
	if a == nil {
		// false を返す。
		return false
	}
	// 線形探索（roles は通常数件）。
	for _, r := range a.Roles {
		// 完全一致のみ。
		if r == role {
			// 一致を返す。
			return true
		}
	}
	// 不在を返す。
	return false
}

// AuthFromContext は context から AuthInfo を取り出す。
// Auth interceptor が attach していない（mode=off / interceptor 未登録）場合は ok=false。
func AuthFromContext(ctx context.Context) (*AuthInfo, bool) {
	v, ok := ctx.Value(authInfoKey{}).(*AuthInfo)
	return v, ok
}

// EnforceTenantBoundary は handler 段で JWT 由来 tenant_id (context の AuthInfo) と
// proto body の tenant_id を比較し、一致しなければ PermissionDenied を返す。
// AuthInfo が context に無い場合（auth=off の dev 環境）は body 由来をそのまま採用。
//
// NFR-E-AC-003 二重防御:
//   - gRPC 直接経路: AuthInterceptor が req body から tenant_id を抽出し JWT と比較
//     （auth.go の interceptor 内 cross-tenant 検査）
//   - HTTP/JSON gateway 経路: invokeWithInterceptors() が req=nil で interceptor を
//     呼ぶため上記検査が silently skipped → handler 段で本関数を呼んで二重防御
//
// 戻り値は最終的に handler が adapter に渡すべき正規 tenant_id（JWT が優先）。
// 不正一致時の error は status.PermissionDenied（HTTP 403）。
func EnforceTenantBoundary(ctx context.Context, bodyTenantID string, rpc string) (string, error) {
	if bodyTenantID == "" {
		return "", status.Error(grpccodes.InvalidArgument, "tier1: tenant_id required in TenantContext ("+rpc+")")
	}
	if info, ok := AuthFromContext(ctx); ok && info != nil && info.TenantID != "" {
		if info.TenantID != bodyTenantID {
			return "", status.Errorf(grpccodes.PermissionDenied,
				"tier1: cross-tenant request rejected (%s): jwt=%q body=%q",
				rpc, info.TenantID, bodyTenantID)
		}
	}
	return bodyTenantID, nil
}

// AuthInterceptorConfig は AuthInterceptor の挙動を制御する。
type AuthInterceptorConfig struct {
	// Mode は AuthModeOff / AuthModeHMAC / AuthModeJWKS のいずれか。
	Mode AuthMode
	// HMACSecret は AuthModeHMAC 時に使う HS256 共有秘密鍵。
	HMACSecret []byte
	// JWKSURL は AuthModeJWKS 時に使う JWKS endpoint URL。
	JWKSURL string
	// JWKSCacheTTL は JWKS の cache 有効期間（既定 10 分）。
	JWKSCacheTTL time.Duration
	// SkipMethods は認証スキップする gRPC full method（例: 健康診断 endpoint）。
	SkipMethods map[string]bool
	// HTTPClient は JWKS 取得用（test 時に差し替え）。
	HTTPClient *http.Client
}

// LoadAuthConfigFromEnv は環境変数から AuthInterceptorConfig を構築する。
// 未設定時の既定は AuthModeOff（test / 早期 dev で interceptor を無効化したい場合に有用）。
func LoadAuthConfigFromEnv() AuthInterceptorConfig {
	mode := AuthMode(os.Getenv("TIER1_AUTH_MODE"))
	if mode == "" {
		mode = AuthModeOff
	}
	return AuthInterceptorConfig{
		Mode:         mode,
		HMACSecret:   []byte(os.Getenv("TIER1_AUTH_HMAC_SECRET")),
		JWKSURL:      os.Getenv("TIER1_AUTH_JWKS_URL"),
		JWKSCacheTTL: 10 * time.Minute,
		SkipMethods: map[string]bool{
			// gRPC 標準 health protocol は認証不要（K8s probe / LB）。
			"/grpc.health.v1.Health/Check":   true,
			"/grpc.health.v1.Health/Watch":   true,
			"/grpc.reflection.v1.ServerReflection/ServerReflectionInfo":      true,
			"/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo": true,
			// k1s0 独自 HealthService も無認証許容（probe / 監視経路）。
			"/k1s0.tier1.health.v1.HealthService/Liveness":  true,
			"/k1s0.tier1.health.v1.HealthService/Readiness": true,
		},
		HTTPClient: http.DefaultClient,
	}
}

// jwksCache は JWKS の TTL 付き cache。期限切れで自動再取得する。
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
	// 高速パス: cache hit + 未期限。
	c.mu.RLock()
	if c.jwks != nil && time.Now().Before(c.expiresAt) {
		j := c.jwks
		c.mu.RUnlock()
		return j, nil
	}
	c.mu.RUnlock()

	// slow path: HTTP 取得して cache 更新。
	c.mu.Lock()
	defer c.mu.Unlock()
	// double-check（他 goroutine が先に更新した場合）。
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

// AuthInterceptor は JWT 検証 + TenantContext 上書きを行う UnaryServerInterceptor。
// mode=off の場合は単純に pass-through（test / 早期 dev 用途）。
func AuthInterceptor(cfg AuthInterceptorConfig) grpc.UnaryServerInterceptor {
	// JWKS cache を mode=jwks 時のみ生成する。
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
	return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		// skip method（health / reflection 等）は認証不要で素通り。
		if cfg.SkipMethods[info.FullMethod] {
			return handler(ctx, req)
		}
		// mode=off は完全 pass-through（既存 test 互換のため）。
		if cfg.Mode == AuthModeOff {
			return handler(ctx, req)
		}

		// gRPC metadata から authorization ヘッダを取り出す。
		md, ok := metadata.FromIncomingContext(ctx)
		if !ok {
			return nil, status.Error(grpccodes.Unauthenticated, "tier1: missing metadata")
		}
		raw := firstAuthHeader(md)
		if raw == "" {
			return nil, status.Error(grpccodes.Unauthenticated, "tier1: missing authorization header")
		}
		token := stripBearerPrefix(raw)
		if token == "" {
			return nil, status.Error(grpccodes.Unauthenticated, "tier1: invalid authorization scheme (expected Bearer)")
		}

		// JWT を検証してクレームを取り出す。
		claims, err := verifyJWT(ctx, cfg, jwks, token)
		if err != nil {
			return nil, status.Errorf(grpccodes.Unauthenticated, "tier1: jwt verification failed: %v", err)
		}
		if claims.TenantID == "" {
			return nil, status.Error(grpccodes.Unauthenticated, "tier1: jwt missing tenant_id claim")
		}

		// L1 テナント上書き: TenantContext.tenant_id が JWT と一致しない場合は PermissionDenied。
		// req が GetContext() を持つ場合のみチェック（HealthService 等は GetContext を持たない）。
		if reqCtx := contextFromRequest(req); reqCtx != "" && reqCtx != claims.TenantID {
			return nil, status.Errorf(grpccodes.PermissionDenied,
				"tier1: tenant_id mismatch: request=%q jwt=%q", reqCtx, claims.TenantID)
		}

		// AuthInfo を context に attach し handler に渡す。Roles は Keycloak realm_access.roles から展開する。
		ctx = context.WithValue(ctx, authInfoKey{}, &AuthInfo{
			TenantID: claims.TenantID,
			Subject:  claims.Subject,
			Mode:     cfg.Mode,
			Roles:    append([]string(nil), claims.RealmAccess.Roles...),
		})
		return handler(ctx, req)
	}
}

// firstAuthHeader は metadata から最初の authorization 値を返す（gRPC 慣例で複数値あり得る）。
func firstAuthHeader(md metadata.MD) string {
	if vs := md.Get("authorization"); len(vs) > 0 {
		return vs[0]
	}
	return ""
}

// stripBearerPrefix は "Bearer <token>" の token 部分を返す（無効形式は空文字）。
func stripBearerPrefix(s string) string {
	const prefix = "Bearer "
	if len(s) <= len(prefix) {
		return ""
	}
	if s[:len(prefix)] != prefix && s[:len(prefix)] != "bearer " && s[:len(prefix)] != "BEARER " {
		return ""
	}
	return s[len(prefix):]
}

// contextFromRequest は request proto から TenantContext.tenant_id を取り出す。
// req が GetContext / GetTenantId を持たない場合は空文字を返す（チェックスキップ）。
func contextFromRequest(req interface{}) string {
	return extractTenantID(req)
}

// verifyJWT は cfg.Mode に従って JWT を検証し、クレームを返す。
func verifyJWT(ctx context.Context, cfg AuthInterceptorConfig, jwks *jwksCache, token string) (*AuthClaims, error) {
	switch cfg.Mode {
	case AuthModeHMAC:
		return verifyHMAC(token, cfg.HMACSecret)
	case AuthModeJWKS:
		if jwks == nil {
			return nil, errors.New("jwks not configured")
		}
		return verifyRSAFromJWKS(ctx, token, jwks)
	default:
		// AuthInterceptor 内では off は早期 return しているため到達しない。
		return nil, fmt.Errorf("unsupported auth mode: %s", cfg.Mode)
	}
}

// verifyHMAC は HS256 / HS384 / HS512 のいずれかで JWT を検証する。
func verifyHMAC(token string, secret []byte) (*AuthClaims, error) {
	if len(secret) == 0 {
		return nil, errors.New("hmac secret not set")
	}
	parsed, err := jwt.ParseSigned(token, []jose.SignatureAlgorithm{jose.HS256, jose.HS384, jose.HS512})
	if err != nil {
		return nil, fmt.Errorf("parse: %w", err)
	}
	var claims AuthClaims
	if err := parsed.Claims(secret, &claims); err != nil {
		return nil, fmt.Errorf("hmac verify: %w", err)
	}
	if err := claims.Claims.ValidateWithLeeway(jwt.Expected{Time: time.Now()}, 30*time.Second); err != nil {
		return nil, fmt.Errorf("standard claims: %w", err)
	}
	return &claims, nil
}

// verifyRSAFromJWKS は JWKS から RSA 公開鍵を取り出して RS256/384/512 を検証する。
func verifyRSAFromJWKS(ctx context.Context, token string, jwks *jwksCache) (*AuthClaims, error) {
	keys, err := jwks.fetch(ctx)
	if err != nil {
		return nil, err
	}
	parsed, err := jwt.ParseSigned(token, []jose.SignatureAlgorithm{jose.RS256, jose.RS384, jose.RS512})
	if err != nil {
		return nil, fmt.Errorf("parse: %w", err)
	}
	if len(parsed.Headers) == 0 {
		return nil, errors.New("jwt has no header")
	}
	kid := parsed.Headers[0].KeyID
	matches := keys.Key(kid)
	if len(matches) == 0 {
		return nil, fmt.Errorf("kid %q not found in jwks", kid)
	}
	var claims AuthClaims
	if err := parsed.Claims(matches[0].Key, &claims); err != nil {
		return nil, fmt.Errorf("rsa verify: %w", err)
	}
	if err := claims.Claims.ValidateWithLeeway(jwt.Expected{Time: time.Now()}, 30*time.Second); err != nil {
		return nil, fmt.Errorf("standard claims: %w", err)
	}
	return &claims, nil
}
