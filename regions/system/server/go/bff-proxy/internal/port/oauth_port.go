// port パッケージは BFF-Proxy の usecase 層が依存する外部サービスとのポートインターフェースを定義する。
// クリーンアーキテクチャの「依存性逆転の原則」に基づき、usecase は具体実装ではなく
// このインターフェースに依存する。これにより oauth パッケージへの直接依存をなくし、
// テスト時のモック差し替えも容易になる。
package port

import (
	"context"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
)

// OAuthClient は OAuth2/OIDC プロバイダー操作のポートインターフェース。
// oauth.Client 構造体の各メソッドに対応し、usecase 層から依存される。
// テスト時にモック差し替えを可能にする。
type OAuthClient interface {
	// AuthCodeURL は PKCE コードチャレンジ付きの認可コードフロー URL を構築する。
	AuthCodeURL(state, codeChallenge string) (string, error)

	// ExchangeCode は認可コードとPKCE verifier を使いトークンに交換する。
	ExchangeCode(ctx context.Context, code, codeVerifier string) (*oauth.TokenResponse, error)

	// ExtractClaims は JWKS 署名検証済みの ID トークンから subject と realm roles を返す。
	// アクセストークンの署名未検証によるロール改ざんリスクを排除するため、
	// 必ず ID トークン（検証済み）から roles を取得する。
	ExtractClaims(ctx context.Context, idToken string) (subject string, roles []string, err error)

	// RefreshToken はリフレッシュトークンを使い新しいトークンセットを取得する。
	RefreshToken(ctx context.Context, refreshToken string) (*oauth.TokenResponse, error)

	// LogoutURL は IdP のエンドセッションエンドポイント URL を返す。
	LogoutURL(idTokenHint, postLogoutRedirectURI string) (string, error)

	// ClearDiscoveryCache はキャッシュ済みの OIDC discovery 結果をクリアする。
	// ログアウト時に呼び出し、次回ログインで最新のプロバイダ情報を再取得させる。
	ClearDiscoveryCache()
}
