// proxy_usecase_test.go は ProxyUseCase のユニットテストを提供する。
// 外部依存（Redis、OAuth プロバイダー）をモックに差し替えて、
// セッション検証・トークンリフレッシュ・M-5 修正（Redis 先行更新）のビジネスロジックを検証する。
package usecase

import (
	"context"
	"errors"
	"log/slog"
	"os"
	"testing"
	"time"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/testutil"
)

// ── モック定義 ────────────────────────────────────────────────

// mockOAuthClientForProxy は OAuthClient ポートのテスト用モック。
// 関数フィールドによって各テストケースで振る舞いを差し替える。
type mockOAuthClientForProxy struct {
	// refreshTokenFn は RefreshToken 呼び出し時の振る舞いを定義する関数フィールド。
	refreshTokenFn func(ctx context.Context, refreshToken string) (*oauth.TokenResponse, error)
}

// AuthCodeURL は AuthOAuthClient インターフェースを満たすための空実装（ProxyUseCase では使用しない）。
func (m *mockOAuthClientForProxy) AuthCodeURL(_, _ string) (string, error) {
	return "", nil
}

// ExchangeCode は AuthOAuthClient インターフェースを満たすための空実装（ProxyUseCase では使用しない）。
func (m *mockOAuthClientForProxy) ExchangeCode(_ context.Context, _, _ string) (*oauth.TokenResponse, error) {
	return nil, nil
}

// ExtractClaims は AuthOAuthClient インターフェースを満たすための空実装（ProxyUseCase では使用しない）。
func (m *mockOAuthClientForProxy) ExtractClaims(_ context.Context, _ string) (string, []string, error) {
	return "", nil, nil
}

// RefreshToken はトークンリフレッシュのモック実装。
// refreshTokenFn フィールドで振る舞いを差し替える。
func (m *mockOAuthClientForProxy) RefreshToken(ctx context.Context, refreshToken string) (*oauth.TokenResponse, error) {
	return m.refreshTokenFn(ctx, refreshToken)
}

// LogoutURL は AuthOAuthClient インターフェースを満たすための空実装（ProxyUseCase では使用しない）。
func (m *mockOAuthClientForProxy) LogoutURL(_, _ string) (string, error) {
	return "", nil
}

// ClearDiscoveryCache は AuthOAuthClient インターフェースを満たすための空実装（ProxyUseCase では使用しない）。
func (m *mockOAuthClientForProxy) ClearDiscoveryCache() {}

// mockSessionStoreForProxy は session.Store インターフェースのテスト用モック。
// 各メソッドをカスタマイズ可能な関数フィールドで定義し、
// エラーケースのシミュレートを容易にする。
type mockSessionStoreForProxy struct {
	// sessions はインメモリのセッションデータストア。
	sessions map[string]*session.SessionData
	// updateFn は Update 呼び出しの振る舞いを上書きする。nil の場合はデフォルト動作（map への保存）。
	updateFn func(ctx context.Context, id string, data *session.SessionData, ttl time.Duration) error
	// deleteFn は Delete 呼び出しの振る舞いを上書きする。nil の場合はデフォルト動作（map から削除）。
	deleteFn func(ctx context.Context, id string) error
}

// newMockSessionStoreForProxy はテスト用インメモリセッションストアを生成する。
func newMockSessionStoreForProxy() *mockSessionStoreForProxy {
	return &mockSessionStoreForProxy{
		sessions: make(map[string]*session.SessionData),
	}
}

// Create はセッションデータを保存し、固定 ID を返す。
func (m *mockSessionStoreForProxy) Create(_ context.Context, data *session.SessionData, _ time.Duration) (string, error) {
	id := "created-session"
	m.sessions[id] = data
	return id, nil
}

// Get は指定 ID のセッションデータを取得する。存在しない場合は nil を返す。
func (m *mockSessionStoreForProxy) Get(_ context.Context, id string) (*session.SessionData, error) {
	if s, ok := m.sessions[id]; ok {
		return s, nil
	}
	return nil, nil
}

// Update はセッションデータを更新する。updateFn が設定されている場合はそちらに委譲する。
func (m *mockSessionStoreForProxy) Update(ctx context.Context, id string, data *session.SessionData, ttl time.Duration) error {
	if m.updateFn != nil {
		return m.updateFn(ctx, id, data, ttl)
	}
	// デフォルト: map にコピーして保存する（M-5: shallow copy で保存することを確認できる）
	copied := *data
	m.sessions[id] = &copied
	return nil
}

