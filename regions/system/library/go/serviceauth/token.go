package serviceauth

import (
	"fmt"
	"time"
)

// ServiceToken は OAuth2 Client Credentials フローで取得したサービストークン。
type ServiceToken struct {
	// AccessToken はアクセストークン文字列。
	AccessToken string
	// TokenType はトークンタイプ（通常 "Bearer"）。
	TokenType string
	// ExpiresAt はトークンの有効期限。
	ExpiresAt time.Time
	// Scope はトークンのスコープ。
	Scope string
}

// IsExpired はトークンが期限切れかどうかを返す。
func (t *ServiceToken) IsExpired() bool {
	return time.Now().After(t.ExpiresAt)
}

// ShouldRefresh はトークンの残り有効期間が閾値以下かどうかを返す。
// 期限切れの 30 秒前にリフレッシュを促す。
func (t *ServiceToken) ShouldRefresh() bool {
	return time.Now().Add(30 * time.Second).After(t.ExpiresAt)
}

// BearerHeader は "Bearer <token>" 形式のヘッダー値を返す。
func (t *ServiceToken) BearerHeader() string {
	return fmt.Sprintf("Bearer %s", t.AccessToken)
}
