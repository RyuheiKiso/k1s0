package handler

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"log/slog"
	"net/http"
	"net/url"
	"time"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
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

	// exchangeCodeTTL はワンタイム交換コードの有効期間（60秒）。
	exchangeCodeTTL = 60 * time.Second
)

// OAuthClient は OAuth2/OIDC プロバイダー操作のインターフェース。
// テスト時にモック差し替えを可能にする。
type OAuthClient interface {
	// AuthCodeURL は認可コードフローの URL を構築する。
	AuthCodeURL(state, codeChallenge string) (string, error)
	// ExchangeCode は認可コードをトークンに交換する。
	ExchangeCode(ctx context.Context, code, codeVerifier string) (*oauth.TokenResponse, error)
	// ExtractSubject は ID トークンから subject を抽出する。
	ExtractSubject(ctx context.Context, idToken string) (string, error)
	// LogoutURL は IdP のログアウト URL を返す。
	LogoutURL(idTokenHint, postLogoutRedirectURI string) (string, error)
	// ClearDiscoveryCache はキャッシュ済みの OIDC discovery 結果をクリアする。
	// ログアウト時に呼び出し、次回ログインで最新のプロバイダ情報を再取得させる。
	ClearDiscoveryCache()
}

// AuthHandler handles the OAuth2/OIDC browser flow.
type AuthHandler struct {
	oauthClient   OAuthClient
	sessionStore  session.Store
	sessionTTL    time.Duration
	postLogoutURI string
	secureCookie  bool
	logger        *slog.Logger
}

// NewAuthHandler creates a new AuthHandler.
func NewAuthHandler(
	oauthClient OAuthClient,
	sessionStore session.Store,
	sessionTTL time.Duration,
	postLogoutURI string,
	secureCookie bool,
	logger *slog.Logger,
) *AuthHandler {
	return &AuthHandler{
		oauthClient:   oauthClient,
		sessionStore:  sessionStore,
		sessionTTL:    sessionTTL,
		postLogoutURI: postLogoutURI,
		secureCookie:  secureCookie,
		logger:        logger,
	}
}

// Login initiates the OIDC authorization code flow with PKCE.
func (h *AuthHandler) Login(c *gin.Context) {
	pkce, err := oauth.NewPKCE()
	if err != nil {
		h.logger.Error("failed to generate PKCE", slog.String("error", err.Error()))
		respondError(c, http.StatusInternalServerError, "BFF_AUTH_PKCE_ERROR")
		return
	}

	state, err := generateRandomString(32)
	if err != nil {
		h.logger.Error("failed to generate state", slog.String("error", err.Error()))
		respondError(c, http.StatusInternalServerError, "BFF_AUTH_STATE_ERROR")
		return
	}

	authURL, err := h.oauthClient.AuthCodeURL(state, pkce.CodeChallenge)
	if err != nil {
		h.logger.Error("failed to build auth URL", slog.String("error", err.Error()))
		respondError(c, http.StatusInternalServerError, "BFF_AUTH_URL_ERROR")
		return
	}

	// Store state and verifier in short-lived cookies.
	maxAge := 300 // 5 minutes
	c.SetSameSite(http.SameSiteLaxMode)
	c.SetCookie(stateCookieName, state, maxAge, "/", "", h.secureCookie, true)
	c.SetCookie(verifierCookieName, pkce.CodeVerifier, maxAge, "/", "", h.secureCookie, true)

	// モバイルクライアント向け: redirect_to パラメータがあれば認証後のリダイレクト先を保存する
	// セキュリティ: カスタムスキームのみ許可し、危険なスキームを明示的に拒否する
	if redirectTo := c.Query("redirect_to"); redirectTo != "" {
		if isAllowedRedirectScheme(redirectTo) {
			c.SetCookie(postAuthRedirectCookie, redirectTo, maxAge, "/", "", h.secureCookie, true)
		}
	}

	c.Redirect(http.StatusFound, authURL)
}

