// usecase パッケージは BFF-Proxy の認証フローに関するビジネスロジックを集約する。
// クリーンアーキテクチャの「usecase 層」として、handler が直接 oauth クライアントや
// セッションストアを操作していた処理を抽象化し、handler の責務を HTTP 変換のみに限定する。
// usecase は port インターフェースにのみ依存し、具体実装に依存しない。
package usecase

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"net/url"
	"time"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/port"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

// exchangeCodeTTL はワンタイム交換コードの有効期間（60秒）。
// モバイルクライアントが /auth/callback のリダイレクト後に /auth/exchange でセッションを
// 確立するまでの猶予時間として設定する。
const exchangeCodeTTL = 60 * time.Second

// AuthOAuthClient は AuthUseCase が必要とする OAuth2/OIDC 操作のインターフェース。
// port.OAuthClient のうち認証フロー（Login/Callback/Logout）に必要なメソッドのみを定義する。
// RefreshToken は ProxyUseCase が使用するため、ここでは含まない。
// handler.OAuthClient は本インターフェースの type alias として定義されるため、
// handler パッケージのモックは自動的にこのインターフェースを実装する。
type AuthOAuthClient interface {
	// AuthCodeURL は PKCE コードチャレンジ付きの認可コードフロー URL を構築する。
	AuthCodeURL(state, codeChallenge string) (string, error)

	// ExchangeCode は認可コードと PKCE verifier をトークンに交換する。
	ExchangeCode(ctx context.Context, code, codeVerifier string) (*oauth.TokenResponse, error)

	// ExtractClaims は JWKS 署名検証済みの ID トークンから subject と realm roles を返す。
	ExtractClaims(ctx context.Context, idToken string) (subject string, roles []string, err error)

	// LogoutURL は IdP のエンドセッションエンドポイント URL を返す。
	LogoutURL(idTokenHint, postLogoutRedirectURI string) (string, error)

	// ClearDiscoveryCache はキャッシュ済みの OIDC discovery 結果をクリアする。
	ClearDiscoveryCache()
}

// LoginInput は Login ユースケースの入力パラメータ。
// handler から受け取る HTTP リクエスト由来のパラメータを保持する。
type LoginInput struct {
	// RedirectTo はモバイルクライアント向けの認証後リダイレクト先 URL。
	// 省略可能。空文字の場合はモバイルリダイレクトを行わない。
	RedirectTo string
}

// LoginOutput は Login ユースケースの出力。
// handler が HTTP レスポンスを構築するために必要な情報を返す。
type LoginOutput struct {
	// AuthURL は IdP の認可エンドポイント URL。handler はここへリダイレクトする。
	AuthURL string
	// State は CSRF 保護用のランダム文字列。Cookie に保存する。
	State string
	// CodeVerifier は PKCE の code_verifier。Cookie に保存する。
	CodeVerifier string
	// AllowMobileRedirect は RedirectTo が許可されたスキームかどうか。
	// true の場合、handler は postAuthRedirectCookie を設定する。
	AllowMobileRedirect bool
}

// CallbackInput は Callback ユースケースの入力パラメータ。
type CallbackInput struct {
	// ExistingSessionID は認証前に存在するセッション ID。セッション固定化攻撃防止のため削除する。
	ExistingSessionID string
	// State はリクエストから取得した OAuth state パラメータ。
	State string
	// CookieState は Cookie から取得した OAuth state。State と一致することを検証する。
	CookieState string
	// Code は IdP から受け取った認可コード。
	Code string
	// CodeVerifier は PKCE の code_verifier。Cookie から取得する。
	CodeVerifier string
	// PostAuthRedirect はモバイルクライアント向けの認証後リダイレクト先 URL。
	PostAuthRedirect string
	// SessionTTL はセッションの有効期間。
	SessionTTL time.Duration
}

// CallbackOutput は Callback ユースケースの出力。
type CallbackOutput struct {
	// SessionID は新しく作成されたセッション ID。Cookie として発行する。
	SessionID string
	// CSRFToken はセッションに紐づく CSRF トークン。
	CSRFToken string
	// MobileRedirectURL はモバイルリダイレクト先 URL（交換コード付き）。
	// 空文字の場合はモバイルリダイレクトなし（ブラウザフロー）。
	MobileRedirectURL string
}

