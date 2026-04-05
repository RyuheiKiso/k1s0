package handler

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"net/url"
	"os"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

// mockOAuthClient は OAuthClient インターフェースのテスト用モック。
// 関数フィールドでメソッドの振る舞いを差し替える。
type mockOAuthClient struct {
	authCodeURLFn         func(state, codeChallenge string) (string, error)
	exchangeCodeFn        func(ctx context.Context, code, codeVerifier string) (*oauth.TokenResponse, error)
	extractClaimsFn       func(ctx context.Context, idToken string) (string, []string, error)
	logoutURLFn           func(idTokenHint, postLogoutRedirectURI string) (string, error)
	discoveryCacheCleared bool
}

// AuthCodeURL は認可コードフローの URL を構築するモック実装。
func (m *mockOAuthClient) AuthCodeURL(state, codeChallenge string) (string, error) {
	return m.authCodeURLFn(state, codeChallenge)
}

// ExchangeCode は認可コードをトークンに交換するモック実装。
func (m *mockOAuthClient) ExchangeCode(ctx context.Context, code, codeVerifier string) (*oauth.TokenResponse, error) {
	return m.exchangeCodeFn(ctx, code, codeVerifier)
}

// ExtractClaims は JWKS 署名検証済み ID トークンから subject と realm roles を返すモック実装。
func (m *mockOAuthClient) ExtractClaims(ctx context.Context, idToken string) (string, []string, error) {
	return m.extractClaimsFn(ctx, idToken)
}

// LogoutURL は IdP のログアウト URL を返すモック実装。
func (m *mockOAuthClient) LogoutURL(idTokenHint, postLogoutRedirectURI string) (string, error) {
	return m.logoutURLFn(idTokenHint, postLogoutRedirectURI)
}

// ClearDiscoveryCache は OIDC discovery キャッシュクリアのモック実装。
// 呼び出されたことを記録する。
func (m *mockOAuthClient) ClearDiscoveryCache() {
	m.discoveryCacheCleared = true
}

// mockSessionStore は session.Store と session.ExchangeCodeStore の両インターフェースのテスト用モック。
// インメモリの map でセッションおよび交換コードを管理する（H-5 監査対応）。
type mockSessionStore struct {
	sessions      map[string]*session.SessionData
	exchangeCodes map[string]*session.ExchangeCodeData
	counter       int
	ecCounter     int
}

// newMockSessionStore はテスト用のセッションストアを生成する。
func newMockSessionStore() *mockSessionStore {
	return &mockSessionStore{
		sessions:      make(map[string]*session.SessionData),
		exchangeCodes: make(map[string]*session.ExchangeCodeData),
	}
}

// Create はセッションデータを保存し、連番の ID を返す。
func (m *mockSessionStore) Create(_ context.Context, data *session.SessionData, _ time.Duration) (string, error) {
	m.counter++
	id := fmt.Sprintf("test-session-id-%d", m.counter)
	m.sessions[id] = data
	return id, nil
}

// Get は指定 ID のセッションデータを取得する。
func (m *mockSessionStore) Get(_ context.Context, id string) (*session.SessionData, error) {
	if s, ok := m.sessions[id]; ok {
		return s, nil
	}
	return nil, nil
}

// Update は指定 ID のセッションデータを更新する。
func (m *mockSessionStore) Update(_ context.Context, id string, data *session.SessionData, _ time.Duration) error {
	m.sessions[id] = data
	return nil
}

// Delete は指定 ID のセッションデータを削除する。
func (m *mockSessionStore) Delete(_ context.Context, id string) error {
	delete(m.sessions, id)
	return nil
}

// Touch はセッション TTL を延長する（スライディング有効期限）。
func (m *mockSessionStore) Touch(_ context.Context, _ string, _ time.Duration) error {
	return nil
}

// CreateExchangeCode は交換コードデータを保存し、連番のコードを返す（H-5 監査対応）。
func (m *mockSessionStore) CreateExchangeCode(_ context.Context, data *session.ExchangeCodeData, _ time.Duration) (string, error) {
	m.ecCounter++
	code := fmt.Sprintf("test-exchange-code-%d", m.ecCounter)
	m.exchangeCodes[code] = data
	return code, nil
}

