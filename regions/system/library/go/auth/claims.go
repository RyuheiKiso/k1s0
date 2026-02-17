package auth

import (
	"fmt"
	"time"

	"github.com/lestrrat-go/jwx/v2/jwt"
)

// RealmAccess は Keycloak の realm_access Claim を表す。
type RealmAccess struct {
	Roles []string `json:"roles"`
}

// Access はリソースアクセスのロール一覧を表す。
type Access struct {
	Roles []string `json:"roles"`
}

// Claims は JWT トークンの Claims 構造体（認証認可設計.md 準拠）。
type Claims struct {
	Sub              string            `json:"sub"`
	Iss              string            `json:"iss"`
	Aud              string            `json:"aud"`
	Exp              int64             `json:"exp"`
	Iat              int64             `json:"iat"`
	Jti              string            `json:"jti"`
	Typ              string            `json:"typ"`
	Azp              string            `json:"azp"`
	Scope            string            `json:"scope"`
	PreferredUsername string           `json:"preferred_username"`
	Email            string            `json:"email"`
	RealmAccess      RealmAccess       `json:"realm_access"`
	ResourceAccess   map[string]Access `json:"resource_access"`
	TierAccess       []string          `json:"tier_access"`
}

// extractClaims は jwt.Token から Claims 構造体を生成する。
func extractClaims(token jwt.Token) (*Claims, error) {
	claims := &Claims{
		Sub: token.Subject(),
		Iss: token.Issuer(),
		Exp: token.Expiration().Unix(),
		Iat: token.IssuedAt().Unix(),
		Jti: token.JwtID(),
	}

	// aud
	auds := token.Audience()
	if len(auds) > 0 {
		claims.Aud = auds[0]
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

// parseResourceAccess は resource_access の値を map[string]Access に変換する。
func parseResourceAccess(v interface{}) map[string]Access {
	result := make(map[string]Access)
	m, ok := v.(map[string]interface{})
	if !ok {
		return result
	}
	for key, val := range m {
		access := Access{}
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

// ExpiresAt は Claims の有効期限を time.Time で返す。
func (c *Claims) ExpiresAt() time.Time {
	return time.Unix(c.Exp, 0)
}

// IsExpired はトークンの有効期限が切れているかを返す。
func (c *Claims) IsExpired() bool {
	return time.Now().After(c.ExpiresAt())
}

// String は Claims のデバッグ用文字列を返す。
func (c *Claims) String() string {
	return fmt.Sprintf("Claims{sub=%s, iss=%s, aud=%s, username=%s, email=%s}",
		c.Sub, c.Iss, c.Aud, c.PreferredUsername, c.Email)
}