// LogoutInput は Logout ユースケースの入力パラメータ。
type LogoutInput struct {
	// SessionID は削除対象のセッション ID。Cookie から取得する。
	SessionID string
	// PostLogoutURI はログアウト後のリダイレクト先 URI。
	PostLogoutURI string
}

// LogoutOutput は Logout ユースケースの出力。
type LogoutOutput struct {
	// IdPLogoutURL は IdP のエンドセッション URL。空文字の場合は IdP ログアウト不要。
	IdPLogoutURL string
	// FallbackURI は IdP ログアウト URL が構築できなかった場合のフォールバック URI。
	FallbackURI string
}

// SessionCheckInput は Session 確認ユースケースの入力パラメータ。
type SessionCheckInput struct {
	// SessionID は確認対象のセッション ID。Cookie から取得する。
	SessionID string
}

// SessionCheckOutput は Session 確認ユースケースの出力。
type SessionCheckOutput struct {
	// Subject は OIDC subject（ユーザー識別子）。
	Subject string
	// CSRFToken はセッションに紐づく CSRF トークン。
	CSRFToken string
	// Roles は Keycloak realm roles。フロントエンドの権限判定に使用する。
	Roles []string
}

// ExchangeInput はワンタイム交換コード検証ユースケースの入力パラメータ。
type ExchangeInput struct {
	// Code はワンタイム交換コード。クエリパラメータから取得する。
	Code string
	// SessionTTL はセッションの有効期間（Cookie の maxAge に使用）。
	SessionTTL time.Duration
}

// ExchangeOutput はワンタイム交換コード検証ユースケースの出力。
type ExchangeOutput struct {
	// RealSessionID は実際のセッション ID。Cookie として発行する。
	RealSessionID string
	// Subject は OIDC subject（ユーザー識別子）。
	Subject string
	// CSRFToken はセッションに紐づく CSRF トークン。
	CSRFToken string
}

// AuthUseCase は認証フローに関するビジネスロジックを提供する。
// handler は OAuthClient や SessionStore を直接操作せず、AuthUseCase を通して操作する。
type AuthUseCase struct {
	// oauthClient は OAuth2/OIDC プロバイダーとの通信を担うポートインターフェース。
	oauthClient AuthOAuthClient
	// sessionStore はセッションデータの永続化を担うポートインターフェース。
	sessionStore port.SessionStore
}

// NewAuthUseCase は AuthUseCase のコンストラクタ。
// 依存するポートインターフェースを注入する。
func NewAuthUseCase(oauthClient AuthOAuthClient, sessionStore port.SessionStore) *AuthUseCase {
	return &AuthUseCase{
		oauthClient:  oauthClient,
		sessionStore: sessionStore,
	}
}

// Login は OIDC 認可コードフロー（PKCE）の開始処理を行う。
// PKCE コードチャレンジと state を生成し、IdP への認可 URL を返す。
// handler はこの出力を元に Cookie を設定し、IdP へリダイレクトする。
func (uc *AuthUseCase) Login(ctx context.Context, input LoginInput) (*LoginOutput, error) {
	// PKCE コードチャレンジ・verifier のペアを生成する
	pkce, err := oauth.NewPKCE()
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_PKCE_ERROR", Err: err}
	}

	// CSRF 攻撃防止のためのランダム state を生成する
	state, err := generateRandomHex(32)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_STATE_ERROR", Err: err}
	}

	// IdP の認可エンドポイント URL を構築する
	authURL, err := uc.oauthClient.AuthCodeURL(state, pkce.CodeChallenge)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_URL_ERROR", Err: err}
	}

	// モバイルクライアント向けリダイレクト先の検証
	allowMobileRedirect := input.RedirectTo != "" && isAllowedRedirectScheme(input.RedirectTo)

	return &LoginOutput{
		AuthURL:             authURL,
		State:               state,
		CodeVerifier:        pkce.CodeVerifier,
		AllowMobileRedirect: allowMobileRedirect,
	}, nil
}

