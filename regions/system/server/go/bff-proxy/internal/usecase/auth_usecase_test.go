// auth_usecase_test.go は AuthUseCase のユニットテストを提供する。
// 外部依存（OAuth プロバイダー、Redis）をモックに差し替えて、
// Login・Callback・Logout・CheckSession・ExchangeCode の各フローを検証する。
package usecase

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/testutil"
)

// ── モック定義 ────────────────────────────────────────────────

// mockAuthOAuthClient は AuthOAuthClient インターフェースのテスト用モック。
// 関数フィールドでメソッドの振る舞いをテストケースごとに差し替える。
type mockAuthOAuthClient struct {
	// authCodeURLFn は AuthCodeURL 呼び出しの振る舞いを定義する。
	authCodeURLFn func(state, codeChallenge string) (string, error)
	// exchangeCodeFn は ExchangeCode 呼び出しの振る舞いを定義する。
	exchangeCodeFn func(ctx context.Context, code, codeVerifier string) (*oauth.TokenResponse, error)
	// extractClaimsFn は ExtractClaims 呼び出しの振る舞いを定義する。
	extractClaimsFn func(ctx context.Context, idToken string) (string, []string, error)
	// extractFullClaimsFn は ExtractFullClaims 呼び出しの振る舞いを定義する（CRIT-002 監査対応）。
	extractFullClaimsFn func(ctx context.Context, idToken string) (*oauth.IDTokenClaims, error)
	// logoutURLFn は LogoutURL 呼び出しの振る舞いを定義する。
	logoutURLFn func(idTokenHint, postLogoutRedirectURI string) (string, error)
	// discoveryCacheCleared は ClearDiscoveryCache が呼ばれたかどうかの記録フラグ。
	discoveryCacheCleared bool
}

// AuthCodeURL は認可コードフローの URL を構築するモック実装。
func (m *mockAuthOAuthClient) AuthCodeURL(state, codeChallenge string) (string, error) {
	if m.authCodeURLFn != nil {
		return m.authCodeURLFn(state, codeChallenge)
	}
	return "https://idp.example.com/auth?state=" + state, nil
}

// ExchangeCode は認可コードをトークンに交換するモック実装。
func (m *mockAuthOAuthClient) ExchangeCode(ctx context.Context, code, codeVerifier string) (*oauth.TokenResponse, error) {
	if m.exchangeCodeFn != nil {
		return m.exchangeCodeFn(ctx, code, codeVerifier)
	}
	return &oauth.TokenResponse{
		AccessToken:  "default-access-token",
		RefreshToken: "default-refresh-token",
		IDToken:      "default-id-token",
		ExpiresIn:    3600,
	}, nil
}

// ExtractClaims は JWKS 署名検証済み ID トークンから subject と realm roles を返すモック実装。
func (m *mockAuthOAuthClient) ExtractClaims(ctx context.Context, idToken string) (string, []string, error) {
	if m.extractClaimsFn != nil {
		return m.extractClaimsFn(ctx, idToken)
	}
	return "default-subject", []string{"user"}, nil
}

// LogoutURL は IdP のログアウト URL を返すモック実装。
func (m *mockAuthOAuthClient) LogoutURL(idTokenHint, postLogoutRedirectURI string) (string, error) {
	if m.logoutURLFn != nil {
		return m.logoutURLFn(idTokenHint, postLogoutRedirectURI)
	}
	return "https://idp.example.com/logout?redirect=" + postLogoutRedirectURI, nil
}

// ExtractFullClaims は JWKS 署名検証済み ID トークンから IDTokenClaims 全体を返すモック実装。
// CRIT-002 監査対応: AuthOAuthClient インターフェースの ExtractFullClaims メソッドを実装する。
func (m *mockAuthOAuthClient) ExtractFullClaims(ctx context.Context, idToken string) (*oauth.IDTokenClaims, error) {
	if m.extractFullClaimsFn != nil {
		return m.extractFullClaimsFn(ctx, idToken)
	}
	return &oauth.IDTokenClaims{
		Subject:  "default-subject",
		Roles:    []string{"user"},
		TenantID: "default-tenant",
	}, nil
}

// ClearDiscoveryCache は OIDC discovery キャッシュクリアのモック実装。
// 呼び出されたことを記録する。
func (m *mockAuthOAuthClient) ClearDiscoveryCache() {
	m.discoveryCacheCleared = true
}

// mockSessionStoreForAuth は session.Store と session.ExchangeCodeStore の両インターフェースのテスト用モック。
// インメモリ map でセッションおよび交換コードを管理し、Create 失敗シミュレートを関数フィールドで提供する。
// H-5 監査対応: ExchangeCodeStore インターフェースを実装し、交換コード専用の操作を提供する。
type mockSessionStoreForAuth struct {
	// sessions はインメモリのセッションデータストア。
	sessions map[string]*session.SessionData
	// exchangeCodes はインメモリの交換コードデータストア（H-5 監査対応）。
	exchangeCodes map[string]*session.ExchangeCodeData
	// counter は Create のたびにインクリメントされる連番（一意な ID 生成用）。
	counter int
	// createFn は Create の振る舞いを上書きする。nil の場合はデフォルト動作。
	createFn func(ctx context.Context, data *session.SessionData, ttl time.Duration) (string, error)
	// exchangeCodeCounter は CreateExchangeCode のたびにインクリメントされる連番。
	exchangeCodeCounter int
}