// GetExchangeCode は交換コードに対応するデータを取得する（H-5 監査対応）。
func (m *mockSessionStore) GetExchangeCode(_ context.Context, code string) (*session.ExchangeCodeData, error) {
	if d, ok := m.exchangeCodes[code]; ok {
		return d, nil
	}
	return nil, nil
}

// DeleteExchangeCode は交換コードを削除する（H-5 監査対応）。
func (m *mockSessionStore) DeleteExchangeCode(_ context.Context, code string) error {
	delete(m.exchangeCodes, code)
	return nil
}

// errOnCreateSessionStore は Create を呼び出すと必ずエラーを返すテスト用セッションストア。
// セッション保存失敗パスの検証に使用する。
type errOnCreateSessionStore struct {
	// err は Create 呼び出し時に返すエラー。
	err error
}

// Create は常にエラーを返してセッション保存失敗をシミュレートする。
func (m *errOnCreateSessionStore) Create(_ context.Context, _ *session.SessionData, _ time.Duration) (string, error) {
	return "", m.err
}

// Get は常に nil を返す（テストで呼ばれることはないが Store インターフェースを満たすために実装する）。
func (m *errOnCreateSessionStore) Get(_ context.Context, _ string) (*session.SessionData, error) {
	return nil, nil
}

// Update はセッション更新の空実装（テストで呼ばれることはない）。
func (m *errOnCreateSessionStore) Update(_ context.Context, _ string, _ *session.SessionData, _ time.Duration) error {
	return nil
}

// Delete はセッション削除の空実装（テストで呼ばれることはない）。
func (m *errOnCreateSessionStore) Delete(_ context.Context, _ string) error {
	return nil
}

// Touch は TTL 延長の空実装（テストで呼ばれることはない）。
func (m *errOnCreateSessionStore) Touch(_ context.Context, _ string, _ time.Duration) error {
	return nil
}

// CreateExchangeCode は交換コード作成の空実装（H-5 監査対応）。
func (m *errOnCreateSessionStore) CreateExchangeCode(_ context.Context, _ *session.ExchangeCodeData, _ time.Duration) (string, error) {
	return "", nil
}

// GetExchangeCode は交換コード取得の空実装（H-5 監査対応）。
func (m *errOnCreateSessionStore) GetExchangeCode(_ context.Context, _ string) (*session.ExchangeCodeData, error) {
	return nil, nil
}

// DeleteExchangeCode は交換コード削除の空実装（H-5 監査対応）。
func (m *errOnCreateSessionStore) DeleteExchangeCode(_ context.Context, _ string) error {
	return nil
}

// testStore は session.Store と session.ExchangeCodeStore を合成したテスト用インターフェース（H-5 監査対応）。
// mockSessionStore と errOnCreateSessionStore の両方がこのインターフェースを実装する。
type testStore interface {
	session.Store
	session.ExchangeCodeStore
}

// newTestAuthHandler はテスト用の AuthHandler を構築するヘルパー。
// H-5 監査対応: exchangeCodeStore 引数を追加し、store を両方に渡す。
// C-09 監査対応: absoluteMaxTTL（第5引数）を追加し、NewAuthHandler の9引数シグネチャに合わせる。
func newTestAuthHandler(oauthClient OAuthClient, store testStore) *AuthHandler {
	return NewAuthHandler(
		oauthClient,
		store,
		store,
		30*time.Minute,
		24*time.Hour,
		60*time.Second, // POLY-003 監査対応: exchangeCodeTTL（デフォルト 60s）
		"https://app.example.com",
		false,
		"",
		slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelError})),
	)
}

// setupTestRouter はテスト用の Gin ルーターを構築するヘルパー。
func setupTestRouter(h *AuthHandler) *gin.Engine {
	gin.SetMode(gin.TestMode)
	r := gin.New()
	r.GET("/auth/login", h.Login)
	r.GET("/auth/callback", h.Callback)
	r.GET("/auth/session", h.Session)
	r.GET("/auth/exchange", h.Exchange)
	r.POST("/auth/logout", h.Logout)
	return r
}