// Callback は IdP からの認可コードコールバックを処理する。
// セッション固定化攻撃防止のために既存セッションを削除し、
// 認可コードをトークンに交換してセッションを作成する。
// モバイルフローでは交換コードを発行してカスタムスキーム URL を返す。
func (uc *AuthUseCase) Callback(ctx context.Context, input CallbackInput) (*CallbackOutput, error) {
	// セッション固定化攻撃を防止するため、認証前の既存セッションを削除する（S-03 対応）
	if input.ExistingSessionID != "" {
		// 削除失敗は警告として記録するが処理は続行する
		_ = uc.sessionStore.Delete(ctx, input.ExistingSessionID)
	}

	// state パラメータの検証（CSRF 保護）
	if input.CookieState == "" {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_STATE_MISSING"}
	}
	if input.State != input.CookieState {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_STATE_MISMATCH"}
	}

	// 認可コードの存在確認
	if input.Code == "" {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_CODE_MISSING"}
	}

	// PKCE verifier の存在確認
	if input.CodeVerifier == "" {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_VERIFIER_MISSING"}
	}

	// 認可コードとトークンの交換
	tokenResp, err := uc.oauthClient.ExchangeCode(ctx, input.Code, input.CodeVerifier)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_TOKEN_EXCHANGE_FAILED", Err: err}
	}

	// JWKS 署名検証付きで ID トークンから subject と realm roles を取得する
	subject, roles, err := uc.oauthClient.ExtractClaims(ctx, tokenResp.IDToken)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_ID_TOKEN_INVALID", Err: err}
	}

	// セッションに紐づく CSRF トークンを生成する
	csrfToken, err := generateRandomHex(32)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_CSRF_ERROR", Err: err}
	}

	// セッションデータを構築してストアに保存する
	sessData := &session.SessionData{
		AccessToken:  tokenResp.AccessToken,
		RefreshToken: tokenResp.RefreshToken,
		IDToken:      tokenResp.IDToken,
		ExpiresAt:    time.Now().Add(time.Duration(tokenResp.ExpiresIn) * time.Second).Unix(),
		CSRFToken:    csrfToken,
		Subject:      subject,
		Roles:        roles,
	}

	sessionID, err := uc.sessionStore.Create(ctx, sessData, input.SessionTTL)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_SESSION_CREATE_FAILED", Err: err}
	}

	// モバイルフロー: PostAuthRedirect が設定されている場合はワンタイム交換コードを発行する
	if input.PostAuthRedirect != "" {
		// ワンタイム交換コード: セッション ID への参照を短命なエントリとして保存する
		exchangeData := &session.SessionData{
			AccessToken: sessionID,
			ExpiresAt:   time.Now().Add(exchangeCodeTTL).Unix(),
		}
		exchangeCode, err := uc.sessionStore.Create(ctx, exchangeData, exchangeCodeTTL)
		if err != nil {
			return nil, &AuthUseCaseError{Code: "BFF_AUTH_EXCHANGE_CREATE_FAILED", Err: err}
		}

		// リダイレクト先 URL に交換コードを付与する（S-05 対応: url.Parse のエラーを検証する）
		redirectURL, err := url.Parse(input.PostAuthRedirect)
		if err != nil {
			return nil, &AuthUseCaseError{Code: "BFF_AUTH_REDIRECT_URL_INVALID", Err: err}
		}
		q := redirectURL.Query()
		q.Set("code", exchangeCode)
		redirectURL.RawQuery = q.Encode()

		return &CallbackOutput{
			SessionID:         sessionID,
			CSRFToken:         csrfToken,
			MobileRedirectURL: redirectURL.String(),
		}, nil
	}

	// ブラウザフロー: モバイルリダイレクトなし
	return &CallbackOutput{
		SessionID: sessionID,
		CSRFToken: csrfToken,
	}, nil
}

// Logout はセッションを削除し、IdP のエンドセッションエンドポイント URL を返す。
// IdP ログアウト URL が構築できない場合はフォールバック URI を返す。
func (uc *AuthUseCase) Logout(ctx context.Context, input LogoutInput) (*LogoutOutput, error) {
	// ログアウト時に OIDC discovery キャッシュをクリアし、次回ログインで最新情報を再取得させる
	uc.oauthClient.ClearDiscoveryCache()

	output := &LogoutOutput{
		FallbackURI: input.PostLogoutURI,
	}

	if input.SessionID == "" {
		return output, nil
	}

	// セッションデータを取得し、ID トークンを使って IdP ログアウト URL を構築する
	sess, _ := uc.sessionStore.Get(ctx, input.SessionID)

	// セッションをストアから削除する
	_ = uc.sessionStore.Delete(ctx, input.SessionID)

	// IdP ログアウト URL を構築する（id_token_hint 付き）
	if sess != nil && sess.IDToken != "" {
		logoutURL, err := uc.oauthClient.LogoutURL(sess.IDToken, input.PostLogoutURI)
		if err == nil {
			output.IdPLogoutURL = logoutURL
		}
	}

	return output, nil
}