// Delete はセッションデータを削除する。deleteFn が設定されている場合はそちらに委譲する。
func (m *mockSessionStoreForProxy) Delete(ctx context.Context, id string) error {
	if m.deleteFn != nil {
		return m.deleteFn(ctx, id)
	}
	// デフォルト: map から削除する
	delete(m.sessions, id)
	return nil
}

// Touch は TTL 延長の空実装（ProxyUseCase では使用しない）。
func (m *mockSessionStoreForProxy) Touch(_ context.Context, _ string, _ time.Duration) error {
	return nil
}

// ── テストヘルパー ────────────────────────────────────────────

// newTestLogger はテスト用に stderr へ出力する slog ロガーを生成する。
// エラーレベルのログのみ出力し、テスト出力を最小限に抑える。
func newTestLogger() *slog.Logger {
	return slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelError}))
}

// ── テストケース ─────────────────────────────────────────────

// TestPrepareProxy_ValidSession は有効なセッション（期限内）で
// PrepareProxy が AccessToken を正常に返すことを検証する。
// NeedsRefresh が false なのでリフレッシュは行われない。
func TestPrepareProxy_ValidSession(t *testing.T) {
	store := newMockSessionStoreForProxy()
	// セッションデータを用意する（period 内：ExpiresAt が未来）
	sess := testutil.CreateTestSession(testutil.SessionOptions{
		AccessToken: "valid-access-token",
	})
	store.sessions["test-session-id"] = sess

	// oauthClient は nil: NeedsRefresh=false なので RefreshToken は呼ばれない
	uc := NewProxyUseCase(nil, store, 30*time.Minute, newTestLogger())

	out, err := uc.PrepareProxy(context.Background(), PrepareProxyInput{
		SessionData:  sess,
		SessionID:    "test-session-id",
		NeedsRefresh: false,
	})

	if err != nil {
		t.Fatalf("PrepareProxy は nil エラーを期待したが %v が返った", err)
	}
	if out == nil {
		t.Fatal("PrepareProxy は非 nil 出力を期待したが nil が返った")
	}
	if out.AccessToken != "valid-access-token" {
		t.Errorf("AccessToken: want %q, got %q", "valid-access-token", out.AccessToken)
	}
}

// TestPrepareProxy_NeedsRefresh_Success はセッションが期限切れ時に
// トークンリフレッシュが成功し、新しい AccessToken が返されることを検証する。
// また M-5 修正として Redis（sessionStore）が先に更新されることも確認する。
func TestPrepareProxy_NeedsRefresh_Success(t *testing.T) {
	store := newMockSessionStoreForProxy()
	// 期限切れセッションを用意する
	sess := testutil.CreateTestSession(testutil.SessionOptions{
		AccessToken:  "old-access-token",
		RefreshToken: "old-refresh-token",
		TTL:          -time.Hour, // 1 時間前に期限切れ
	})
	store.sessions["session-needs-refresh"] = sess

	// UpdateFn が呼ばれたことを記録するフラグ
	updateCalled := false
	store.updateFn = func(_ context.Context, id string, data *session.SessionData, _ time.Duration) error {
		updateCalled = true
		// M-5: Redis 更新時点でメモリ上のセッション（sess）はまだ古い AccessToken のはず
		if data.AccessToken != "new-access-token" {
			t.Errorf("Redis 更新データの AccessToken: want %q, got %q", "new-access-token", data.AccessToken)
		}
		// M-5: Redis 更新前にメモリ（sess）が変更されていないことを確認する
		if sess.AccessToken != "old-access-token" {
			t.Errorf("Redis 更新前にメモリの AccessToken が変更されている: got %q", sess.AccessToken)
		}
		// デフォルト動作: map に保存する
		copied := *data
		store.sessions[id] = &copied
		return nil
	}

	// リフレッシュ成功を返すモック OAuthClient を構築する
	mockClient := &mockOAuthClientForProxy{
		refreshTokenFn: func(_ context.Context, _ string) (*oauth.TokenResponse, error) {
			return &oauth.TokenResponse{
				AccessToken:  "new-access-token",
				RefreshToken: "new-refresh-token",
				IDToken:      "new-id-token",
				ExpiresIn:    3600,
			}, nil
		},
	}

	uc := NewProxyUseCase(mockClient, store, 30*time.Minute, newTestLogger())

	out, err := uc.PrepareProxy(context.Background(), PrepareProxyInput{
		SessionData:  sess,
		SessionID:    "session-needs-refresh",
		NeedsRefresh: true,
	})

	if err != nil {
		t.Fatalf("PrepareProxy は nil エラーを期待したが %v が返った", err)
	}
	if out == nil {
		t.Fatal("PrepareProxy は非 nil 出力を期待したが nil が返った")
	}
	// リフレッシュ後の新しい AccessToken が返ること
	if out.AccessToken != "new-access-token" {
		t.Errorf("AccessToken: want %q, got %q", "new-access-token", out.AccessToken)
	}
	// Redis（sessionStore）の Update が呼ばれていること
	if !updateCalled {
		t.Error("sessionStore.Update が呼ばれなかった")
	}
	// M-5: Redis 更新成功後にメモリ上のセッションも更新されていること
	if sess.AccessToken != "new-access-token" {
		t.Errorf("Redis 更新成功後のメモリ AccessToken: want %q, got %q", "new-access-token", sess.AccessToken)
	}
	if sess.RefreshToken != "new-refresh-token" {
		t.Errorf("Redis 更新成功後のメモリ RefreshToken: want %q, got %q", "new-refresh-token", sess.RefreshToken)
	}
}