// newMockSessionStoreForAuth はテスト用インメモリセッションストアを生成する。
func newMockSessionStoreForAuth() *mockSessionStoreForAuth {
	return &mockSessionStoreForAuth{
		sessions:      make(map[string]*session.SessionData),
		exchangeCodes: make(map[string]*session.ExchangeCodeData),
	}
}

// Create はセッションデータを保存し、連番の ID を返す。createFn が設定されていればそちらに委譲する。
func (m *mockSessionStoreForAuth) Create(ctx context.Context, data *session.SessionData, ttl time.Duration) (string, error) {
	if m.createFn != nil {
		return m.createFn(ctx, data, ttl)
	}
	m.counter++
	id := "auth-session-id-" + string(rune('0'+m.counter))
	m.sessions[id] = data
	return id, nil
}

// Get は指定 ID のセッションデータを取得する。存在しない場合は nil を返す。
func (m *mockSessionStoreForAuth) Get(_ context.Context, id string) (*session.SessionData, error) {
	if s, ok := m.sessions[id]; ok {
		return s, nil
	}
	return nil, nil
}

// Update はセッションデータを更新する。
func (m *mockSessionStoreForAuth) Update(_ context.Context, id string, data *session.SessionData, _ time.Duration) error {
	m.sessions[id] = data
	return nil
}

// Delete はセッションデータを削除する。
func (m *mockSessionStoreForAuth) Delete(_ context.Context, id string) error {
	delete(m.sessions, id)
	return nil
}

// Touch は TTL 延長の空実装（AuthUseCase では使用しない）。
func (m *mockSessionStoreForAuth) Touch(_ context.Context, _ string, _ time.Duration) error {
	return nil
}

// CreateExchangeCode はモバイルフロー用交換コードデータを保存し、連番のコードキーを返す（H-5 監査対応）。
func (m *mockSessionStoreForAuth) CreateExchangeCode(_ context.Context, data *session.ExchangeCodeData, _ time.Duration) (string, error) {
	m.exchangeCodeCounter++
	code := "exchange-code-" + string(rune('0'+m.exchangeCodeCounter))
	m.exchangeCodes[code] = data
	return code, nil
}

// GetExchangeCode は交換コードキーに対応する ExchangeCodeData を取得する（H-5 監査対応）。
func (m *mockSessionStoreForAuth) GetExchangeCode(_ context.Context, code string) (*session.ExchangeCodeData, error) {
	if d, ok := m.exchangeCodes[code]; ok {
		return d, nil
	}
	return nil, nil
}

// DeleteExchangeCode は交換コードを削除する（H-5 監査対応）。
func (m *mockSessionStoreForAuth) DeleteExchangeCode(_ context.Context, code string) error {
	delete(m.exchangeCodes, code)
	return nil
}

// ── Login テスト ──────────────────────────────────────────────

// TestLogin_Success は認可コードフローの開始処理が正常に動作することを検証する。
// AuthCodeURL が呼ばれ、State・CodeVerifier・AuthURL が返されること。
func TestLogin_Success(t *testing.T) {
	mockClient := &mockAuthOAuthClient{
		authCodeURLFn: func(state, codeChallenge string) (string, error) {
			return "https://idp.example.com/auth?state=" + state + "&code_challenge=" + codeChallenge, nil
		},
	}
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	out, err := uc.Login(context.Background(), LoginInput{
		RedirectTo: "",
	})

	if err != nil {
		t.Fatalf("Login は nil エラーを期待したが %v が返った", err)
	}
	if out == nil {
		t.Fatal("Login は非 nil 出力を期待したが nil が返った")
	}
	// AuthURL が空でないこと
	if out.AuthURL == "" {
		t.Error("AuthURL が空")
	}
	// State が空でないこと（ランダム生成される）
	if out.State == "" {
		t.Error("State が空")
	}
	// CodeVerifier が空でないこと（PKCE 生成される）
	if out.CodeVerifier == "" {
		t.Error("CodeVerifier が空")
	}
	// RedirectTo が空のため AllowMobileRedirect は false であること
	if out.AllowMobileRedirect {
		t.Error("AllowMobileRedirect: want false（RedirectTo が空）, got true")
	}
}

// TestLogin_WithValidMobileRedirect はモバイルリダイレクト先スキームが許可されている場合に
// AllowMobileRedirect が true になることを検証する。
func TestLogin_WithValidMobileRedirect(t *testing.T) {
	mockClient := &mockAuthOAuthClient{}
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	out, err := uc.Login(context.Background(), LoginInput{
		// k1s0:// スキームは許可リストに含まれる
		RedirectTo: "k1s0://app/callback",
	})

	if err != nil {
		t.Fatalf("Login は nil エラーを期待したが %v が返った", err)
	}
	if !out.AllowMobileRedirect {
		t.Error("AllowMobileRedirect: want true（k1s0:// は許可スキーム）, got false")
	}
}

