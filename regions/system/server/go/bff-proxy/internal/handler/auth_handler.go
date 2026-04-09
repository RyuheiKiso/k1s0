// auth_handler.go は OAuth2/OIDC 認証フローの HTTP ハンドラーを提供する。
// Login/Callback/Logout/Session/Exchange の各エンドポイントを実装する。
// 認証フローのビジネスロジックは AuthUseCase に委譲し、
// このハンドラーは HTTP リクエスト/レスポンスの変換（Cookie の読み書き、リダイレクト等）のみを担当する。
package handler

import (
	"crypto/rand"
	"encoding/hex"
	"html"
	"log/slog"
	"net/http"
	"time"
	"unicode/utf8"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/port"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/usecase"
)

const (
	// CookieName is the session cookie name.
	CookieName = "k1s0_session"

	// stateCookieName holds the OAuth state for CSRF protection during login.
	stateCookieName = "k1s0_oauth_state"

	// verifierCookieName holds the PKCE code_verifier during the auth flow.
	verifierCookieName = "k1s0_pkce_verifier"

	// postAuthRedirectCookie はモバイルクライアント向けの認証後リダイレクト先を保持する。
	postAuthRedirectCookie = "k1s0_post_auth_redirect"
)

// OAuthClient は OAuth2/OIDC プロバイダー操作のインターフェース。
// usecase.AuthOAuthClient の type alias として定義し、
// テスト時のモック差し替えを可能にしつつ循環参照を避ける。
type OAuthClient = usecase.AuthOAuthClient

// AuthHandler は OAuth2/OIDC ブラウザ認証フローを処理する HTTP ハンドラー。
// ビジネスロジックは AuthUseCase に委譲し、HTTP 変換のみを担当する。
type AuthHandler struct {
	// authUseCase は認証フローのビジネスロジックを提供する。
	authUseCase   *usecase.AuthUseCase
	sessionTTL    time.Duration
	postLogoutURI string
	secureCookie  bool
	// cookieDomain は発行する Cookie の Domain 属性。空文字の場合はブラウザがオリジンから自動設定する。
	cookieDomain string
	logger       *slog.Logger
	// absoluteMaxTTL はセッションの絶対最大有効期間（M-17 監査対応）。
	// スライディングウィンドウで TTL が延長されても、この期間を超えたセッションは無効化される。
	absoluteMaxTTL time.Duration
}

// NewAuthHandler は AuthHandler を生成する。
// oauthClient と sessionStore を受け取り、AuthUseCase を内部で構築する。
// exchangeCodeStore はモバイルフロー用ワンタイム交換コードの永続化ストアで、
// session.RedisStore または session.EncryptedStore を渡す（H-5 監査対応）。
// absoluteMaxTTL はセッションの絶対最大有効期間（M-17 監査対応）。
// exchangeCodeTTL はワンタイム交換コードの有効期間（POLY-003 監査対応: 設定可能化）。0 の場合は 60s。
func NewAuthHandler(
	oauthClient OAuthClient,
	sessionStore port.SessionStore,
	exchangeCodeStore port.ExchangeCodeStore,
	sessionTTL time.Duration,
	absoluteMaxTTL time.Duration,
	exchangeCodeTTL time.Duration,
	postLogoutURI string,
	secureCookie bool,
	cookieDomain string,
	logger *slog.Logger,
) *AuthHandler {
	return &AuthHandler{
		authUseCase:    usecase.NewAuthUseCase(oauthClient, sessionStore, exchangeCodeStore, exchangeCodeTTL),
		sessionTTL:     sessionTTL,
		absoluteMaxTTL: absoluteMaxTTL,
		postLogoutURI:  postLogoutURI,
		secureCookie:   secureCookie,
		cookieDomain:   cookieDomain,
		logger:         logger,
	}
}