// TestLogin_RedirectsToAuthURL は Login 正常系のテスト。
// 302 リダイレクトが返り、state/verifier cookie が設定されることを検証する。
func TestLogin_RedirectsToAuthURL(t *testing.T) {
	mock := &mockOAuthClient{
		authCodeURLFn: func(state, codeChallenge string) (string, error) {
			return "https://idp.example.com/auth?state=" + state, nil
		},
	}

	h := newTestAuthHandler(mock, newMockSessionStore())
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/login", nil)
	router.ServeHTTP(w, req)

	// 302 リダイレクトであること
	assert.Equal(t, http.StatusFound, w.Code)
	assert.Contains(t, w.Header().Get("Location"), "https://idp.example.com/auth")

	// state と verifier cookie が設定されていることを確認
	cookies := w.Result().Cookies()
	var hasState, hasVerifier bool
	for _, c := range cookies {
		if c.Name == "k1s0_oauth_state" {
			hasState = true
		}
		if c.Name == "k1s0_pkce_verifier" {
			hasVerifier = true
		}
	}
	assert.True(t, hasState, "state cookie should be set")
	assert.True(t, hasVerifier, "verifier cookie should be set")
}

// TestCallback_Success は Callback 正常系のテスト。
// トークン交換 → セッション作成 → 200 レスポンスを検証する。
func TestCallback_Success(t *testing.T) {
	mock := &mockOAuthClient{
		exchangeCodeFn: func(_ context.Context, code, verifier string) (*oauth.TokenResponse, error) {
			return &oauth.TokenResponse{
				AccessToken:  "access-token-123",
				RefreshToken: "refresh-token-456",
				IDToken:      "id-token-789",
				TokenType:    "Bearer",
				ExpiresIn:    3600,
			}, nil
		},
		// ExtractClaims は JWKS 署名検証済み ID トークンから subject と roles を返す
		extractClaimsFn: func(_ context.Context, idToken string) (string, []string, error) {
			return "user-sub-001", []string{"user"}, nil
		},
	}

	store := newMockSessionStore()
	h := newTestAuthHandler(mock, store)
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/callback?state=test-state&code=auth-code-123", nil)
	// state cookie と verifier cookie を設定
	req.AddCookie(&http.Cookie{Name: "k1s0_oauth_state", Value: "test-state"})
	req.AddCookie(&http.Cookie{Name: "k1s0_pkce_verifier", Value: "test-verifier"})
	router.ServeHTTP(w, req)

	// 200 レスポンスであること
	assert.Equal(t, http.StatusOK, w.Code)

	// レスポンスボディに authenticated ステータスが含まれる
	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	assert.Equal(t, "authenticated", body["status"])
	assert.NotEmpty(t, body["csrf_token"])

	// セッションが作成されている
	assert.Len(t, store.sessions, 1)
}

