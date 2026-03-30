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

	// L-15 監査対応: 後方互換フィールド廃止スケジュール
	// これらのフィールドは旧クライアントとの互換性のために残されているが、
	// 新規コードでは上記の正式フィールド（Issuer, Audience, ExpiresAt, IssuedAt, Username）を使用すること。
	//
	// 廃止タイムライン:
	//   Phase 1（即時）: 新規コードでの使用を禁止し、既存コードを新フィールドに移行開始
	//   Phase 2（2026-06-30 完了目標）: 全クライアントを新フィールドへ移行完了
	//   Phase 3（2026-09-30 完了目標）: これらの後方互換フィールドをコードベースから削除
	//
	// 参考: ADR-0020（Deprecated フィールドの段階的移行計画）
	//
	// Deprecated: Iss は Issuer を使用すること。
	Iss string `json:"-"`
	// Deprecated: Aud は Audience を使用すること。
	Aud string `json:"-"`
	// Deprecated: Exp は ExpiresAt を使用すること。
	Exp int64 `json:"-"`
	// Deprecated: Iat は IssuedAt を使用すること。
	Iat int64 `json:"-"`
	// Deprecated: PreferredUsername は Username を使用すること。
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

// parseRealmAccess は realm_access の値を RealmAccess に変換する（any: Go 1.18+ 推奨エイリアスを使用する）。
func parseRealmAccess(v any) RealmAccess {
	ra := RealmAccess{}
	m, ok := v.(map[string]any)
	if !ok {
		return ra
	}
	ra.Roles = parseStringSlice(m["roles"])
	return ra
}

// parseResourceAccess は resource_access の値を map[string]RoleSet に変換する（any: Go 1.18+ 推奨エイリアスを使用する）。
func parseResourceAccess(v any) map[string]RoleSet {
	result := make(map[string]RoleSet)
	m, ok := v.(map[string]any)
	if !ok {
		return result
	}
	for key, val := range m {
		access := RoleSet{}
		if am, ok := val.(map[string]any); ok {
			access.Roles = parseStringSlice(am["roles"])
		}
		result[key] = access
	}
	return result
}

// parseStringSlice はインターフェースを []string に変換する（any: Go 1.18+ 推奨エイリアスを使用する）。
func parseStringSlice(v any) []string {
	if v == nil {
		return nil
	}
	arr, ok := v.([]any)
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
// Exp フィールドへのフォールバックを削除し、ExpiresAt のみを参照する（L-003）。
// extractClaims が常に ExpiresAt を設定するため、フォールバックは不要。
func (c *Claims) IsExpired() bool {
	return time.Now().After(c.ExpiresAt)
}

// maskEmail はメールアドレスの PII をマスキングする。
// "@" より前の部分の先頭1文字を残して "***" で置換する。
// 例: "user@example.com" → "u***@example.com"
// "@" がない場合や空文字列の場合は "***" を返す。
func maskEmail(email string) string {
	if email == "" {
		return "***"
	}
	atIdx := -1
	for i, ch := range email {
		if ch == '@' {
			atIdx = i
			break
		}
	}
	if atIdx < 0 {
		return "***"
	}
	// 先頭1文字 + "***" + "@以降" の形式でマスク
	return email[:1] + "***" + email[atIdx:]
}

// String は Claims のデバッグ用文字列を返す。
// email は PII のため maskEmail でマスキングして出力する。
func (c *Claims) String() string {
	return fmt.Sprintf("Claims{sub=%s, iss=%s, aud=%v, username=%s, email=%s}",
		c.Sub, c.Issuer, c.Audience, c.Username, maskEmail(c.Email))
}