// TestLogin_WithInvalidMobileRedirect は不許可スキームの場合に
// AllowMobileRedirect が false になることを検証する。
func TestLogin_WithInvalidMobileRedirect(t *testing.T) {
	mockClient := &mockAuthOAuthClient{}
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	out, err := uc.Login(context.Background(), LoginInput{
		// https:// スキームは許可リストに含まれない
		RedirectTo: "https://evil.example.com",
	})

	if err != nil {
		t.Fatalf("Login は nil エラーを期待したが %v が返った", err)
	}
	if out.AllowMobileRedirect {
		t.Error("AllowMobileRedirect: want false（https:// は不許可スキーム）, got true")
	}
}

// TestLogin_AuthCodeURLError は AuthCodeURL がエラーを返した場合に
// Login がエラーを返すことを検証する。
func TestLogin_AuthCodeURLError(t *testing.T) {
	authErr := errors.New("idp unreachable")
	mockClient := &mockAuthOAuthClient{
		authCodeURLFn: func(_, _ string) (string, error) {
			return "", authErr
		},
	}
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	out, err := uc.Login(context.Background(), LoginInput{})

	if out != nil {
		t.Errorf("Login は nil 出力を期待したが %v が返った", out)
	}
	if err == nil {
		t.Fatal("Login はエラーを期待したが nil が返った")
	}
	var ucErr *AuthUseCaseError
	if !errors.As(err, &ucErr) {
		t.Fatalf("エラー型: want *AuthUseCaseError, got %T", err)
	}
	if ucErr.Code != "BFF_AUTH_URL_ERROR" {
		t.Errorf("エラーコード: want %q, got %q", "BFF_AUTH_URL_ERROR", ucErr.Code)
	}
}

// ── Callback テスト ───────────────────────────────────────────

// TestCallback_BrowserFlow_Success はブラウザフローのコールバック処理が正常に動作することを検証する。
// セッション固定化防止（既存セッション削除）、state 検証、トークン交換、セッション作成を確認する。
func TestCallback_BrowserFlow_Success(t *testing.T) {
	mockClient := &mockAuthOAuthClient{
		exchangeCodeFn: func(_ context.Context, code, _ string) (*oauth.TokenResponse, error) {
			if code != "valid-code" {
				return nil, errors.New("invalid code")
			}
			return &oauth.TokenResponse{
				AccessToken:  "callback-access-token",
				RefreshToken: "callback-refresh-token",
				IDToken:      "callback-id-token",
				ExpiresIn:    3600,
			}, nil
		},
		extractClaimsFn: func(_ context.Context, _ string) (string, []string, error) {
			return "test-subject", []string{"user", "admin"}, nil
		},
	}

	store := newMockSessionStoreForAuth()
	// セッション固定化防止: 既存セッションを登録しておく
	existingSess := testutil.CreateTestSession(testutil.SessionOptions{})
	store.sessions["old-session-id"] = existingSess

	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	out, err := uc.Callback(context.Background(), CallbackInput{
		ExistingSessionID: "old-session-id",
		State:             "csrf-state-value",
		CookieState:       "csrf-state-value",
		Code:              "valid-code",
		CodeVerifier:      "code-verifier-value",
		PostAuthRedirect:  "", // ブラウザフロー
		SessionTTL:        30 * time.Minute,
	})

	if err != nil {
		t.Fatalf("Callback は nil エラーを期待したが %v が返った", err)
	}
	if out == nil {
		t.Fatal("Callback は非 nil 出力を期待したが nil が返った")
	}
	// 新しいセッション ID が返ること
	if out.SessionID == "" {
		t.Error("SessionID が空")
	}
	// CSRF トークンが返ること
	if out.CSRFToken == "" {
		t.Error("CSRFToken が空")
	}
	// ブラウザフロー: MobileRedirectURL が空であること
	if out.MobileRedirectURL != "" {
		t.Errorf("MobileRedirectURL: want empty, got %q", out.MobileRedirectURL)
	}
	// セッション固定化防止: 既存セッションが削除されていること（S-03 対応）
	if _, exists := store.sessions["old-session-id"]; exists {
		t.Error("セッション固定化防止: 既存セッションが削除されていない（S-03）")
	}
}

