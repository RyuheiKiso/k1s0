package session

import "time"

// SessionData represents a user session stored in Redis.
type SessionData struct {
	// AccessToken is the OAuth2 bearer token for upstream API calls.
	AccessToken string `json:"access_token"`

	// RefreshToken is the OAuth2 refresh token for silent renewal.
	RefreshToken string `json:"refresh_token,omitempty"`

	// IDToken is the OIDC ID token (kept for logout).
	IDToken string `json:"id_token,omitempty"`

	// ExpiresAt is the access token expiration time (Unix timestamp).
	ExpiresAt int64 `json:"expires_at"`

	// CSRFToken is the per-session CSRF token bound to this session.
	CSRFToken string `json:"csrf_token"`

	// CSRFTokenCreatedAt は CSRF トークンの生成時刻（Unix タイムスタンプ）。
	// CSRF トークンの有効期間（30分 TTL）検証に使用する（H-12 監査対応）。
	CSRFTokenCreatedAt int64 `json:"csrf_token_created_at"`

	// Subject is the OIDC sub claim (user identifier).
	Subject string `json:"sub"`

	// Roles は Keycloak realm roles（admin 等の権限管理に使用）。
	Roles []string `json:"roles,omitempty"`

	// CreatedAt is when the session was created (Unix timestamp).
	CreatedAt int64 `json:"created_at"`

	// AbsoluteExpiry はセッションの絶対有効期限（Unix タイムスタンプ）（M-17 監査対応）。
	// スライディングウィンドウで TTL が延長されても、この時刻を超えたセッションは無効化される。
	AbsoluteExpiry int64 `json:"absolute_expiry,omitempty"`
}

// IsExpired returns true when the access token has expired.
func (s *SessionData) IsExpired() bool {
	return time.Now().Unix() > s.ExpiresAt
}

// IsAbsoluteExpired はセッションの絶対有効期限を超過しているかを返す（M-17 監査対応）。
// AbsoluteExpiry が設定されていない（0）場合は期限切れとみなさない。
func (s *SessionData) IsAbsoluteExpired() bool {
	if s.AbsoluteExpiry == 0 {
		return false
	}
	return time.Now().Unix() > s.AbsoluteExpiry
}

// Data は SessionData の Go 命名規約準拠の短縮エイリアス（§3.2 監査対応: stutter 命名を解消）。
// 新しいコードでは session.Data を使用すること。
type Data = SessionData

// ExchangeCodeData はモバイルフロー用ワンタイム交換コードのデータを表す。
// SessionData.AccessToken にセッション ID を格納する意味論的誤用を解消するため（H-5 監査対応）、
// 交換コード専用の構造体として分離する。
type ExchangeCodeData struct {
	// SessionID は交換コードが参照する実際のセッション ID。
	SessionID string
	// PostAuthRedirect は認証後のリダイレクト先 URL（現在は未使用。将来の拡張用フィールド）。
	PostAuthRedirect string
	// ExpiresAt は交換コードの有効期限（Unix タイムスタンプ）。
	ExpiresAt int64
}

// IsExpired は交換コードが期限切れかどうかを返す。
func (e *ExchangeCodeData) IsExpired() bool {
	return time.Now().Unix() > e.ExpiresAt
}
