package handler

import (
	"crypto/rand"
	"encoding/hex"
	"log/slog"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"

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
)

// AuthHandler handles the OAuth2/OIDC browser flow.
type AuthHandler struct {
	oauthClient     *oauth.Client
	sessionStore    session.Store
	sessionTTL      time.Duration
	postLogoutURI   string
	secureCookie    bool
	logger          *slog.Logger
}

// NewAuthHandler creates a new AuthHandler.
func NewAuthHandler(
	oauthClient *oauth.Client,
	sessionStore session.Store,
	sessionTTL time.Duration,
	postLogoutURI string,
	secureCookie bool,
	logger *slog.Logger,
) *AuthHandler {
	return &AuthHandler{
		oauthClient:  oauthClient,
		sessionStore: sessionStore,
		sessionTTL:   sessionTTL,
		postLogoutURI: postLogoutURI,
		secureCookie: secureCookie,
		logger:       logger,
	}
}

// Login initiates the OIDC authorization code flow with PKCE.
func (h *AuthHandler) Login(c *gin.Context) {
	pkce, err := oauth.NewPKCE()
	if err != nil {
		h.logger.Error("failed to generate PKCE", slog.String("error", err.Error()))
		c.JSON(http.StatusInternalServerError, gin.H{"error": "BFF_AUTH_PKCE_ERROR"})
		return
	}

	state, err := generateRandomString(32)
	if err != nil {
		h.logger.Error("failed to generate state", slog.String("error", err.Error()))
		c.JSON(http.StatusInternalServerError, gin.H{"error": "BFF_AUTH_STATE_ERROR"})
		return
	}

	authURL, err := h.oauthClient.AuthCodeURL(state, pkce.CodeChallenge)
	if err != nil {
		h.logger.Error("failed to build auth URL", slog.String("error", err.Error()))
		c.JSON(http.StatusInternalServerError, gin.H{"error": "BFF_AUTH_URL_ERROR"})
		return
	}

	// Store state and verifier in short-lived cookies.
	maxAge := 300 // 5 minutes
	c.SetSameSite(http.SameSiteLaxMode)
	c.SetCookie(stateCookieName, state, maxAge, "/", "", h.secureCookie, true)
	c.SetCookie(verifierCookieName, pkce.CodeVerifier, maxAge, "/", "", h.secureCookie, true)

	c.Redirect(http.StatusFound, authURL)
}

// Callback handles the OIDC callback, exchanges the code, and creates a session.
func (h *AuthHandler) Callback(c *gin.Context) {
	// Verify state parameter.
	state, err := c.Cookie(stateCookieName)
	if err != nil || state == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "BFF_AUTH_STATE_MISSING"})
		return
	}

	queryState := c.Query("state")
	if queryState != state {
		c.JSON(http.StatusBadRequest, gin.H{"error": "BFF_AUTH_STATE_MISMATCH"})
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
		})
		return
	}

	code := c.Query("code")
	if code == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "BFF_AUTH_CODE_MISSING"})
		return
	}

	// Retrieve PKCE verifier.
	verifier, err := c.Cookie(verifierCookieName)
	if err != nil || verifier == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "BFF_AUTH_VERIFIER_MISSING"})
		return
	}

	// Exchange authorization code for tokens.
	tokenResp, err := h.oauthClient.ExchangeCode(c.Request.Context(), code, verifier)
	if err != nil {
		h.logger.Error("token exchange failed", slog.String("error", err.Error()))
		c.JSON(http.StatusInternalServerError, gin.H{"error": "BFF_AUTH_TOKEN_EXCHANGE_FAILED"})
		return
	}

	// Generate CSRF token for the session.
	csrfToken, err := generateRandomString(32)
	if err != nil {
		h.logger.Error("failed to generate CSRF token", slog.String("error", err.Error()))
		c.JSON(http.StatusInternalServerError, gin.H{"error": "BFF_AUTH_CSRF_ERROR"})
		return
	}

	// Create session.
	sessData := &session.SessionData{
		AccessToken:  tokenResp.AccessToken,
		RefreshToken: tokenResp.RefreshToken,
		IDToken:      tokenResp.IDToken,
		ExpiresAt:    time.Now().Add(time.Duration(tokenResp.ExpiresIn) * time.Second).Unix(),
		CSRFToken:    csrfToken,
	}

	sessionID, err := h.sessionStore.Create(c.Request.Context(), sessData, h.sessionTTL)
	if err != nil {
		h.logger.Error("failed to create session", slog.String("error", err.Error()))
		c.JSON(http.StatusInternalServerError, gin.H{"error": "BFF_AUTH_SESSION_CREATE_FAILED"})
		return
	}

	// Clear OAuth flow cookies.
	c.SetCookie(stateCookieName, "", -1, "/", "", h.secureCookie, true)
	c.SetCookie(verifierCookieName, "", -1, "/", "", h.secureCookie, true)

	// Set session cookie.
	c.SetSameSite(http.SameSiteLaxMode)
	c.SetCookie(CookieName, sessionID, int(h.sessionTTL.Seconds()), "/", "", h.secureCookie, true)

	c.JSON(http.StatusOK, gin.H{
		"status":     "authenticated",
		"csrf_token": csrfToken,
	})
}

// Logout destroys the session and redirects to the IdP logout endpoint.
func (h *AuthHandler) Logout(c *gin.Context) {
	sessionID, err := c.Cookie(CookieName)
	if err == nil && sessionID != "" {
		sess, _ := h.sessionStore.Get(c.Request.Context(), sessionID)

		// Delete session from store.
		_ = h.sessionStore.Delete(c.Request.Context(), sessionID)

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

// generateRandomString generates a hex-encoded random string of the given byte length.
func generateRandomString(n int) (string, error) {
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}