// TestCallback_MobileFlow_Success はモバイルフローのコールバック処理が正常に動作することを検証する。
// PostAuthRedirect が設定された場合に MobileRedirectURL（交換コード付き）が返ることを確認する。
func TestCallback_MobileFlow_Success(t *testing.T) {
	mockClient := &mockAuthOAuthClient{
		exchangeCodeFn: func(_ context.Context, _, _ string) (*oauth.TokenResponse, error) {
			return &oauth.TokenResponse{
				AccessToken: "mobile-access-token",
				IDToken:     "mobile-id-token",
				ExpiresIn:   3600,
			}, nil
		},
		extractClaimsFn: func(_ context.Context, _ string) (string, []string, error) {
			return "mobile-subject", []string{"user"}, nil
		},
	}

	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	out, err := uc.Callback(context.Background(), CallbackInput{
		State:            "state-value",
		CookieState:      "state-value",
		Code:             "auth-code",
		CodeVerifier:     "verifier",
		PostAuthRedirect: "k1s0://app/callback", // モバイルフロー
		SessionTTL:       30 * time.Minute,
	})

	if err != nil {
		t.Fatalf("Callback は nil エラーを期待したが %v が返った", err)
	}
	if out == nil {
		t.Fatal("Callback は非 nil 出力を期待したが nil が返った")
	}
	// モバイルフロー: MobileRedirectURL が設定されていること
	if out.MobileRedirectURL == "" {
		t.Error("MobileRedirectURL が空（モバイルフロー時は設定されるべき）")
	}
	// MobileRedirectURL に交換コードが含まれていること
	if !containsCode(out.MobileRedirectURL) {
		t.Errorf("MobileRedirectURL に code パラメータが含まれていない: %q", out.MobileRedirectURL)
	}
}

// containsCode は URL 文字列に "code=" クエリパラメータが含まれるかを確認するヘルパー。
func containsCode(rawURL string) bool {
	return len(rawURL) > 0 && contains(rawURL, "code=")
}

// contains は target が s に含まれるかを確認するヘルパー。
// strings パッケージのインポートを避けるためインライン実装する。
func contains(s, target string) bool {
	if len(target) == 0 {
		return true
	}
	for i := 0; i <= len(s)-len(target); i++ {
		if s[i:i+len(target)] == target {
			return true
		}
	}
	return false
}

// TestCallback_StateMissing は Cookie の state が空の場合にエラーが返ることを検証する。
func TestCallback_StateMissing(t *testing.T) {
	mockClient := &mockAuthOAuthClient{}
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	_, err := uc.Callback(context.Background(), CallbackInput{
		State:       "some-state",
		CookieState: "", // Cookie の state が空
		Code:        "code",
		CodeVerifier: "verifier",
	})

	if err == nil {
		t.Fatal("Callback はエラーを期待したが nil が返った")
	}
	var ucErr *AuthUseCaseError
	if !errors.As(err, &ucErr) {
		t.Fatalf("エラー型: want *AuthUseCaseError, got %T", err)
	}
	if ucErr.Code != "BFF_AUTH_STATE_MISSING" {
		t.Errorf("エラーコード: want %q, got %q", "BFF_AUTH_STATE_MISSING", ucErr.Code)
	}
}

// TestCallback_StateMismatch は state パラメータが不一致の場合にエラーが返ることを検証する（CSRF 保護）。
func TestCallback_StateMismatch(t *testing.T) {
	mockClient := &mockAuthOAuthClient{}
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	_, err := uc.Callback(context.Background(), CallbackInput{
		State:        "request-state",
		CookieState:  "different-cookie-state", // 不一致
		Code:         "code",
		CodeVerifier: "verifier",
	})

	if err == nil {
		t.Fatal("Callback はエラーを期待したが nil が返った")
	}
	var ucErr *AuthUseCaseError
	if !errors.As(err, &ucErr) {
		t.Fatalf("エラー型: want *AuthUseCaseError, got %T", err)
	}
	if ucErr.Code != "BFF_AUTH_STATE_MISMATCH" {
		t.Errorf("エラーコード: want %q, got %q", "BFF_AUTH_STATE_MISMATCH", ucErr.Code)
	}
}

// TestCallback_TokenExchangeFailure はトークン交換が失敗した場合にエラーが返ることを検証する。
func TestCallback_TokenExchangeFailure(t *testing.T) {
	exchangeErr := errors.New("token exchange failed")
	mockClient := &mockAuthOAuthClient{
		exchangeCodeFn: func(_ context.Context, _, _ string) (*oauth.TokenResponse, error) {
			return nil, exchangeErr
		},
	}
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	_, err := uc.Callback(context.Background(), CallbackInput{
		State:        "state",
		CookieState:  "state",
		Code:         "auth-code",
		CodeVerifier: "verifier",
		SessionTTL:   30 * time.Minute,
	})

	if err == nil {
		t.Fatal("Callback はエラーを期待したが nil が返った")
	}
	var ucErr *AuthUseCaseError
	if !errors.As(err, &ucErr) {
		t.Fatalf("エラー型: want *AuthUseCaseError, got %T", err)
	}
	if ucErr.Code != "BFF_AUTH_TOKEN_EXCHANGE_FAILED" {
		t.Errorf("エラーコード: want %q, got %q", "BFF_AUTH_TOKEN_EXCHANGE_FAILED", ucErr.Code)
	}
}

// ── Logout テスト ─────────────────────────────────────────────