// TestPrepareProxy_NeedsRefresh_Failure はトークンリフレッシュが失敗した場合に
// エラーが返され、無効なセッションがストアから削除されることを検証する（H-003 対応）。
func TestPrepareProxy_NeedsRefresh_Failure(t *testing.T) {
	store := newMockSessionStoreForProxy()
	// 期限切れセッションを登録する
	sess := testutil.CreateTestSession(testutil.SessionOptions{
		AccessToken:  "expired-token",
		RefreshToken: "expired-refresh-token",
		TTL:          -time.Hour,
	})
	store.sessions["expired-session"] = sess

	// リフレッシュ失敗を返すモック OAuthClient を構築する
	refreshErr := errors.New("invalid_grant")
	mockClient := &mockOAuthClientForProxy{
		refreshTokenFn: func(_ context.Context, _ string) (*oauth.TokenResponse, error) {
			return nil, refreshErr
		},
	}

	uc := NewProxyUseCase(mockClient, store, 30*time.Minute, newTestLogger())

	out, err := uc.PrepareProxy(context.Background(), PrepareProxyInput{
		SessionData:  sess,
		SessionID:    "expired-session",
		NeedsRefresh: true,
	})

	// リフレッシュ失敗時は nil 出力 + エラーが返ること
	if out != nil {
		t.Errorf("PrepareProxy は nil 出力を期待したが %v が返った", out)
	}
	if err == nil {
		t.Fatal("PrepareProxy はエラーを期待したが nil が返った")
	}
	// エラーコードの確認
	var ucErr *ProxyUseCaseError
	if !errors.As(err, &ucErr) {
		t.Fatalf("エラー型: want *ProxyUseCaseError, got %T", err)
	}
	if ucErr.Code != "BFF_PROXY_TOKEN_EXPIRED" {
		t.Errorf("エラーコード: want %q, got %q", "BFF_PROXY_TOKEN_EXPIRED", ucErr.Code)
	}
	// H-003: リフレッシュ失敗後にセッションがストアから削除されていること
	if _, exists := store.sessions["expired-session"]; exists {
		t.Error("リフレッシュ失敗後、無効なセッションはストアから削除されるべき（H-003）")
	}
}