// Login は OIDC 認可コードフロー（PKCE）を開始する。
// AuthUseCase.Login にビジネスロジックを委譲し、Cookie 設定と IdP へのリダイレクトを行う。
func (h *AuthHandler) Login(c *gin.Context) {
	out, err := h.authUseCase.Login(c.Request.Context(), usecase.LoginInput{
		RedirectTo: c.Query("redirect_to"),
	})
	if err != nil {
		usecaseErr, ok := err.(*usecase.AuthUseCaseError)
		if ok {
			h.logger.Error("Login ユースケースエラー", slog.String("code", usecaseErr.Code), slog.Any("error", usecaseErr.Err))
			respondError(c, http.StatusInternalServerError, usecaseErr.Code)
		} else {
			h.logger.Error("Login 予期しないエラー", slog.Any("error", err))
			respondError(c, http.StatusInternalServerError, "BFF_AUTH_INTERNAL_ERROR")
		}
		return
	}

	// state と PKCE verifier を短命な Cookie に保存する（有効期限: 5分）
	maxAge := 300
	c.SetSameSite(http.SameSiteLaxMode)
	c.SetCookie(stateCookieName, out.State, maxAge, "/", h.cookieDomain, h.secureCookie, true)
	c.SetCookie(verifierCookieName, out.CodeVerifier, maxAge, "/", h.cookieDomain, h.secureCookie, true)

	// モバイルクライアント向け: 許可されたスキームの場合のみ認証後リダイレクト先を保存する
	if out.AllowMobileRedirect {
		c.SetCookie(postAuthRedirectCookie, c.Query("redirect_to"), maxAge, "/", h.cookieDomain, h.secureCookie, true)
	}

	c.Redirect(http.StatusFound, out.AuthURL)
}

// Callback は IdP からのコールバックを処理する。
// AuthUseCase.Callback にビジネスロジックを委譲し、セッション Cookie の設定とレスポンスを行う。
func (h *AuthHandler) Callback(c *gin.Context) {
	// 既存セッション ID（セッション固定化攻撃防止のため UseCase に渡す）
	existingSessionID, _ := c.Cookie(CookieName)

	// Cookie State の取得（不在は UseCase 側で BFF_AUTH_STATE_MISSING として処理する）
	cookieState, _ := c.Cookie(stateCookieName)

	// PKCE verifier の取得（不在は UseCase 側で BFF_AUTH_VERIFIER_MISSING として処理する）
	verifier, _ := c.Cookie(verifierCookieName)

	// IdP からのエラーレスポンスを確認する
	// M-8 監査対応: error_description を HTML エスケープ + 256 文字制限でサニタイズしてから
	// ログ出力およびレスポンスに使用する（未サニタイズでの情報露出・XSS リスクを排除する）
	if errCode := c.Query("error"); errCode != "" {
		sanitizedDesc := sanitizeErrorDescription(c.Query("error_description"))
		h.logger.Warn("OIDC callback error",
			slog.String("error", errCode),
			slog.String("description", sanitizedDesc),
		)
		c.JSON(http.StatusBadRequest, gin.H{
			"error":       "BFF_AUTH_IDP_ERROR",
			"description": sanitizedDesc,
			"request_id":  middleware.GetRequestID(c),
		})
		return
	}

	// モバイルクライアント向けリダイレクト先（Cookie から取得）
	postAuthRedirect, _ := c.Cookie(postAuthRedirectCookie)

	out, err := h.authUseCase.Callback(c.Request.Context(), usecase.CallbackInput{
		ExistingSessionID: existingSessionID,
		State:             c.Query("state"),
		CookieState:       cookieState,
		Code:              c.Query("code"),
		CodeVerifier:      verifier,
		PostAuthRedirect:  postAuthRedirect,
		SessionTTL:        h.sessionTTL,
		AbsoluteMaxTTL:    h.absoluteMaxTTL, // M-17 監査対応: セッション絶対有効期限
	})
	if err != nil {
		usecaseErr, ok := err.(*usecase.AuthUseCaseError)
		if !ok {
			h.logger.Error("Callback 予期しないエラー", slog.Any("error", err))
			respondError(c, http.StatusInternalServerError, "BFF_AUTH_INTERNAL_ERROR")
			return
		}
		// エラーコードに応じた HTTP ステータスコードを返す
		switch usecaseErr.Code {
		case "BFF_AUTH_STATE_MISSING", "BFF_AUTH_STATE_MISMATCH",
			"BFF_AUTH_CODE_MISSING", "BFF_AUTH_VERIFIER_MISSING",
			"BFF_AUTH_REDIRECT_URL_INVALID":
			respondBadRequest(c, usecaseErr.Code)
		case "BFF_AUTH_ID_TOKEN_INVALID":
			respondError(c, http.StatusUnauthorized, usecaseErr.Code)
		default:
			if usecaseErr.Err != nil {
				h.logger.Error("Callback ユースケースエラー", slog.String("code", usecaseErr.Code), slog.Any("error", usecaseErr.Err))
			}
			respondError(c, http.StatusInternalServerError, usecaseErr.Code)
		}
		return
	}

	// OAuth フロー用 Cookie をクリアする
	c.SetCookie(stateCookieName, "", -1, "/", h.cookieDomain, h.secureCookie, true)
	c.SetCookie(verifierCookieName, "", -1, "/", h.cookieDomain, h.secureCookie, true)

	// セッション Cookie を発行する
	c.SetSameSite(http.SameSiteLaxMode)
	c.SetCookie(CookieName, out.SessionID, int(h.sessionTTL.Seconds()), "/", h.cookieDomain, h.secureCookie, true)

	// モバイルフロー: 交換コード付きカスタムスキーム URL へリダイレクトする
	if out.MobileRedirectURL != "" {
		// リダイレクト用 Cookie をクリアする
		c.SetCookie(postAuthRedirectCookie, "", -1, "/", h.cookieDomain, h.secureCookie, true)
		c.Redirect(http.StatusFound, out.MobileRedirectURL)
		return
	}

	// ブラウザフロー: 認証完了レスポンスを返す
	c.JSON(http.StatusOK, gin.H{
		"status":     "authenticated",
		"csrf_token": out.CSRFToken,
	})
}