// TestLogout_Success はセッションが存在する場合のログアウト処理を検証する。
// セッションが削除され、IdP のログアウト URL が返されること。
func TestLogout_Success(t *testing.T) {
	mockClient := &mockAuthOAuthClient{
		logoutURLFn: func(idTokenHint, postLogoutRedirectURI string) (string, error) {
			return "https://idp.example.com/logout?id_token_hint=" + idTokenHint, nil
		},
	}

	store := newMockSessionStoreForAuth()
	// ログアウト対象のセッションを登録する
	sess := testutil.CreateTestSession(testutil.SessionOptions{
		IDToken: "logout-id-token",
	})
	store.sessions["logout-session-id"] = sess

	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	out, err := uc.Logout(context.Background(), LogoutInput{
		SessionID:     "logout-session-id",
		PostLogoutURI: "https://app.example.com",
	})

	if err != nil {
		t.Fatalf("Logout は nil エラーを期待したが %v が返った", err)
	}
	if out == nil {
		t.Fatal("Logout は非 nil 出力を期待したが nil が返った")
	}
	// IdP ログアウト URL が返されること
	if out.IdPLogoutURL == "" {
		t.Error("IdPLogoutURL が空")
	}
	// セッションが削除されていること
	if _, exists := store.sessions["logout-session-id"]; exists {
		t.Error("ログアウト後、セッションは削除されるべき")
	}
	// ClearDiscoveryCache が呼ばれていること
	if !mockClient.discoveryCacheCleared {
		t.Error("ClearDiscoveryCache が呼ばれていない")
	}
}

// TestLogout_NoSession はセッション ID が空の場合のログアウト処理を検証する。
// セッション ID なしでもエラーなく処理できること。
func TestLogout_NoSession(t *testing.T) {
	mockClient := &mockAuthOAuthClient{}
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	out, err := uc.Logout(context.Background(), LogoutInput{
		SessionID:     "", // セッション ID なし
		PostLogoutURI: "https://app.example.com",
	})

	if err != nil {
		t.Fatalf("Logout は nil エラーを期待したが %v が返った", err)
	}
	if out == nil {
		t.Fatal("Logout は非 nil 出力を期待したが nil が返った")
	}
	// セッションなし: IdP ログアウト URL は空（セッションデータなし）
	if out.IdPLogoutURL != "" {
		t.Errorf("IdPLogoutURL: want empty（セッションなし）, got %q", out.IdPLogoutURL)
	}
	// フォールバック URI が返されること
	if out.FallbackURI != "https://app.example.com" {
		t.Errorf("FallbackURI: want %q, got %q", "https://app.example.com", out.FallbackURI)
	}
}

// TestLogout_IdPLogoutURLError は IdP ログアウト URL の構築が失敗した場合に
// フォールバック URI が返されることを検証する。
func TestLogout_IdPLogoutURLError(t *testing.T) {
	mockClient := &mockAuthOAuthClient{
		logoutURLFn: func(_, _ string) (string, error) {
			return "", errors.New("idp logout url error")
		},
	}

	store := newMockSessionStoreForAuth()
	sess := testutil.CreateTestSession(testutil.SessionOptions{IDToken: "id-token"})
	store.sessions["session-with-idp-error"] = sess

	uc := NewAuthUseCase(mockClient, store, store, 0) // 0 → defaultExchangeCodeTTL(60s)

	out, err := uc.Logout(context.Background(), LogoutInput{
		SessionID:     "session-with-idp-error",
		PostLogoutURI: "https://app.example.com/fallback",
	})

	if err != nil {
		t.Fatalf("Logout は nil エラーを期待したが %v が返った", err)
	}
	// IdP ログアウト URL 構築失敗時: IdPLogoutURL は空
	if out.IdPLogoutURL != "" {
		t.Errorf("IdPLogoutURL: want empty（構築失敗）, got %q", out.IdPLogoutURL)
	}
	// フォールバック URI が返されること
	if out.FallbackURI != "https://app.example.com/fallback" {
		t.Errorf("FallbackURI: want %q, got %q", "https://app.example.com/fallback", out.FallbackURI)
	}
}

// ── CheckSession テスト ───────────────────────────────────────

// TestCheckSession_Success は有効なセッションが存在する場合にユーザー情報が返ることを検証する。
func TestCheckSession_Success(t *testing.T) {
	store := newMockSessionStoreForAuth()
	sess := testutil.CreateTestSession(testutil.SessionOptions{
		Subject:   "check-subject",
		CSRFToken: "check-csrf-token",
		Roles:     []string{"admin", "user"},
	})
	store.sessions["valid-session-id"] = sess

	uc := NewAuthUseCase(&mockAuthOAuthClient{}, store, store, 0) // 0 → defaultExchangeCodeTTL(60s) MEDIUM-006 監査対応

	out, err := uc.CheckSession(context.Background(), SessionCheckInput{
		SessionID: "valid-session-id",
	})

	if err != nil {
		t.Fatalf("CheckSession は nil エラーを期待したが %v が返った", err)
	}
	if out == nil {
		t.Fatal("CheckSession は非 nil 出力を期待したが nil が返った")
	}
	if out.Subject != "check-subject" {
		t.Errorf("Subject: want %q, got %q", "check-subject", out.Subject)
	}
	if out.CSRFToken != "check-csrf-token" {
		t.Errorf("CSRFToken: want %q, got %q", "check-csrf-token", out.CSRFToken)
	}
	if len(out.Roles) != 2 {
		t.Errorf("Roles 数: want 2, got %d", len(out.Roles))
	}
}

