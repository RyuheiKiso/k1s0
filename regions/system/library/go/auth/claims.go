package authlib

import (
	"fmt"
	"time"

	"github.com/lestrrat-go/jwx/v2/jwt"
)

// RealmAccess は Keycloak の realm_access Claim を表す。
type RealmAccess struct {
	Roles []string `json:"roles"`
}

// RoleSet はリソースアクセスのロール一覧を表す。
type RoleSet struct {
	Roles []string `json:"roles"`
}

// Access is kept as an alias for backward compatibility.
type Access = RoleSet

// Claims は JWT トークンの Claims 構造体（認証認可設計.md 準拠）。
type Claims struct {
	Sub            string             `json:"sub"`
	Issuer         string             `json:"iss"`
	Audience       []string           `json:"aud"`
	ExpiresAt      time.Time          `json:"exp"`
	IssuedAt       time.Time          `json:"iat"`
	Jti            string             `json:"jti"`
	Typ            string             `json:"typ"`
	Azp            string             `json:"azp"`
	Scope          string             `json:"scope"`
	Username       string             `json:"preferred_username"`
	Email          string             `json:"email"`
	RealmAccess    RealmAccess        `json:"realm_access"`
	ResourceAccess map[string]RoleSet `json:"resource_access"`
	TierAccess     []string           `json:"tier_access"`

	// Deprecated compatibility fields.
	Iss               string `json:"-"`
	Aud               string `json:"-"`
	Exp               int64  `json:"-"`
	Iat               int64  `json:"-"`
	PreferredUsername string `json:"-"`
}

// extractClaims は jwt.Token から Claims 構造体を生成する。
func extractClaims(token jwt.Token) (*Claims, error) {
	claims := &Claims{
		Sub:       token.Subject(),
		Issuer:    token.Issuer(),
		ExpiresAt: token.Expiration(),
		IssuedAt:  token.IssuedAt(),
		Jti:       token.JwtID(),
	}
	claims.Iss = claims.Issuer
	claims.Exp = claims.ExpiresAt.Unix()
	claims.Iat = claims.IssuedAt.Unix()

	// aud
	claims.Audience = token.Audience()
	if len(claims.Audience) > 0 {
		claims.Aud = claims.Audience[0]
	}

	// typ
	if v, ok := token.Get("typ"); ok {
		if s, ok := v.(string); ok {
			claims.Typ = s
		}
	}

	// azp
	if v, ok := token.Get("azp"); ok {
		if s, ok := v.(string); ok {
			claims.Azp = s
		}
	}

	// scope
	if v, ok := token.Get("scope"); ok {
		if s, ok := v.(string); ok {
			claims.Scope = s
		}
	}

	// preferred_username
	if v, ok := token.Get("preferred_username"); ok {
		if s, ok := v.(string); ok {
			claims.Username = s
			claims.PreferredUsername = s
		}
	}

	// email
	if v, ok := token.Get("email"); ok {
		if s, ok := v.(string); ok {
			claims.Email = s
		}
	}

	// realm_access
	if v, ok := token.Get("realm_access"); ok {
		claims.RealmAccess = parseRealmAccess(v)
	}

	// resource_access
	if v, ok := token.Get("resource_access"); ok {
		claims.ResourceAccess = parseResourceAccess(v)
	}

	// tier_access
	if v, ok := token.Get("tier_access"); ok {
		claims.TierAccess = parseStringSlice(v)
	}

	return claims, nil
}

// parseRealmAccess は realm_access の値を RealmAccess に変換する。
func parseRealmAccess(v interface{}) RealmAccess {
	ra := RealmAccess{}
	m, ok := v.(map[string]interface{})
	if !ok {
		return ra
	}
	ra.Roles = parseStringSlice(m["roles"])
	return ra
}

// parseResourceAccess は resource_access の値を map[string]RoleSet に変換する。
func parseResourceAccess(v interface{}) map[string]RoleSet {
	result := make(map[string]RoleSet)
	m, ok := v.(map[string]interface{})
	if !ok {
		return result
	}
	for key, val := range m {
		access := RoleSet{}
		if am, ok := val.(map[string]interface{}); ok {
			access.Roles = parseStringSlice(am["roles"])
		}
		result[key] = access
	}
	return result
}

// parseStringSlice はインターフェースを []string に変換する。
func parseStringSlice(v interface{}) []string {
	if v == nil {
		return nil
	}
	arr, ok := v.([]interface{})
	if !ok {
		return nil
	}
	result := make([]string, 0, len(arr))
	for _, item := range arr {
		if s, ok := item.(string); ok {
			result = append(result, s)
		}
	}
	return result
}

// IsExpired はトークンの有効期限が切れているかを返す。
func (c *Claims) IsExpired() bool {
	exp := c.ExpiresAt
	if exp.IsZero() && c.Exp > 0 {
		exp = time.Unix(c.Exp, 0)
	}
	return time.Now().After(exp)
}

// String は Claims のデバッグ用文字列を返す。
func (c *Claims) String() string {
	return fmt.Sprintf("Claims{sub=%s, iss=%s, aud=%v, username=%s, email=%s}",
		c.Sub, c.Issuer, c.Audience, c.Username, c.Email)
}