// Logout はセッションを削除し、IdP のログアウトエンドポイントへリダイレクトする。
// AuthUseCase.Logout にビジネスロジックを委譲する。
func (h *AuthHandler) Logout(c *gin.Context) {
	sessionID, err := c.Cookie(CookieName)
	if err != nil {
		sessionID = ""
	}

	// H-6 監査対応: Logout エラーを無視せずログに記録する。
	// ログアウト失敗はユーザー体験に影響しないが（Cookie クリアとリダイレクトは続行する）、
	// 監視・デバッグのために警告ログを出力する。
	out, err := h.authUseCase.Logout(c.Request.Context(), usecase.LogoutInput{
		SessionID:     sessionID,
		PostLogoutURI: h.postLogoutURI,
	})
	if err != nil {
		h.logger.Warn("Logout エラー（ユーザー体験への影響なし）", "error", err)
	}

	// セッション Cookie をクリアする（Logout エラーに関わらず必ずクリアする）
	c.SetCookie(CookieName, "", -1, "/", h.cookieDomain, h.secureCookie, true)

	// out が nil の場合（Logout エラー時）はフォールバック URI にリダイレクトする
	if out == nil {
		if h.postLogoutURI != "" {
			c.Redirect(http.StatusFound, h.postLogoutURI)
		} else {
			c.JSON(http.StatusOK, gin.H{"status": "logged_out"})
		}
		return
	}

	// IdP ログアウト URL が構築できた場合はリダイレクトする
	if out.IDPLogoutURL != "" {
		c.Redirect(http.StatusFound, out.IDPLogoutURL)
		return
	}

	// フォールバック: post-logout URI へリダイレクトする
	if out.FallbackURI != "" {
		c.Redirect(http.StatusFound, out.FallbackURI)
		return
	}

	c.JSON(http.StatusOK, gin.H{"status": "logged_out"})
}

