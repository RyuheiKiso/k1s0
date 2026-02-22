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

	// Subject is the OIDC sub claim (user identifier).
	Subject string `json:"sub"`

	// CreatedAt is when the session was created (Unix timestamp).
	CreatedAt int64 `json:"created_at"`
}

// IsExpired returns true when the access token has expired.
func (s *SessionData) IsExpired() bool {
	return time.Now().Unix() > s.ExpiresAt
}