// Callback handles the OIDC callback, exchanges the code, and creates a session.
func (h *AuthHandler) Callback(c *gin.Context) {
	// Verify state parameter.
	state, err := c.Cookie(stateCookieName)
	if err != nil || state == "" {
		respondBadRequest(c, "BFF_AUTH_STATE_MISSING")
		return
	}

	queryState := c.Query("state")
	if queryState != state {
		respondBadRequest(c, "BFF_AUTH_STATE_MISMATCH")
		return
	}

	// Check for error from IdP.
	if errCode := c.Query("error"); errCode != "" {
		h.logger.Warn("OIDC callback error",
			slog.String("error", errCode),
			slog.String("description", c.Query("error_description")),
		)
		c.JSON(http.StatusBadRequest, gin.H{
			"error":       "BFF_AUTH_IDP_ERROR",
			"description": c.Query("error_description"),
			"request_id":  middleware.GetRequestID(c),
		})
		return
	}

	code := c.Query("code")
	if code == "" {
		respondBadRequest(c, "BFF_AUTH_CODE_MISSING")
		return
	}

	// Retrieve PKCE verifier.
	verifier, err := c.Cookie(verifierCookieName)
	if err != nil || verifier == "" {
		respondBadRequest(c, "BFF_AUTH_VERIFIER_MISSING")
		return
	}

	// Exchange authorization code for tokens.
	tokenResp, err := h.oauthClient.ExchangeCode(c.Request.Context(), code, verifier)
	if err != nil {
		h.logger.Error("token exchange failed", slog.String("error", err.Error()))
		respondError(c, http.StatusInternalServerError, "BFF_AUTH_TOKEN_EXCHANGE_FAILED")
		return
	}

	// JWKSによる署名検証付きでsubjectを取得する
	subject, err := h.oauthClient.ExtractSubject(c.Request.Context(), tokenResp.IDToken)
	if err != nil {
		h.logger.Error("failed to extract subject from id_token", slog.String("error", err.Error()))
		respondError(c, http.StatusUnauthorized, "BFF_AUTH_ID_TOKEN_INVALID")
		return
	}

	// Generate CSRF token for the session.
	csrfToken, err := generateRandomString(32)
	if err != nil {
		h.logger.Error("failed to generate CSRF token", slog.String("error", err.Error()))
		respondError(c, http.StatusInternalServerError, "BFF_AUTH_CSRF_ERROR")
		return
	}

	// Create session.
	sessData := &session.SessionData{
		AccessToken:  tokenResp.AccessToken,
		RefreshToken: tokenResp.RefreshToken,
		IDToken:      tokenResp.IDToken,
		ExpiresAt:    time.Now().Add(time.Duration(tokenResp.ExpiresIn) * time.Second).Unix(),
		CSRFToken:    csrfToken,
		Subject:      subject,
	}

	sessionID, err := h.sessionStore.Create(c.Request.Context(), sessData, h.sessionTTL)
	if err != nil {
		h.logger.Error("failed to create session", slog.String("error", err.Error()))
		respondError(c, http.StatusInternalServerError, "BFF_AUTH_SESSION_CREATE_FAILED")
		return
	}

	// Clear OAuth flow cookies.
	c.SetCookie(stateCookieName, "", -1, "/", "", h.secureCookie, true)
	c.SetCookie(verifierCookieName, "", -1, "/", "", h.secureCookie, true)

	// Set session cookie.
	c.SetSameSite(http.SameSiteLaxMode)
	c.SetCookie(CookieName, sessionID, int(h.sessionTTL.Seconds()), "/", "", h.secureCookie, true)

	// モバイルクライアント向け: 認証後リダイレクト先が設定されている場合はワンタイム交換コードを発行してリダイレクトする
	postAuthRedirect, redirectErr := c.Cookie(postAuthRedirectCookie)
	if redirectErr == nil && postAuthRedirect != "" {
		// リダイレクト用 Cookie をクリアする
		c.SetCookie(postAuthRedirectCookie, "", -1, "/", "", h.secureCookie, true)

		// ワンタイム交換コード: セッション ID への参照を短命なエントリとして保存する
		exchangeData := &session.SessionData{
			AccessToken: sessionID,
			ExpiresAt:   time.Now().Add(exchangeCodeTTL).Unix(),
		}
		exchangeCode, err := h.sessionStore.Create(c.Request.Context(), exchangeData, exchangeCodeTTL)
		if err != nil {
			h.logger.Error("failed to create exchange code", slog.String("error", err.Error()))
			respondError(c, http.StatusInternalServerError, "BFF_AUTH_EXCHANGE_CREATE_FAILED")
			return
		}

		// リダイレクト先 URL に交換コードを付与する
		redirectURL, _ := url.Parse(postAuthRedirect)
		q := redirectURL.Query()
		q.Set("code", exchangeCode)
		redirectURL.RawQuery = q.Encode()
		c.Redirect(http.StatusFound, redirectURL.String())
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"status":     "authenticated",
		"csrf_token": csrfToken,
	})
}