// TestCheckSession_NotFound はセッション ID が空の場合にエラーが返ることを検証する。
func TestCheckSession_NotFound(t *testing.T) {
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(&mockAuthOAuthClient{}, store, store, 0) // 0 → defaultExchangeCodeTTL(60s) MEDIUM-006 監査対応

	_, err := uc.CheckSession(context.Background(), SessionCheckInput{
		SessionID: "", // 空の SessionID
	})

	if err == nil {
		t.Fatal("CheckSession はエラーを期待したが nil が返った")
	}
	var ucErr *AuthUseCaseError
	if !errors.As(err, &ucErr) {
		t.Fatalf("エラー型: want *AuthUseCaseError, got %T", err)
	}
	if ucErr.Code != "BFF_AUTH_SESSION_NOT_FOUND" {
		t.Errorf("エラーコード: want %q, got %q", "BFF_AUTH_SESSION_NOT_FOUND", ucErr.Code)
	}
}

// TestCheckSession_Expired は期限切れセッションの場合にエラーが返ることを検証する。
func TestCheckSession_Expired(t *testing.T) {
	store := newMockSessionStoreForAuth()
	// 期限切れセッションを登録する
	expiredSess := testutil.CreateExpiredSession("expired-subject", "expired-csrf")
	store.sessions["expired-session-id"] = expiredSess

	uc := NewAuthUseCase(&mockAuthOAuthClient{}, store, store, 0) // 0 → defaultExchangeCodeTTL(60s) MEDIUM-006 監査対応

	_, err := uc.CheckSession(context.Background(), SessionCheckInput{
		SessionID: "expired-session-id",
	})

	if err == nil {
		t.Fatal("CheckSession はエラーを期待したが nil が返った")
	}
	var ucErr *AuthUseCaseError
	if !errors.As(err, &ucErr) {
		t.Fatalf("エラー型: want *AuthUseCaseError, got %T", err)
	}
	if ucErr.Code != "BFF_AUTH_SESSION_EXPIRED" {
		t.Errorf("エラーコード: want %q, got %q", "BFF_AUTH_SESSION_EXPIRED", ucErr.Code)
	}
}

// TestCheckSession_RolesNilToEmpty は roles が nil の場合に空スライスが返ることを検証する。
// JSON シリアライズで null ではなく [] になることを保証する。
func TestCheckSession_RolesNilToEmpty(t *testing.T) {
	store := newMockSessionStoreForAuth()
	sess := &session.SessionData{
		Subject:   "nil-roles-subject",
		CSRFToken: "nil-roles-csrf",
		ExpiresAt: time.Now().Add(time.Hour).Unix(),
		Roles:     nil, // roles が nil
	}
	store.sessions["nil-roles-session"] = sess

	uc := NewAuthUseCase(&mockAuthOAuthClient{}, store, store, 0) // 0 → defaultExchangeCodeTTL(60s) MEDIUM-006 監査対応

	out, err := uc.CheckSession(context.Background(), SessionCheckInput{
		SessionID: "nil-roles-session",
	})

	if err != nil {
		t.Fatalf("CheckSession は nil エラーを期待したが %v が返った", err)
	}
	// roles は nil ではなく空スライスであること
	if out.Roles == nil {
		t.Error("Roles: want []string{} (空スライス), got nil")
	}
	if len(out.Roles) != 0 {
		t.Errorf("Roles 数: want 0, got %d", len(out.Roles))
	}
}

// ── ExchangeCode テスト ───────────────────────────────────────