// TestPrepareProxy_SessionUpdateFailure は M-5 修正のテスト。
// Redis（sessionStore）の Update が失敗した場合に:
//   - エラーが返されること
//   - メモリ上のセッション（AccessToken）が変更されていないこと
//
// Redis 更新前にメモリを変更しないことで状態乖離を防ぐ設計を検証する。
func TestPrepareProxy_SessionUpdateFailure(t *testing.T) {
	store := newMockSessionStoreForProxy()
	// 期限切れセッションを登録する
	sess := testutil.CreateTestSession(testutil.SessionOptions{
		AccessToken:  "before-update-token",
		RefreshToken: "refresh-token",
		TTL:          -time.Hour,
	})
	store.sessions["update-fail-session"] = sess

	// Update を必ずエラーにする
	updateErr := errors.New("redis connection lost")
	store.updateFn = func(_ context.Context, _ string, _ *session.SessionData, _ time.Duration) error {
		return updateErr
	}

	// リフレッシュ自体は成功するモック OAuthClient を構築する
	mockClient := &mockOAuthClientForProxy{
		refreshTokenFn: func(_ context.Context, _ string) (*oauth.TokenResponse, error) {
			return &oauth.TokenResponse{
				AccessToken: "new-token-after-refresh",
				ExpiresIn:   3600,
			}, nil
		},
	}

	uc := NewProxyUseCase(mockClient, store, 30*time.Minute, newTestLogger())

	out, err := uc.PrepareProxy(context.Background(), PrepareProxyInput{
		SessionData:  sess,
		SessionID:    "update-fail-session",
		NeedsRefresh: true,
	})

	// Redis 更新失敗時は nil 出力 + エラーが返ること
	if out != nil {
		t.Errorf("PrepareProxy は nil 出力を期待したが %v が返った", out)
	}
	if err == nil {
		t.Fatal("PrepareProxy はエラーを期待したが nil が返った")
	}
	// エラーコードの確認
	var ucErr *ProxyUseCaseError
	if !errors.As(err, &ucErr) {
		t.Fatalf("エラー型: want *ProxyUseCaseError, got %T", err)
	}
	if ucErr.Code != "BFF_PROXY_SESSION_UPDATE_FAILED" {
		t.Errorf("エラーコード: want %q, got %q", "BFF_PROXY_SESSION_UPDATE_FAILED", ucErr.Code)
	}
	// M-5: Redis 更新失敗後はメモリ上のセッション（AccessToken）が変更されていないこと
	if sess.AccessToken != "before-update-token" {
		t.Errorf("M-5: Redis 更新失敗後にメモリの AccessToken が変更されてしまっている: got %q", sess.AccessToken)
	}
}

// TestPrepareProxy_SessionNotFound はセッションデータが nil の場合に
// PrepareProxy が nil ポインタで panic せず、正常に処理することを検証する。
// SessionMiddleware が nil を渡すことは想定しないが、防御的に動作を確認する。
// NeedsRefresh=false かつ SessionData が空の場合、AccessToken は空文字が返る。
func TestPrepareProxy_SessionNotFound(t *testing.T) {
	store := newMockSessionStoreForProxy()
	// セッションデータが存在しない（nil）場合を表現するため、空のセッションを使用する。
	// PrepareProxy のシグネチャでは *session.SessionData を受け取るため、
	// 「セッション未発見」は handler 層（SessionMiddleware）が担当し、
	// usecase は SessionData が nil でない前提で動作する。
	// ここでは NeedsRefresh=false + AccessToken="" のケースで
	// 空文字の AccessToken が返ることを確認する。
	emptySess := &session.SessionData{
		AccessToken: "",
		ExpiresAt:   time.Now().Add(time.Hour).Unix(),
	}

	uc := NewProxyUseCase(nil, store, 30*time.Minute, newTestLogger())

	out, err := uc.PrepareProxy(context.Background(), PrepareProxyInput{
		SessionData:  emptySess,
		SessionID:    "empty-session",
		NeedsRefresh: false,
	})

	// エラーなし、空の AccessToken が返ること
	if err != nil {
		t.Fatalf("PrepareProxy は nil エラーを期待したが %v が返った", err)
	}
	if out == nil {
		t.Fatal("PrepareProxy は非 nil 出力を期待したが nil が返った")
	}
	if out.AccessToken != "" {
		t.Errorf("空セッションの AccessToken: want %q, got %q", "", out.AccessToken)
	}
}

// TestProxyUseCaseError_Error は ProxyUseCaseError.Error() のフォーマットを検証する。
func TestProxyUseCaseError_Error(t *testing.T) {
	// Err あり
	e := &ProxyUseCaseError{Code: "CODE_1", Err: errors.New("detail")}
	want := "CODE_1: detail"
	if e.Error() != want {
		t.Errorf("Error(): want %q, got %q", want, e.Error())
	}

	// Err なし
	e2 := &ProxyUseCaseError{Code: "CODE_2"}
	if e2.Error() != "CODE_2" {
		t.Errorf("Error(): want %q, got %q", "CODE_2", e2.Error())
	}
}

// TestProxyUseCaseError_Unwrap は errors.Is/As によるエラー辿りを検証する。
func TestProxyUseCaseError_Unwrap(t *testing.T) {
	inner := errors.New("inner error")
	e := &ProxyUseCaseError{Code: "WRAP_CODE", Err: inner}

	if !errors.Is(e, inner) {
		t.Error("errors.Is によって inner error を辿れるべき")
	}
}
