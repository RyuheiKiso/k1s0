// usecase パッケージは BFF-Proxy の認証フローに関するビジネスロジックを集約する。
// クリーンアーキテクチャの「usecase 層」として、handler が直接 oauth クライアントや
// セッションストアを操作していた処理を抽象化し、handler の責務を HTTP 変換のみに限定する。
// usecase は port インターフェースにのみ依存し、具体実装に依存しない。
package usecase

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"log/slog"
	"net/url"
	"time"

	bffmetrics "github.com/k1s0-platform/system-server-go-bff-proxy/internal/metrics"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/port"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/util"
)

// defaultExchangeCodeTTL はワンタイム交換コードのデフォルト有効期間（60秒）。
// POLY-003 監査対応: TTL は設定可能にし、AuthUseCase 生成時に注入する。
// NewAuthUseCase の exchangeCodeTTL 引数が 0 の場合はこのデフォルト値を使用する。
const defaultExchangeCodeTTL = 60 * time.Second

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
	// AbsoluteMaxTTL はセッションの絶対最大有効期間（M-17 監査対応）。
	// スライディングウィンドウで TTL が延長されても、この期間を超えたセッションは強制無効化される。
	AbsoluteMaxTTL time.Duration
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
	// exchangeCodeStore はモバイルフロー用ワンタイム交換コードの永続化を担うポートインターフェース（H-5 監査対応）。
	// SessionData.AccessToken へのセッション ID 格納という意味論的誤用を解消するために分離する。
	exchangeCodeStore port.ExchangeCodeStore
	// exchangeCodeTTL はワンタイム交換コードの有効期間（POLY-003 監査対応: 設定可能化）。
	exchangeCodeTTL time.Duration
}