// TestExchangeCode_Success はワンタイム交換コードが有効な場合に
// 実際のセッション ID と subject が返ることを検証する。
// H-5 監査対応: exchangeCodes マップに ExchangeCodeData を格納し、SessionData.AccessToken を使わない。
func TestExchangeCode_Success(t *testing.T) {
	store := newMockSessionStoreForAuth()

	// 実際のセッションを登録する
	realSess := testutil.CreateTestSession(testutil.SessionOptions{
		Subject:   "exchange-subject",
		CSRFToken: "exchange-csrf-token",
	})
	store.sessions["real-session-id"] = realSess

	// 交換コードエントリを ExchangeCodeData として登録する（H-5 監査対応）
	// sessions マップではなく exchangeCodes マップに格納する
	exchangeEntry := testutil.CreateExchangeCodeEntry("real-session-id", 60*time.Second)
	store.exchangeCodes["one-time-code"] = exchangeEntry

	uc := NewAuthUseCase(&mockAuthOAuthClient{}, store, store, 0) // 0 → defaultExchangeCodeTTL(60s) MEDIUM-006 監査対応

	out, err := uc.ExchangeCode(context.Background(), ExchangeInput{
		Code:       "one-time-code",
		SessionTTL: 30 * time.Minute,
	})

	if err != nil {
		t.Fatalf("ExchangeCode は nil エラーを期待したが %v が返った", err)
	}
	if out == nil {
		t.Fatal("ExchangeCode は非 nil 出力を期待したが nil が返った")
	}
	if out.RealSessionID != "real-session-id" {
		t.Errorf("RealSessionID: want %q, got %q", "real-session-id", out.RealSessionID)
	}
	if out.Subject != "exchange-subject" {
		t.Errorf("Subject: want %q, got %q", "exchange-subject", out.Subject)
	}
	if out.CSRFToken != "exchange-csrf-token" {
		t.Errorf("CSRFToken: want %q, got %q", "exchange-csrf-token", out.CSRFToken)
	}
	// ワンタイム使用: 交換コードが exchangeCodes ストアから削除されていること（H-5 監査対応）
	if _, exists := store.exchangeCodes["one-time-code"]; exists {
		t.Error("交換コードはワンタイム使用のため、使用後は削除されるべき")
	}
}

// TestExchangeCode_InvalidCode は存在しない交換コードの場合にエラーが返ることを検証する。
func TestExchangeCode_InvalidCode(t *testing.T) {
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(&mockAuthOAuthClient{}, store, store, 0) // 0 → defaultExchangeCodeTTL(60s) MEDIUM-006 監査対応

	_, err := uc.ExchangeCode(context.Background(), ExchangeInput{
		Code:       "nonexistent-code",
		SessionTTL: 30 * time.Minute,
	})

	if err == nil {
		t.Fatal("ExchangeCode はエラーを期待したが nil が返った")
	}
	var ucErr *AuthUseCaseError
	if !errors.As(err, &ucErr) {
		t.Fatalf("エラー型: want *AuthUseCaseError, got %T", err)
	}
	if ucErr.Code != "BFF_AUTH_EXCHANGE_CODE_INVALID" {
		t.Errorf("エラーコード: want %q, got %q", "BFF_AUTH_EXCHANGE_CODE_INVALID", ucErr.Code)
	}
}

// TestExchangeCode_EmptyCode は交換コードが空の場合にエラーが返ることを検証する。
func TestExchangeCode_EmptyCode(t *testing.T) {
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(&mockAuthOAuthClient{}, store, store, 0) // 0 → defaultExchangeCodeTTL(60s) MEDIUM-006 監査対応

	_, err := uc.ExchangeCode(context.Background(), ExchangeInput{
		Code: "", // 空コード
	})

	if err == nil {
		t.Fatal("ExchangeCode はエラーを期待したが nil が返った")
	}
	var ucErr *AuthUseCaseError
	if !errors.As(err, &ucErr) {
		t.Fatalf("エラー型: want *AuthUseCaseError, got %T", err)
	}
	if ucErr.Code != "BFF_AUTH_EXCHANGE_CODE_MISSING" {
		t.Errorf("エラーコード: want %q, got %q", "BFF_AUTH_EXCHANGE_CODE_MISSING", ucErr.Code)
	}
}

// ── AuthUseCaseError テスト ───────────────────────────────────

// TestAuthUseCaseError_Error は AuthUseCaseError.Error() のフォーマットを検証する。
func TestAuthUseCaseError_Error(t *testing.T) {
	// Err あり
	e := &AuthUseCaseError{Code: "AUTH_CODE_1", Err: errors.New("some detail")}
	want := "AUTH_CODE_1: some detail"
	if e.Error() != want {
		t.Errorf("Error(): want %q, got %q", want, e.Error())
	}

	// Err なし
	e2 := &AuthUseCaseError{Code: "AUTH_CODE_2"}
	if e2.Error() != "AUTH_CODE_2" {
		t.Errorf("Error(): want %q, got %q", "AUTH_CODE_2", e2.Error())
	}
}

// TestAuthUseCaseError_Unwrap は errors.Is/As によるエラー辿りを検証する。
func TestAuthUseCaseError_Unwrap(t *testing.T) {
	inner := errors.New("inner auth error")
	e := &AuthUseCaseError{Code: "AUTH_WRAP_CODE", Err: inner}

	if !errors.Is(e, inner) {
		t.Error("errors.Is によって inner error を辿れるべき")
	}
}

// ── CheckSession CSRF 再生成テスト（H-003 監査対応）──────────────────