// Session はセッションクッキーを検証し、現在のユーザー情報を返す。
// 有効なセッションがあれば 200 + ユーザー情報、無効なら 401 を返す。
func (h *AuthHandler) Session(c *gin.Context) {
	// セッションクッキーからセッション ID を取得する
	sessionID, err := c.Cookie(CookieName)
	if err != nil || sessionID == "" {
		respondError(c, http.StatusUnauthorized, "BFF_AUTH_SESSION_NOT_FOUND")
		return
	}

	// AuthUseCase.CheckSession にセッション検証を委譲する
	out, err := h.authUseCase.CheckSession(c.Request.Context(), usecase.SessionCheckInput{
		SessionID: sessionID,
	})
	if err != nil {
		usecaseErr, ok := err.(*usecase.AuthUseCaseError)
		if !ok {
			h.logger.Error("Session 予期しないエラー", slog.Any("error", err))
			respondError(c, http.StatusInternalServerError, "BFF_AUTH_SESSION_ERROR")
			return
		}
		switch usecaseErr.Code {
		case "BFF_AUTH_SESSION_NOT_FOUND":
			respondError(c, http.StatusUnauthorized, usecaseErr.Code)
		case "BFF_AUTH_SESSION_EXPIRED":
			respondError(c, http.StatusUnauthorized, usecaseErr.Code)
		default:
			if usecaseErr.Err != nil {
				h.logger.Error("Session ユースケースエラー", slog.String("code", usecaseErr.Code), slog.Any("error", usecaseErr.Err))
			}
			respondError(c, http.StatusInternalServerError, usecaseErr.Code)
		}
		return
	}

	// 有効なセッション情報を返す（roles はフロントエンドの /admin ルート認可に使用する）
	c.JSON(http.StatusOK, gin.H{
		"id":            out.Subject,
		"authenticated": true,
		"csrf_token":    out.CSRFToken,
		"roles":         out.Roles,
	})
}

// Exchange はワンタイム交換コードを検証し、セッションクッキーを発行する。
// モバイルクライアントが OAuth 認証完了後にセッションを確立するために使用する。
func (h *AuthHandler) Exchange(c *gin.Context) {
	// 交換コードを取得する
	code := c.Query("code")
	if code == "" {
		respondBadRequest(c, "BFF_AUTH_EXCHANGE_CODE_MISSING")
		return
	}

	// AuthUseCase.ExchangeCode にビジネスロジックを委譲する
	out, err := h.authUseCase.ExchangeCode(c.Request.Context(), usecase.ExchangeInput{
		Code:       code,
		SessionTTL: h.sessionTTL,
	})
	if err != nil {
		usecaseErr, ok := err.(*usecase.AuthUseCaseError)
		if !ok {
			h.logger.Error("Exchange 予期しないエラー", slog.Any("error", err))
			respondError(c, http.StatusInternalServerError, "BFF_AUTH_EXCHANGE_ERROR")
			return
		}
		switch usecaseErr.Code {
		case "BFF_AUTH_EXCHANGE_CODE_MISSING":
			respondBadRequest(c, usecaseErr.Code)
		case "BFF_AUTH_EXCHANGE_CODE_INVALID", "BFF_AUTH_SESSION_NOT_FOUND":
			respondError(c, http.StatusUnauthorized, usecaseErr.Code)
		default:
			if usecaseErr.Err != nil {
				h.logger.Error("Exchange ユースケースエラー", slog.String("code", usecaseErr.Code), slog.Any("error", usecaseErr.Err))
			}
			respondError(c, http.StatusInternalServerError, usecaseErr.Code)
		}
		return
	}

	// セッションクッキーを発行する（モバイルクライアントの Dio が自動保存する）
	c.SetSameSite(http.SameSiteLaxMode)
	c.SetCookie(CookieName, out.RealSessionID, int(h.sessionTTL.Seconds()), "/", h.cookieDomain, h.secureCookie, true)

	// セッション情報を返す
	c.JSON(http.StatusOK, gin.H{
		"id":            out.Subject,
		"authenticated": true,
		"csrf_token":    out.CSRFToken,
	})
}

// generateRandomString はバイト長 n のランダムな hex エンコード文字列を生成する。
// テストから直接参照されるためこのパッケージに残す。
func generateRandomString(n int) (string, error) {
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}

// sanitizeErrorDescription は IdP から受け取った error_description をサニタイズする（M-8 監査対応）。
// XSS 攻撃防止のために HTML エスケープを適用し、
// ログやレスポンスへの過剰な情報露出を防ぐために 256 文字に切り詰める。
func sanitizeErrorDescription(desc string) string {
	// ルーン境界で安全に 256 文字に切り詰める
	if utf8.RuneCountInString(desc) > 256 {
		runes := []rune(desc)
		desc = string(runes[:256])
	}
	// HTML エスケープにより XSS インジェクションを防止する
	return html.EscapeString(desc)
}