// CheckSession はセッション ID からセッションデータを取得し、有効性を検証する。
// 有効なセッションの場合はユーザー情報を返す。
func (uc *AuthUseCase) CheckSession(ctx context.Context, input SessionCheckInput) (*SessionCheckOutput, error) {
	if input.SessionID == "" {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_SESSION_NOT_FOUND"}
	}

	sess, err := uc.sessionStore.Get(ctx, input.SessionID)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_SESSION_ERROR", Err: err}
	}

	if sess == nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_SESSION_NOT_FOUND"}
	}

	if sess.IsExpired() {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_SESSION_EXPIRED"}
	}

	// roles が nil の場合は空スライスに変換する（JSON で null ではなく [] になる）
	roles := sess.Roles
	if roles == nil {
		roles = []string{}
	}

	return &SessionCheckOutput{
		Subject:   sess.Subject,
		CSRFToken: sess.CSRFToken,
		Roles:     roles,
	}, nil
}

// ExchangeCode はワンタイム交換コードを検証し、実際のセッション ID を返す。
// モバイルクライアントが OAuth 認証完了後にセッションを確立するために使用する。
func (uc *AuthUseCase) ExchangeCode(ctx context.Context, input ExchangeInput) (*ExchangeOutput, error) {
	if input.Code == "" {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_EXCHANGE_CODE_MISSING"}
	}

	// 交換コードに対応するエントリをセッションストアから取得する
	exchangeData, err := uc.sessionStore.Get(ctx, input.Code)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_EXCHANGE_ERROR", Err: err}
	}

	// 交換コードが存在しないか期限切れの場合は 401 を返す
	if exchangeData == nil || exchangeData.IsExpired() {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_EXCHANGE_CODE_INVALID"}
	}

	// 実際のセッション ID を取得する（AccessToken フィールドに格納されている）
	realSessionID := exchangeData.AccessToken

	// 実際のセッションがまだ有効か確認する（交換コード削除より前に検証し、
	// セッション無効時に交換コードが消費されてしまう問題を防ぐ）
	realSession, err := uc.sessionStore.Get(ctx, realSessionID)
	if err != nil || realSession == nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_SESSION_NOT_FOUND"}
	}

	// 全検証通過後に交換コードを削除する（ワンタイム使用）
	_ = uc.sessionStore.Delete(ctx, input.Code)

	return &ExchangeOutput{
		RealSessionID: realSessionID,
		Subject:       realSession.Subject,
		CSRFToken:     realSession.CSRFToken,
	}, nil
}

// AuthUseCaseError は AuthUseCase が返すエラー型。
// エラーコードと元のエラーをラップする。
type AuthUseCaseError struct {
	// Code はエラーコード（例: "BFF_AUTH_STATE_MISMATCH"）。
	Code string
	// Err は元のエラー（省略可能）。
	Err error
}

// Error は error インターフェースの実装。
func (e *AuthUseCaseError) Error() string {
	if e.Err != nil {
		return e.Code + ": " + e.Err.Error()
	}
	return e.Code
}

// Unwrap は元のエラーを返す（errors.Is/As で辿れるようにする）。
func (e *AuthUseCaseError) Unwrap() error {
	return e.Err
}

// generateRandomHex はバイト長 n のランダムな hex エンコード文字列を生成する。
func generateRandomHex(n int) (string, error) {
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}

// isAllowedRedirectScheme はモバイルリダイレクト先 URL のスキームを検証する。
// allowlist 方式: k1s0:// スキームのみ許可する。
// denylist ではなく allowlist を使用することで、未知の危険スキームを漏れなくブロックする。
func isAllowedRedirectScheme(rawURL string) bool {
	parsedURL, err := url.Parse(rawURL)
	if err != nil || parsedURL.Scheme == "" {
		return false
	}

	// allowlist: k1s0 カスタムスキームのみ許可する
	if parsedURL.Scheme != "k1s0" {
		return false
	}

	// Host が空でないことを確認する（k1s0://host の形式であること）
	if parsedURL.Host == "" {
		return false
	}

	return true
}