// TestCallback_StateMismatch は Callback 異常系のテスト。
// state パラメータが cookie と一致しない場合に 400 が返ることを検証する。
func TestCallback_StateMismatch(t *testing.T) {
	mock := &mockOAuthClient{}
	h := newTestAuthHandler(mock, newMockSessionStore())
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/callback?state=wrong-state&code=auth-code", nil)
	req.AddCookie(&http.Cookie{Name: "k1s0_oauth_state", Value: "correct-state"})
	req.AddCookie(&http.Cookie{Name: "k1s0_pkce_verifier", Value: "test-verifier"})
	router.ServeHTTP(w, req)

	// 400 エラーであること
	assert.Equal(t, http.StatusBadRequest, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストオブジェクト、コードは ["code"] キーで取得する
	assert.Equal(t, "BFF_AUTH_STATE_MISMATCH", body["error"].(map[string]any)["code"])
}

// TestCallback_CodeMissing は Callback 異常系のテスト。
// code パラメータが欠落している場合に 400 が返ることを検証する。
func TestCallback_CodeMissing(t *testing.T) {
	mock := &mockOAuthClient{}
	h := newTestAuthHandler(mock, newMockSessionStore())
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/callback?state=test-state", nil)
	req.AddCookie(&http.Cookie{Name: "k1s0_oauth_state", Value: "test-state"})
	req.AddCookie(&http.Cookie{Name: "k1s0_pkce_verifier", Value: "test-verifier"})
	router.ServeHTTP(w, req)

	// 400 エラーであること
	assert.Equal(t, http.StatusBadRequest, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストオブジェクト、コードは ["code"] キーで取得する
	assert.Equal(t, "BFF_AUTH_CODE_MISSING", body["error"].(map[string]any)["code"])
}

// TestLogout_WithSession は Logout 正常系のテスト。
// セッションが存在する場合、セッション削除後に IdP ログアウト URL へリダイレクトする。
// また、OIDC discovery キャッシュがクリアされることを検証する。
func TestLogout_WithSession(t *testing.T) {
	mock := &mockOAuthClient{
		logoutURLFn: func(idTokenHint, postLogoutRedirectURI string) (string, error) {
			return "https://idp.example.com/logout?post_logout_redirect_uri=" + postLogoutRedirectURI, nil
		},
	}

	store := newMockSessionStore()
	// セッションを事前に作成
	store.sessions["existing-session"] = &session.SessionData{
		AccessToken: "access-token",
		IDToken:     "id-token-for-logout",
		Subject:     "user-001",
	}

	h := newTestAuthHandler(mock, store)
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodPost, "/auth/logout", nil)
	req.AddCookie(&http.Cookie{Name: "k1s0_session", Value: "existing-session"})
	router.ServeHTTP(w, req)

	// 302 リダイレクトで IdP ログアウト URL へ遷移すること
	assert.Equal(t, http.StatusFound, w.Code)
	assert.Contains(t, w.Header().Get("Location"), "https://idp.example.com/logout")

	// セッションが削除されていること
	assert.Empty(t, store.sessions)

	// OIDC discovery キャッシュがクリアされていること
	assert.True(t, mock.discoveryCacheCleared, "logout 時に discovery キャッシュがクリアされること")
}

// TestLogout_NoSession はセッションなしの Logout テスト。
// セッションがない場合でも discovery キャッシュはクリアされ、postLogoutURI へリダイレクトする。
func TestLogout_NoSession(t *testing.T) {
	mock := &mockOAuthClient{}
	h := newTestAuthHandler(mock, newMockSessionStore())
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodPost, "/auth/logout", nil)
	router.ServeHTTP(w, req)

	// postLogoutURI へのリダイレクトであること
	assert.Equal(t, http.StatusFound, w.Code)
	assert.Equal(t, "https://app.example.com", w.Header().Get("Location"))

	// セッションがなくても discovery キャッシュはクリアされること
	assert.True(t, mock.discoveryCacheCleared, "セッションなしでも discovery キャッシュがクリアされること")
}