// Logout destroys the session and redirects to the IdP logout endpoint.
func (h *AuthHandler) Logout(c *gin.Context) {
	// ログアウト時にOIDC discoveryキャッシュをクリアし、次回ログインで最新のプロバイダ情報を再取得させる
	h.oauthClient.ClearDiscoveryCache()

	sessionID, err := c.Cookie(CookieName)
	if err == nil && sessionID != "" {
		sess, _ := h.sessionStore.Get(c.Request.Context(), sessionID)

		// セッションストアからセッションを削除する
		if err := h.sessionStore.Delete(c.Request.Context(), sessionID); err != nil {
			h.logger.Warn("セッション削除に失敗", slog.String("session_id", sessionID), slog.String("error", err.Error()))
		}

		// Clear session cookie.
		c.SetCookie(CookieName, "", -1, "/", "", h.secureCookie, true)

		// Build IdP logout URL with id_token_hint if available.
		if sess != nil && sess.IDToken != "" {
			logoutURL, err := h.oauthClient.LogoutURL(sess.IDToken, h.postLogoutURI)
			if err == nil {
				c.Redirect(http.StatusFound, logoutURL)
				return
			}
		}
	}

	// Fallback: redirect to post-logout URI.
	if h.postLogoutURI != "" {
		c.Redirect(http.StatusFound, h.postLogoutURI)
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

	// セッションストアからセッションデータを取得する
	sess, err := h.sessionStore.Get(c.Request.Context(), sessionID)
	if err != nil {
		h.logger.Error("failed to get session", slog.String("error", err.Error()))
		respondError(c, http.StatusInternalServerError, "BFF_AUTH_SESSION_ERROR")
		return
	}

	// セッションが存在しない場合は 401 を返す
	if sess == nil {
		respondError(c, http.StatusUnauthorized, "BFF_AUTH_SESSION_NOT_FOUND")
		return
	}

	// アクセストークンの有効期限が切れている場合は 401 を返す
	if sess.IsExpired() {
		respondError(c, http.StatusUnauthorized, "BFF_AUTH_SESSION_EXPIRED")
		return
	}

	// 有効なセッション情報を返す
	c.JSON(http.StatusOK, gin.H{
		"id":            sess.Subject,
		"authenticated": true,
		"csrf_token":    sess.CSRFToken,
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

	// 交換コードに対応するエントリをセッションストアから取得する
	exchangeData, err := h.sessionStore.Get(c.Request.Context(), code)
	if err != nil {
		h.logger.Error("failed to get exchange code", slog.String("error", err.Error()))
		respondError(c, http.StatusInternalServerError, "BFF_AUTH_EXCHANGE_ERROR")
		return
	}

	// 交換コードが存在しないか期限切れの場合は 401 を返す
	if exchangeData == nil || exchangeData.IsExpired() {
		respondError(c, http.StatusUnauthorized, "BFF_AUTH_EXCHANGE_CODE_INVALID")
		return
	}

	// 実際のセッション ID を取得する（AccessToken フィールドに格納されている）
	realSessionID := exchangeData.AccessToken

	// 実際のセッションがまだ有効か確認する（交換コード削除より前に検証し、
	// セッション無効時に交換コードが消費されてしまう問題を防ぐ）
	realSession, err := h.sessionStore.Get(c.Request.Context(), realSessionID)
	if err != nil || realSession == nil {
		respondError(c, http.StatusUnauthorized, "BFF_AUTH_SESSION_NOT_FOUND")
		return
	}

	// 全検証通過後に交換コードを削除する（ワンタイム使用）
	if err := h.sessionStore.Delete(c.Request.Context(), code); err != nil {
		h.logger.Warn("交換コード削除に失敗", slog.String("code", code), slog.String("error", err.Error()))
	}

	// セッションクッキーを発行する（モバイルクライアントの Dio が自動保存する）
	c.SetSameSite(http.SameSiteLaxMode)
	c.SetCookie(CookieName, realSessionID, int(h.sessionTTL.Seconds()), "/", "", h.secureCookie, true)

	// セッション情報を返す
	c.JSON(http.StatusOK, gin.H{
		"id":            realSession.Subject,
		"authenticated": true,
		"csrf_token":    realSession.CSRFToken,
	})
}

// isAllowedRedirectScheme はモバイルリダイレクト先 URL のスキームを検証する。
// オープンリダイレクト攻撃と XSS を防止するため、以下を許可しない:
//   - http / https（オープンリダイレクト攻撃）
//   - javascript / data / vbscript（XSS 攻撃）
//   - 空スキームまたはパース不可能な URL
//
// アプリ固有のカスタムスキーム（例: k1s0://, myapp://）のみ許可する。
func isAllowedRedirectScheme(rawURL string) bool {
	parsedURL, err := url.Parse(rawURL)
	if err != nil || parsedURL.Scheme == "" {
		return false
	}

	// 危険なスキームを明示的にブロックする
	switch parsedURL.Scheme {
	case "http", "https", "javascript", "data", "vbscript", "file":
		return false
	}

	// Host が空でないことを確認する（スキーム://host の形式であること）
	if parsedURL.Host == "" {
		return false
	}

	return true
}

// generateRandomString generates a hex-encoded random string of the given byte length.
func generateRandomString(n int) (string, error) {
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}