// NewAuthUseCase は AuthUseCase のコンストラクタ。
// 依存するポートインターフェースを注入する。
// exchangeCodeStore は ExchangeCodeStore インターフェースを実装する必要がある。
// session.RedisStore と session.EncryptedStore は両方このインターフェースを実装している。
// exchangeCodeTTL が 0 の場合は defaultExchangeCodeTTL（60s）を使用する（POLY-003 監査対応）。
func NewAuthUseCase(oauthClient AuthOAuthClient, sessionStore port.SessionStore, exchangeCodeStore port.ExchangeCodeStore, exchangeCodeTTL time.Duration) *AuthUseCase {
	if exchangeCodeTTL <= 0 {
		exchangeCodeTTL = defaultExchangeCodeTTL
	}
	return &AuthUseCase{
		oauthClient:       oauthClient,
		sessionStore:      sessionStore,
		exchangeCodeStore: exchangeCodeStore,
		exchangeCodeTTL:   exchangeCodeTTL,
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

// validateCallbackInput は Callback 入力パラメータの存在確認と整合性検証を行う。
// 検証ロジックを分離することで Callback の循環複雑度を削減する（§3.2 監査対応: cc=14 → cc<12）。
func validateCallbackInput(input CallbackInput) error {
	if input.CookieState == "" {
		return &AuthUseCaseError{Code: "BFF_AUTH_STATE_MISSING"}
	}
	if input.State != input.CookieState {
		return &AuthUseCaseError{Code: "BFF_AUTH_STATE_MISMATCH"}
	}
	if input.Code == "" {
		return &AuthUseCaseError{Code: "BFF_AUTH_CODE_MISSING"}
	}
	if input.CodeVerifier == "" {
		return &AuthUseCaseError{Code: "BFF_AUTH_VERIFIER_MISSING"}
	}
	return nil
}

// buildMobileRedirectOutput はモバイルフロー用のワンタイム交換コードを生成して CallbackOutput を返す。
// モバイル固有のロジックを分離することで Callback の循環複雑度を削減する（§3.2 監査対応）。
// H-5 監査対応: SessionData.AccessToken へのセッション ID 格納という意味論的誤用を解消し、
// ExchangeCodeData 専用型を使用する。
func (uc *AuthUseCase) buildMobileRedirectOutput(ctx context.Context, sessionID, csrfToken, postAuthRedirect string) (*CallbackOutput, error) {
	// ワンタイム交換コード: ExchangeCodeData 専用型でセッション ID を保存する（H-5 監査対応）
	// SessionData.AccessToken を流用せず、意味論的に正確な ExchangeCodeData を使用する。
	exchangeData := &session.ExchangeCodeData{
		SessionID:        sessionID,
		PostAuthRedirect: postAuthRedirect,
		ExpiresAt:        time.Now().Add(uc.exchangeCodeTTL).Unix(),
	}
	exchangeCode, err := uc.exchangeCodeStore.CreateExchangeCode(ctx, exchangeData, uc.exchangeCodeTTL)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_EXCHANGE_CREATE_FAILED", Err: err}
	}

	// リダイレクト先 URL に交換コードを付与する（S-05 対応: url.Parse のエラーを検証する）
	redirectURL, err := url.Parse(postAuthRedirect)
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

// Callback は IdP からの認可コードコールバックを処理する。
// セッション固定化攻撃防止のために既存セッションを削除し、
// 認可コードをトークンに交換してセッションを作成する。
// モバイルフローでは交換コードを発行してカスタムスキーム URL を返す。
func (uc *AuthUseCase) Callback(ctx context.Context, input CallbackInput) (*CallbackOutput, error) {
	// セッション固定化攻撃を防止するため、認証前の既存セッションを削除する（S-03 対応）
	if input.ExistingSessionID != "" {
		// 削除失敗は警告ログを出力して処理を続行する（H-3 対応）
		// セッション ID は漏洩防止のためマスクして出力する（HIGH-7 対応）
		if err := uc.sessionStore.Delete(ctx, input.ExistingSessionID); err != nil {
			slog.WarnContext(ctx, "既存セッションの削除に失敗しました（処理は続行します）",
				"session_id", util.MaskSessionID(input.ExistingSessionID),
				"error", err,
			)
			// L-003 監査対応: コールバック時の既存セッション削除失敗をメトリクスに記録する
			bffmetrics.SessionDeleteErrorsTotal.WithLabelValues("callback").Inc()
		}
	}

	// 入力パラメータの検証（state/code/verifier の存在確認と整合性チェック）
	if err := validateCallbackInput(input); err != nil {
		return nil, err
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
	now := time.Now()
	sessData := &session.SessionData{
		AccessToken:        tokenResp.AccessToken,
		RefreshToken:       tokenResp.RefreshToken,
		IDToken:            tokenResp.IDToken,
		ExpiresAt:          now.Add(time.Duration(tokenResp.ExpiresIn) * time.Second).Unix(),
		CSRFToken:          csrfToken,
		CSRFTokenCreatedAt: now.Unix(), // H-12 監査対応: CSRF トークンの TTL 検証用に生成時刻を記録する
		Subject:            subject,
		Roles:              roles,
	}
	// M-17 監査対応: セッションの絶対有効期限を設定する。
	// スライディングウィンドウで TTL が延長されても、この期限を超えたセッションは無効化される。
	if input.AbsoluteMaxTTL > 0 {
		sessData.AbsoluteExpiry = now.Add(input.AbsoluteMaxTTL).Unix()
	}

	sessionID, err := uc.sessionStore.Create(ctx, sessData, input.SessionTTL)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_SESSION_CREATE_FAILED", Err: err}
	}

	// モバイルフロー: PostAuthRedirect が設定されている場合はワンタイム交換コードを発行する
	// M-9 監査対応: Cookie から取得した postAuthRedirect を UseCase 側でも再検証する。
	// handler 側での Cookie 設定時の検証（Login の isAllowedRedirectScheme）に加え、
	// UseCase 入口でも再検証することで多層防御を実現し、オープンリダイレクト攻撃を防ぐ。
	if input.PostAuthRedirect != "" {
		if !isAllowedRedirectScheme(input.PostAuthRedirect) {
			return nil, &AuthUseCaseError{Code: "BFF_AUTH_REDIRECT_URL_INVALID"}
		}
		return uc.buildMobileRedirectOutput(ctx, sessionID, csrfToken, input.PostAuthRedirect)
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
	// 取得失敗は警告ログを出力して処理を続行する（H-3 対応）
	// セッション ID は漏洩防止のためマスクして出力する（HIGH-7 対応）
	sess, err := uc.sessionStore.Get(ctx, input.SessionID)
	if err != nil {
		slog.WarnContext(ctx, "ログアウト時のセッション取得に失敗しました",
			"session_id", util.MaskSessionID(input.SessionID),
			"error", err,
		)
	}

	// セッションをストアから削除する（削除失敗は警告ログを出力して処理を続行する）（H-3 対応）
	// セッション ID は漏洩防止のためマスクして出力する（HIGH-7 対応）
	if err := uc.sessionStore.Delete(ctx, input.SessionID); err != nil {
		slog.WarnContext(ctx, "ログアウト時のセッション削除に失敗しました",
			"session_id", util.MaskSessionID(input.SessionID),
			"error", err,
		)
		// L-003 監査対応: ログアウト時のセッション削除失敗をメトリクスに記録する
		bffmetrics.SessionDeleteErrorsTotal.WithLabelValues("logout").Inc()
	}

	// IdP ログアウト URL を構築する（id_token_hint 付き）
	if sess != nil && sess.IDToken != "" {
		logoutURL, err := uc.oauthClient.LogoutURL(sess.IDToken, input.PostLogoutURI)
		if err == nil {
			output.IdPLogoutURL = logoutURL
		}
	}

	return output, nil
}

// csrfRefreshThreshold は CSRF トークンを再生成するタイムスタンプのしきい値。
// TTL（30分）の残り5分を切った時点で新しいトークンを発行する（H-003 監査対応）。
// これにより CSRF トークン TTL 切れによる 403 エラーを防止しつつ、
// 窃取されたトークンの有効期間を最小化する。
const csrfRefreshThreshold = 25 * time.Minute

// CheckSession はセッション ID からセッションデータを取得し、有効性を検証する。
// 有効なセッションの場合はユーザー情報を返す。
// H-003 監査対応: CSRF トークンが TTL（30分）のしきい値（25分）を超えた場合に再生成する。
// 再生成したトークンはセッションに保存し、クライアントに返す。
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

	// H-003 監査対応: CSRF トークンの生成から csrfRefreshThreshold（25分）を超えた場合は再生成する。
	// CSRFTokenCreatedAt == 0 の旧形式セッションは再生成をスキップする（後方互換性のため）。
	csrfToken := sess.CSRFToken
	if sess.CSRFTokenCreatedAt > 0 {
		csrfAge := time.Since(time.Unix(sess.CSRFTokenCreatedAt, 0))
		if csrfAge > csrfRefreshThreshold {
			// 新しい CSRF トークンを生成してセッションを更新する
			newCSRFToken, err := generateRandomHex(32)
			if err != nil {
				return nil, &AuthUseCaseError{Code: "BFF_AUTH_CSRF_ERROR", Err: err}
			}
			sess.CSRFToken = newCSRFToken
			sess.CSRFTokenCreatedAt = time.Now().Unix()
			// セッションストアにトークン更新を反映する（TTL はセッション TTL のデフォルト値を維持する）
			// TTL は SessionCheckInput に含まれないため 0 を渡すと一部ストアで問題が起きる可能性があるため
			// Update は TTL を受け取るが、ここでは残存 TTL を再設定するため sessionTTL 相当のデフォルト値を使用する。
			// セッション自体の有効期間は SessionMiddleware の Touch で管理されるため、
			// ここでは実際のアクセストークン有効期限（ExpiresAt）までを TTL として渡す。
			remainingTTL := time.Until(time.Unix(sess.ExpiresAt, 0))
			if remainingTTL <= 0 {
				remainingTTL = 30 * time.Minute
			}
			if updateErr := uc.sessionStore.Update(ctx, input.SessionID, sess, remainingTTL); updateErr != nil {
				// セッション更新失敗は警告ログを出力して処理を続行する（旧トークンを返す）
				slog.WarnContext(ctx, "CSRF トークン再生成後のセッション更新に失敗しました（旧トークンを返します）",
					"error", updateErr,
				)
				csrfToken = sess.CSRFToken
			} else {
				csrfToken = newCSRFToken
			}
		}
	}

	// roles が nil の場合は空スライスに変換する（JSON で null ではなく [] になる）
	roles := sess.Roles
	if roles == nil {
		roles = []string{}
	}

	return &SessionCheckOutput{
		Subject:   sess.Subject,
		CSRFToken: csrfToken,
		Roles:     roles,
	}, nil
}

// ExchangeCode はワンタイム交換コードを検証し、実際のセッション ID を返す。
// モバイルクライアントが OAuth 認証完了後にセッションを確立するために使用する。
// H-5 監査対応: ExchangeCodeStore から ExchangeCodeData を取得し、意味論的に正確なフィールドを使用する。
func (uc *AuthUseCase) ExchangeCode(ctx context.Context, input ExchangeInput) (*ExchangeOutput, error) {
	if input.Code == "" {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_EXCHANGE_CODE_MISSING"}
	}

	// 交換コードに対応するエントリを ExchangeCodeStore から取得する（H-5 監査対応）
	// SessionData.AccessToken を流用せず、ExchangeCodeData.SessionID フィールドを使用する。
	exchangeData, err := uc.exchangeCodeStore.GetExchangeCode(ctx, input.Code)
	if err != nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_EXCHANGE_ERROR", Err: err}
	}

	// 交換コードが存在しないか期限切れの場合は 401 を返す
	if exchangeData == nil || exchangeData.IsExpired() {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_EXCHANGE_CODE_INVALID"}
	}

	// 意味論的に正確な SessionID フィールドから実際のセッション ID を取得する（H-5 監査対応）
	realSessionID := exchangeData.SessionID

	// 実際のセッションがまだ有効か確認する（交換コード削除より前に検証し、
	// セッション無効時に交換コードが消費されてしまう問題を防ぐ）
	realSession, err := uc.sessionStore.Get(ctx, realSessionID)
	if err != nil || realSession == nil {
		return nil, &AuthUseCaseError{Code: "BFF_AUTH_SESSION_NOT_FOUND"}
	}

	// 全検証通過後に交換コードを削除する（ワンタイム使用）（削除失敗は警告ログを出力して処理を続行する）（H-3 対応）
	if err := uc.exchangeCodeStore.DeleteExchangeCode(ctx, input.Code); err != nil {
		slog.WarnContext(ctx, "交換コードの削除に失敗しました",
			"code", input.Code,
			"error", err,
		)
	}

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
