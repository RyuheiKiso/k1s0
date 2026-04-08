// testutil パッケージはテスト用の共通ヘルパー・ファクトリー関数を提供する。
// 各テストファイルで繰り返し使われるセッションデータや OIDC クレームの
// テスト用構造体を一元管理し、テストコードの重複を削減する。
// モック実装そのものは handler パッケージの auth_flow_test.go に残す（循環インポート回避）。
package testutil

import (
	"time"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

// SessionOptions は CreateTestSession のオプションパラメータ。
// 未指定フィールドはデフォルト値が使われる。
type SessionOptions struct {
	// AccessToken はセッションに格納するアクセストークン。省略時は "test-access-token"。
	AccessToken string
	// RefreshToken はリフレッシュトークン。省略時は "test-refresh-token"。
	RefreshToken string
	// IDToken は ID トークン。省略時は "test-id-token"。
	IDToken string
	// Subject は OIDC subject (ユーザー識別子)。省略時は "test-user-sub"。
	Subject string
	// CSRFToken は CSRF トークン。省略時は "test-csrf-token"。
	CSRFToken string
	// CSRFTokenCreatedAt は CSRF トークン生成時刻（Unix タイムスタンプ）。
	// 省略時は time.Now().Unix()（再生成しきい値を超えないよう現在時刻を使う）。
	// 0 を明示すると旧形式セッション（MED-001 変更前）のシミュレーションになる。
	CSRFTokenCreatedAt int64
	// Roles は Keycloak realm roles。省略時は []string{"user"}。
	Roles []string
	// TTL はセッションの有効期間。省略時は 1 時間（正常なセッション）。
	// 負の値を指定すると期限切れセッションを作成できる（例: -1 * time.Hour）。
	TTL time.Duration
}

// CreateTestSession はテスト用の SessionData を生成するヘルパー。
// opts で指定されたフィールドを上書きし、省略されたフィールドはデフォルト値を使う。
// 各テストケースで繰り返し作成するセッションデータの定型コードを削減する。
func CreateTestSession(opts SessionOptions) *session.SessionData {
	// デフォルト値を設定する
	accessToken := opts.AccessToken
	if accessToken == "" {
		accessToken = "test-access-token"
	}

	refreshToken := opts.RefreshToken
	if refreshToken == "" {
		refreshToken = "test-refresh-token"
	}

	idToken := opts.IDToken
	if idToken == "" {
		idToken = "test-id-token"
	}

	subject := opts.Subject
	if subject == "" {
		subject = "test-user-sub"
	}

	csrfToken := opts.CSRFToken
	if csrfToken == "" {
		csrfToken = "test-csrf-token"
	}

	roles := opts.Roles
	if roles == nil {
		roles = []string{"user"}
	}

	// CSRF トークン生成時刻: 省略時は現在時刻（再生成しきい値を超えない）
	// MED-001 監査対応: CSRFTokenCreatedAt=0 の旧セッションは TTL 超過と判定され再生成される。
	// テストがデフォルトで再生成をトリガーしないよう現在時刻をデフォルトとする。
	csrfCreatedAt := opts.CSRFTokenCreatedAt
	if csrfCreatedAt == 0 {
		csrfCreatedAt = time.Now().Unix()
	}

	// TTL に応じた有効期限を計算する
	ttl := opts.TTL
	if ttl == 0 {
		ttl = time.Hour
	}
	expiresAt := time.Now().Add(ttl).Unix()

	return &session.SessionData{
		AccessToken:        accessToken,
		RefreshToken:       refreshToken,
		IDToken:            idToken,
		Subject:            subject,
		CSRFToken:          csrfToken,
		CSRFTokenCreatedAt: csrfCreatedAt,
		Roles:              roles,
		ExpiresAt:          expiresAt,
	}
}

// CreateExpiredSession は期限切れの SessionData を生成するヘルパー。
// セッション期限切れに関するテストケースで使用する。
func CreateExpiredSession(subject, csrfToken string) *session.SessionData {
	return CreateTestSession(SessionOptions{
		Subject:   subject,
		CSRFToken: csrfToken,
		TTL:       -time.Hour, // 1 時間前に期限切れ
	})
}

// CreateExchangeCodeEntry はワンタイム交換コードのエントリを生成するヘルパー（H-5 監査対応）。
// モバイルフローの /auth/exchange エンドポイントのテストで使用する。
// H-5 監査対応: SessionData.AccessToken を流用せず、ExchangeCodeData 専用型を使用する。
// realSessionID には実際のセッション ID を指定する。
func CreateExchangeCodeEntry(realSessionID string, ttl time.Duration) *session.ExchangeCodeData {
	if ttl == 0 {
		ttl = 60 * time.Second
	}
	return &session.ExchangeCodeData{
		// SessionID フィールドに実セッション ID を格納する（意味論的に正確）
		SessionID: realSessionID,
		ExpiresAt: time.Now().Add(ttl).Unix(),
	}
}