// TestSession_Valid は有効なセッションでの Session エンドポイントテスト。
// セッションクッキーが有効な場合に 200 + ユーザー情報を返すことを検証する。
func TestSession_Valid(t *testing.T) {
	mock := &mockOAuthClient{}
	store := newMockSessionStore()
	// 有効なセッションを事前に作成する
	store.sessions["valid-session"] = &session.SessionData{
		AccessToken: "access-token-123",
		Subject:     "user-sub-001",
		CSRFToken:   "csrf-token-abc",
		ExpiresAt:   time.Now().Add(1 * time.Hour).Unix(),
	}

	h := newTestAuthHandler(mock, store)
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/session", nil)
	req.AddCookie(&http.Cookie{Name: "k1s0_session", Value: "valid-session"})
	router.ServeHTTP(w, req)

	// 200 レスポンスであること
	assert.Equal(t, http.StatusOK, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	assert.Equal(t, "user-sub-001", body["id"])
	assert.Equal(t, true, body["authenticated"])
	assert.Equal(t, "csrf-token-abc", body["csrf_token"])
}

// TestSession_NoCookie はセッションクッキーなしの Session テスト。
// セッションクッキーがない場合に 401 を返すことを検証する。
func TestSession_NoCookie(t *testing.T) {
	mock := &mockOAuthClient{}
	h := newTestAuthHandler(mock, newMockSessionStore())
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/session", nil)
	router.ServeHTTP(w, req)

	// 401 エラーであること
	assert.Equal(t, http.StatusUnauthorized, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストオブジェクト、コードは ["code"] キーで取得する
	assert.Equal(t, "BFF_AUTH_SESSION_NOT_FOUND", body["error"].(map[string]any)["code"])
}

// TestSession_InvalidSession は無効なセッション ID での Session テスト。
// セッションストアに存在しない ID の場合に 401 を返すことを検証する。
func TestSession_InvalidSession(t *testing.T) {
	mock := &mockOAuthClient{}
	h := newTestAuthHandler(mock, newMockSessionStore())
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/session", nil)
	req.AddCookie(&http.Cookie{Name: "k1s0_session", Value: "non-existent-session"})
	router.ServeHTTP(w, req)

	// 401 エラーであること
	assert.Equal(t, http.StatusUnauthorized, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストオブジェクト、コードは ["code"] キーで取得する
	assert.Equal(t, "BFF_AUTH_SESSION_NOT_FOUND", body["error"].(map[string]any)["code"])
}

// TestSession_Expired は期限切れセッションでの Session テスト。
// セッションの有効期限が切れている場合に 401 を返すことを検証する。
func TestSession_Expired(t *testing.T) {
	mock := &mockOAuthClient{}
	store := newMockSessionStore()
	// 期限切れのセッションを事前に作成する
	store.sessions["expired-session"] = &session.SessionData{
		AccessToken: "access-token-expired",
		Subject:     "user-sub-002",
		CSRFToken:   "csrf-token-xyz",
		ExpiresAt:   time.Now().Add(-1 * time.Hour).Unix(),
	}

	h := newTestAuthHandler(mock, store)
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/session", nil)
	req.AddCookie(&http.Cookie{Name: "k1s0_session", Value: "expired-session"})
	router.ServeHTTP(w, req)

	// 401 エラーであること
	assert.Equal(t, http.StatusUnauthorized, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストオブジェクト、コードは ["code"] キーで取得する
	assert.Equal(t, "BFF_AUTH_SESSION_EXPIRED", body["error"].(map[string]any)["code"])
}

// TestLogin_WithMobileRedirect はモバイルリダイレクトパラメータ付きの Login テスト。
// カスタムスキームの redirect_to が Cookie に保存されることを検証する。
func TestLogin_WithMobileRedirect(t *testing.T) {
	mock := &mockOAuthClient{
		authCodeURLFn: func(state, codeChallenge string) (string, error) {
			return "https://idp.example.com/auth?state=" + state, nil
		},
	}

	h := newTestAuthHandler(mock, newMockSessionStore())
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/login?redirect_to=k1s0://auth/callback", nil)
	router.ServeHTTP(w, req)

	// 302 リダイレクトであること
	assert.Equal(t, http.StatusFound, w.Code)

	// redirect_to の Cookie が設定されていること（URL エンコードされる場合がある）
	cookies := w.Result().Cookies()
	var hasRedirect bool
	for _, c := range cookies {
		if c.Name == "k1s0_post_auth_redirect" {
			hasRedirect = true
			decoded, _ := url.QueryUnescape(c.Value)
			assert.Equal(t, "k1s0://auth/callback", decoded)
		}
	}
	assert.True(t, hasRedirect, "post_auth_redirect cookie should be set")
}

// TestLogin_RejectsHTTPRedirect は http/https スキームの redirect_to を拒否するテスト。
// オープンリダイレクト攻撃を防止する。
func TestLogin_RejectsHTTPRedirect(t *testing.T) {
	mock := &mockOAuthClient{
		authCodeURLFn: func(state, codeChallenge string) (string, error) {
			return "https://idp.example.com/auth?state=" + state, nil
		},
	}

	h := newTestAuthHandler(mock, newMockSessionStore())
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/login?redirect_to=https://evil.com/steal", nil)
	router.ServeHTTP(w, req)

	// リダイレクトは返るが、redirect cookie は設定されないこと
	cookies := w.Result().Cookies()
	for _, c := range cookies {
		assert.NotEqual(t, "k1s0_post_auth_redirect", c.Name, "http/https redirect should be rejected")
	}
}

// TestLogin_RejectsDangerousSchemes は javascript/data 等の危険なスキームを拒否するテスト。
func TestLogin_RejectsDangerousSchemes(t *testing.T) {
	mock := &mockOAuthClient{
		authCodeURLFn: func(state, codeChallenge string) (string, error) {
			return "https://idp.example.com/auth?state=" + state, nil
		},
	}

	dangerousURLs := []string{
		"javascript:alert(1)",
		"data:text/html,<script>alert(1)</script>",
		"vbscript:msgbox",
		"file:///etc/passwd",
	}

	for _, dangerous := range dangerousURLs {
		t.Run(dangerous, func(t *testing.T) {
			h := newTestAuthHandler(mock, newMockSessionStore())
			router := setupTestRouter(h)

			w := httptest.NewRecorder()
			req := httptest.NewRequest(http.MethodGet, "/auth/login?redirect_to="+url.QueryEscape(dangerous), nil)
			router.ServeHTTP(w, req)

			cookies := w.Result().Cookies()
			for _, c := range cookies {
				assert.NotEqual(t, "k1s0_post_auth_redirect", c.Name,
					"dangerous scheme should be rejected: "+dangerous)
			}
		})
	}
}

// TestLogin_RejectsArbitraryCustomScheme は k1s0 以外の任意カスタムスキームを拒否するテスト。
// allowlist が denylist ではなく許可リスト方式であることを確認する（Finding 6）。
func TestLogin_RejectsArbitraryCustomScheme(t *testing.T) {
	mock := &mockOAuthClient{
		authCodeURLFn: func(state, codeChallenge string) (string, error) {
			return "https://idp.example.com/auth?state=" + state, nil
		},
	}

	// k1s0 以外の任意カスタムスキームはすべて拒否されること
	arbitrarySchemes := []string{
		"evilapp://callback",
		"myapp://open",
		"com.example.app://auth",
		"k1s0evil://callback", // k1s0 プレフィックスを含む偽スキームも拒否
	}

	for _, rawURL := range arbitrarySchemes {
		t.Run(rawURL, func(t *testing.T) {
			h := newTestAuthHandler(mock, newMockSessionStore())
			router := setupTestRouter(h)

			w := httptest.NewRecorder()
			req := httptest.NewRequest(http.MethodGet, "/auth/login?redirect_to="+url.QueryEscape(rawURL), nil)
			router.ServeHTTP(w, req)

			cookies := w.Result().Cookies()
			for _, c := range cookies {
				assert.NotEqual(t, "k1s0_post_auth_redirect", c.Name,
					"arbitrary custom scheme should be rejected: "+rawURL)
			}
		})
	}
}

// TestCallback_MobileRedirect はモバイルリダイレクト付きの Callback テスト。
// 認証成功後にカスタムスキームへ交換コード付きでリダイレクトされることを検証する。
func TestCallback_MobileRedirect(t *testing.T) {
	mock := &mockOAuthClient{
		exchangeCodeFn: func(_ context.Context, code, verifier string) (*oauth.TokenResponse, error) {
			return &oauth.TokenResponse{
				AccessToken:  "access-token-mobile",
				RefreshToken: "refresh-token-mobile",
				IDToken:      "id-token-mobile",
				TokenType:    "Bearer",
				ExpiresIn:    3600,
			}, nil
		},
		// ExtractClaims は JWKS 署名検証済み ID トークンから subject と roles を返す
		extractClaimsFn: func(_ context.Context, idToken string) (string, []string, error) {
			return "mobile-user-001", []string{"user"}, nil
		},
	}

	store := newMockSessionStore()
	h := newTestAuthHandler(mock, store)
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/callback?state=test-state&code=auth-code-mobile", nil)
	req.AddCookie(&http.Cookie{Name: "k1s0_oauth_state", Value: "test-state"})
	req.AddCookie(&http.Cookie{Name: "k1s0_pkce_verifier", Value: "test-verifier"})
	req.AddCookie(&http.Cookie{Name: "k1s0_post_auth_redirect", Value: "k1s0://auth/callback"})
	router.ServeHTTP(w, req)

	// 302 リダイレクトであること
	assert.Equal(t, http.StatusFound, w.Code)

	// リダイレクト先がカスタムスキームで、交換コードが含まれていること
	location := w.Header().Get("Location")
	assert.Contains(t, location, "k1s0://auth/callback")
	assert.Contains(t, location, "code=")

	// 実セッションが作成されていること（H-5 監査対応: 交換コードは sessions ではなく exchangeCodes に格納）
	assert.GreaterOrEqual(t, len(store.sessions), 1, "should have real session")
	// 交換コードが exchangeCodes マップに作成されていること（H-5 監査対応）
	assert.GreaterOrEqual(t, len(store.exchangeCodes), 1, "should have exchange code entry")
}

// TestExchange_Valid は有効な交換コードでの Exchange テスト。
// 交換コードでセッションクッキーが発行されることを検証する。
// H-5 監査対応: exchangeCodes マップに ExchangeCodeData を格納する。
func TestExchange_Valid(t *testing.T) {
	mock := &mockOAuthClient{}
	store := newMockSessionStore()

	// 実セッションを事前作成する
	store.sessions["real-session-id"] = &session.SessionData{
		AccessToken: "access-token-real",
		Subject:     "user-sub-exchange",
		CSRFToken:   "csrf-exchange-123",
		ExpiresAt:   time.Now().Add(1 * time.Hour).Unix(),
	}
	// 交換コード用エントリを ExchangeCodeData として登録する（H-5 監査対応）
	store.exchangeCodes["exchange-code-id"] = &session.ExchangeCodeData{
		SessionID: "real-session-id",
		ExpiresAt: time.Now().Add(60 * time.Second).Unix(),
	}

	h := newTestAuthHandler(mock, store)
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/exchange?code=exchange-code-id", nil)
	router.ServeHTTP(w, req)

	// 200 レスポンスであること
	assert.Equal(t, http.StatusOK, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	assert.Equal(t, "user-sub-exchange", body["id"])
	assert.Equal(t, true, body["authenticated"])
	assert.Equal(t, "csrf-exchange-123", body["csrf_token"])

	// セッションクッキーが設定されていること
	cookies := w.Result().Cookies()
	var hasSession bool
	for _, c := range cookies {
		if c.Name == "k1s0_session" {
			hasSession = true
			assert.Equal(t, "real-session-id", c.Value)
		}
	}
	assert.True(t, hasSession, "session cookie should be set")

	// 交換コードが削除されていること（ワンタイム使用）（H-5 監査対応）
	_, exists := store.exchangeCodes["exchange-code-id"]
	assert.False(t, exists, "exchange code should be deleted after use")
}

// TestExchange_InvalidCode は無効な交換コードでの Exchange テスト。
func TestExchange_InvalidCode(t *testing.T) {
	mock := &mockOAuthClient{}
	h := newTestAuthHandler(mock, newMockSessionStore())
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/exchange?code=non-existent", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストされたオブジェクトのためコードフィールドを参照する
	assert.Equal(t, "BFF_AUTH_EXCHANGE_CODE_INVALID", body["error"].(map[string]any)["code"])
}

// TestExchange_MissingCode は交換コード未指定での Exchange テスト。
func TestExchange_MissingCode(t *testing.T) {
	mock := &mockOAuthClient{}
	h := newTestAuthHandler(mock, newMockSessionStore())
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/exchange", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusBadRequest, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストされたオブジェクトのためコードフィールドを参照する
	assert.Equal(t, "BFF_AUTH_EXCHANGE_CODE_MISSING", body["error"].(map[string]any)["code"])
}

// TestExchange_ExpiredCode は期限切れ交換コードでの Exchange テスト。
// H-5 監査対応: exchangeCodes マップに期限切れの ExchangeCodeData を格納する。
func TestExchange_ExpiredCode(t *testing.T) {
	mock := &mockOAuthClient{}
	store := newMockSessionStore()

	// 期限切れの交換コード用エントリを ExchangeCodeData として作成する（H-5 監査対応）
	store.exchangeCodes["expired-exchange"] = &session.ExchangeCodeData{
		SessionID: "some-session-id",
		ExpiresAt: time.Now().Add(-1 * time.Minute).Unix(),
	}

	h := newTestAuthHandler(mock, store)
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/exchange?code=expired-exchange", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストされたオブジェクトのためコードフィールドを参照する
	assert.Equal(t, "BFF_AUTH_EXCHANGE_CODE_INVALID", body["error"].(map[string]any)["code"])
}

// TestCallback_TokenExchangeFailed は Callback 異常系のテスト。
// IdP への認可コード交換（ExchangeCode）が失敗した場合に 500 が返ることを検証する。
// IdP が一時的に利用不能な場合や、コードが既に使用済みの場合にこのケースが発生する。
func TestCallback_TokenExchangeFailed(t *testing.T) {
	mock := &mockOAuthClient{
		// ExchangeCode は IdP 側の障害などでエラーを返す
		exchangeCodeFn: func(_ context.Context, code, verifier string) (*oauth.TokenResponse, error) {
			return nil, errors.New("idp connection refused")
		},
	}

	store := newMockSessionStore()
	h := newTestAuthHandler(mock, store)
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/callback?state=test-state&code=auth-code-123", nil)
	req.AddCookie(&http.Cookie{Name: "k1s0_oauth_state", Value: "test-state"})
	req.AddCookie(&http.Cookie{Name: "k1s0_pkce_verifier", Value: "test-verifier"})
	router.ServeHTTP(w, req)

	// ExchangeCode 失敗時は 500 が返ること
	assert.Equal(t, http.StatusInternalServerError, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストオブジェクト
	assert.Equal(t, "BFF_AUTH_TOKEN_EXCHANGE_FAILED", body["error"].(map[string]any)["code"])

	// セッションは作成されていないこと
	assert.Empty(t, store.sessions)
}

// TestCallback_IDTokenInvalid は Callback 異常系のテスト。
// ID トークンの検証（ExtractClaims）が失敗した場合に 401 が返ることを検証する。
// JWKS 署名が不正または ID トークンの有効期限切れの場合にこのケースが発生する。
func TestCallback_IDTokenInvalid(t *testing.T) {
	mock := &mockOAuthClient{
		// ExchangeCode は成功するが、ID トークンが不正で署名検証に失敗する
		exchangeCodeFn: func(_ context.Context, code, verifier string) (*oauth.TokenResponse, error) {
			return &oauth.TokenResponse{
				AccessToken:  "access-token-123",
				RefreshToken: "refresh-token-456",
				IDToken:      "invalid-id-token",
				TokenType:    "Bearer",
				ExpiresIn:    3600,
			}, nil
		},
		// ExtractClaims は ID トークンの署名検証失敗をシミュレートする
		extractClaimsFn: func(_ context.Context, idToken string) (string, []string, error) {
			return "", nil, errors.New("jwks: signature verification failed")
		},
	}

	store := newMockSessionStore()
	h := newTestAuthHandler(mock, store)
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/callback?state=test-state&code=auth-code-123", nil)
	req.AddCookie(&http.Cookie{Name: "k1s0_oauth_state", Value: "test-state"})
	req.AddCookie(&http.Cookie{Name: "k1s0_pkce_verifier", Value: "test-verifier"})
	router.ServeHTTP(w, req)

	// ID トークン不正時は 401 が返ること
	assert.Equal(t, http.StatusUnauthorized, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストオブジェクト
	assert.Equal(t, "BFF_AUTH_ID_TOKEN_INVALID", body["error"].(map[string]any)["code"])

	// セッションは作成されていないこと
	assert.Empty(t, store.sessions)
}

// TestCallback_SessionCreateFailed は Callback 異常系のテスト。
// セッションストアへの保存（Create）が失敗した場合に 500 が返ることを検証する。
// Redis 障害などでセッション永続化できない場合にこのケースが発生する。
func TestCallback_SessionCreateFailed(t *testing.T) {
	mock := &mockOAuthClient{
		// トークン交換と ID トークン検証は成功する
		exchangeCodeFn: func(_ context.Context, code, verifier string) (*oauth.TokenResponse, error) {
			return &oauth.TokenResponse{
				AccessToken:  "access-token-123",
				RefreshToken: "refresh-token-456",
				IDToken:      "id-token-789",
				TokenType:    "Bearer",
				ExpiresIn:    3600,
			}, nil
		},
		extractClaimsFn: func(_ context.Context, idToken string) (string, []string, error) {
			return "user-sub-001", []string{"user"}, nil
		},
	}

	// セッション保存が常に失敗するストアを使用する
	failStore := &errOnCreateSessionStore{
		err: errors.New("redis: connection refused"),
	}
	h := newTestAuthHandler(mock, failStore)
	router := setupTestRouter(h)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/auth/callback?state=test-state&code=auth-code-123", nil)
	req.AddCookie(&http.Cookie{Name: "k1s0_oauth_state", Value: "test-state"})
	req.AddCookie(&http.Cookie{Name: "k1s0_pkce_verifier", Value: "test-verifier"})
	router.ServeHTTP(w, req)

	// セッション保存失敗時は 500 が返ること
	assert.Equal(t, http.StatusInternalServerError, w.Code)

	var body map[string]any
	err := json.NewDecoder(w.Body).Decode(&body)
	require.NoError(t, err)
	// ADR-0005形式: body["error"] はネストオブジェクト
	assert.Equal(t, "BFF_AUTH_SESSION_CREATE_FAILED", body["error"].(map[string]any)["code"])
}