// TestCheckSession_CSRFRefresh_WhenThresholdExceeded は CSRF トークンが csrfRefreshThreshold（25分）を
// 超えた場合に新しいトークンが生成され、セッションに保存されることを検証する（H-003 監査対応）。
func TestCheckSession_CSRFRefresh_WhenThresholdExceeded(t *testing.T) {
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(&mockAuthOAuthClient{}, store, store, 0)

	// 26分前に作成された CSRF トークン（csrfRefreshThreshold=25分 を超えている）
	oldCSRFToken := "old-csrf-token-abc123"
	sessionID, err := store.Create(context.Background(), &session.SessionData{
		AccessToken:        "access-token",
		CSRFToken:          oldCSRFToken,
		CSRFTokenCreatedAt: time.Now().Add(-26 * time.Minute).Unix(),
		ExpiresAt:          time.Now().Add(10 * time.Minute).Unix(),
		Subject:            "user-1",
	}, 30*time.Minute)
	if err != nil {
		t.Fatalf("セッション作成に失敗: %v", err)
	}

	out, err := uc.CheckSession(context.Background(), SessionCheckInput{SessionID: sessionID})
	if err != nil {
		t.Fatalf("CheckSession は nil エラーを期待したが %v が返った", err)
	}

	// CSRF トークンが再生成されていること
	if out.CSRFToken == oldCSRFToken {
		t.Error("CSRF トークンが再生成されていない（古いトークンがそのまま返された）")
	}
	if out.CSRFToken == "" {
		t.Error("CSRF トークンが空")
	}

	// セッションストアに新しいトークンが保存されていること
	updatedSess, err := store.Get(context.Background(), sessionID)
	if err != nil || updatedSess == nil {
		t.Fatalf("セッション取得に失敗: %v", err)
	}
	if updatedSess.CSRFToken != out.CSRFToken {
		t.Errorf("ストア内の CSRF トークンが返値と異なる: store=%q, out=%q", updatedSess.CSRFToken, out.CSRFToken)
	}
	if updatedSess.CSRFTokenCreatedAt == 0 {
		t.Error("CSRFTokenCreatedAt が更新されていない")
	}
}

// TestCheckSession_CSRFRefresh_SkippedWhenFresh は CSRF トークンがしきい値（25分）以内の場合に
// 再生成されないことを検証する（H-003 監査対応）。
func TestCheckSession_CSRFRefresh_SkippedWhenFresh(t *testing.T) {
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(&mockAuthOAuthClient{}, store, store, 0)

	// 5分前に作成された CSRF トークン（しきい値以内）
	originalCSRFToken := "fresh-csrf-token-xyz456"
	sessionID, err := store.Create(context.Background(), &session.SessionData{
		AccessToken:        "access-token",
		CSRFToken:          originalCSRFToken,
		CSRFTokenCreatedAt: time.Now().Add(-5 * time.Minute).Unix(),
		ExpiresAt:          time.Now().Add(10 * time.Minute).Unix(),
		Subject:            "user-2",
	}, 30*time.Minute)
	if err != nil {
		t.Fatalf("セッション作成に失敗: %v", err)
	}

	out, err := uc.CheckSession(context.Background(), SessionCheckInput{SessionID: sessionID})
	if err != nil {
		t.Fatalf("CheckSession は nil エラーを期待したが %v が返った", err)
	}

	// CSRF トークンが変更されていないこと（新鮮なので再生成不要）
	if out.CSRFToken != originalCSRFToken {
		t.Errorf("CSRF トークンが変更されてしまった: want %q, got %q", originalCSRFToken, out.CSRFToken)
	}
}

// TestCheckSession_CSRFRefresh_LegacySessionGetsRegenerated は CSRFTokenCreatedAt=0 の旧形式セッションで
// CSRF トークンが再生成されることを検証する。
// MED-001 監査対応: 後方互換ガードを削除し、旧形式セッションも TTL 超過として再生成する。
func TestCheckSession_CSRFRefresh_LegacySessionGetsRegenerated(t *testing.T) {
	store := newMockSessionStoreForAuth()
	uc := NewAuthUseCase(&mockAuthOAuthClient{}, store, store, 0)

	// CSRFTokenCreatedAt=0 の旧形式セッション（MED-001 対応: 1970-01-01 起算で TTL 超過と判定される）
	legacyCSRFToken := "legacy-csrf-token"
	sessionID, err := store.Create(context.Background(), &session.SessionData{
		AccessToken:        "access-token",
		CSRFToken:          legacyCSRFToken,
		CSRFTokenCreatedAt: 0, // 旧形式: 生成時刻未設定 → TTL 超過と判定され再生成される
		ExpiresAt:          time.Now().Add(10 * time.Minute).Unix(),
		Subject:            "user-3",
	}, 30*time.Minute)
	if err != nil {
		t.Fatalf("セッション作成に失敗: %v", err)
	}

	out, err := uc.CheckSession(context.Background(), SessionCheckInput{SessionID: sessionID})
	if err != nil {
		t.Fatalf("CheckSession は nil エラーを期待したが %v が返った", err)
	}

	// MED-001 監査対応: CSRFTokenCreatedAt=0 は TTL 超過と判定され、新しいトークンに再生成される
	if out.CSRFToken == legacyCSRFToken {
		t.Errorf("旧形式セッションの CSRF トークンが再生成されなかった: got %q（変更されるべき）", out.CSRFToken)
	}
	if out.CSRFToken == "" {
		t.Error("再生成後の CSRF トークンが空文字")
	}
}
