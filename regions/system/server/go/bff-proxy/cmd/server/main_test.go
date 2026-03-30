package main

import (
	"context"
	"log/slog"
	"os"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/config"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
)

// isProductionEnvironment の環境判定ロジックをテストする。
// M-011 監査対応: fail-safe パターン（デフォルト本番）に変更後のテスト。
// dev/development/test/local のみ false、それ以外は true を返すことを検証する。
func TestIsProductionEnvironment(t *testing.T) {
	cases := []struct {
		env      string
		expected bool
	}{
		// 本番環境判定（明示的に本番を指定した場合）
		{env: "prod", expected: true},
		{env: "production", expected: true},
		// 大文字・スペース混在でも正しく判定する
		{env: "PROD", expected: true},
		{env: "PRODUCTION", expected: true},
		{env: " prod ", expected: true},
		// M-011: タイポは安全のため本番扱いとなる（fail-safe）
		{env: "prodction", expected: true},
		// M-011: staging は明示的な非本番リストにないため本番扱いとなる
		{env: "staging", expected: true},
		// M-011: 空文字は安全のため本番扱いとなる（fail-safe）
		{env: "", expected: true},
		// 明示的に開発・テスト環境として指定された場合のみ非本番
		{env: "dev", expected: false},
		{env: "development", expected: false},
		{env: "local", expected: false},
		{env: "test", expected: false},
	}

	for _, tc := range cases {
		got := isProductionEnvironment(tc.env)
		if got != tc.expected {
			t.Errorf("isProductionEnvironment(%q) = %v, want %v", tc.env, got, tc.expected)
		}
	}
}

// newLogger のログレベルマッピングをテストする。
// 各レベル文字列が正しい slog.Level に変換されることを検証する。
func TestNewLoggerLevel(t *testing.T) {
	cases := []struct {
		level         string
		expectedLevel slog.Level
	}{
		{level: "debug", expectedLevel: slog.LevelDebug},
		{level: "info", expectedLevel: slog.LevelInfo},
		{level: "warn", expectedLevel: slog.LevelWarn},
		{level: "error", expectedLevel: slog.LevelError},
		// 不明な値はデフォルトの Info レベルを返す
		{level: "unknown", expectedLevel: slog.LevelInfo},
		{level: "", expectedLevel: slog.LevelInfo},
	}

	for _, tc := range cases {
		cfg := config.LogConfig{
			Level:  tc.level,
			Format: "json",
		}
		logger := newLogger(cfg)
		if logger == nil {
			t.Errorf("newLogger returned nil for level=%q", tc.level)
			continue
		}
		// SA1012: context.Background() を使用して nil コンテキストを回避する（§3.2 監査対応）
		if !logger.Enabled(context.Background(), tc.expectedLevel) {
			t.Errorf("newLogger(level=%q): expected level %v to be enabled", tc.level, tc.expectedLevel)
		}
	}
}

// newLogger の JSON/Text フォーマット切り替えをテストする。
// どちらのフォーマットでも logger が正常に生成されることを検証する。
func TestNewLoggerFormat(t *testing.T) {
	formats := []string{"json", "text", "JSON", "Text", "TEXT"}
	for _, format := range formats {
		cfg := config.LogConfig{
			Level:  "info",
			Format: format,
		}
		logger := newLogger(cfg)
		if logger == nil {
			t.Errorf("newLogger returned nil for format=%q", format)
		}
	}
}

// M-14 監査対応: initSessionStore のエラーパスと正常パスをテストする。
// カバレッジ 6% から向上させるための追加テスト。

// SESSION_ENCRYPTION_KEY が不正な hex 文字列の場合にエラーを返すことを確認する。
func TestInitSessionStore_InvalidHexKey(t *testing.T) {
	t.Setenv("SESSION_ENCRYPTION_KEY", "not-valid-hex!!")
	cfg := &config.BFFConfig{
		App:     config.AppConfig{Environment: "production"},
		Session: config.SessionConfig{Prefix: "test:"},
	}
	_, _, err := initSessionStore(cfg, nil, slog.Default())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "SESSION_ENCRYPTION_KEY")
}

// SESSION_ENCRYPTION_KEY が有効な hex だが 32 バイト未満の場合にエラーを返すことを確認する。
func TestInitSessionStore_InvalidKeyLength(t *testing.T) {
	// "aabbccdd" は 4 バイト（8 hex 文字）のみ
	t.Setenv("SESSION_ENCRYPTION_KEY", "aabbccdd")
	cfg := &config.BFFConfig{
		App:     config.AppConfig{Environment: "production"},
		Session: config.SessionConfig{Prefix: "test:"},
	}
	_, _, err := initSessionStore(cfg, nil, slog.Default())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "SESSION_ENCRYPTION_KEY")
}

// SESSION_ENCRYPTION_KEY が未設定で非開発環境（本番）の場合にエラーを返すことを確認する。
func TestInitSessionStore_MissingKeyProductionEnv(t *testing.T) {
	// テスト環境では SESSION_ENCRYPTION_KEY が設定されている可能性があるため、
	// 明示的に空文字列に設定して os.Getenv が "" を返すようにする
	os.Unsetenv("SESSION_ENCRYPTION_KEY") //nolint:errcheck
	t.Cleanup(func() { os.Unsetenv("SESSION_ENCRYPTION_KEY") }) //nolint:errcheck
	cfg := &config.BFFConfig{
		App:     config.AppConfig{Environment: "production"},
		Session: config.SessionConfig{Prefix: "test:"},
	}
	_, _, err := initSessionStore(cfg, nil, slog.Default())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "SESSION_ENCRYPTION_KEY")
}

// SESSION_ENCRYPTION_KEY が未設定でも dev 環境では正常に初期化できることを確認する。
func TestInitSessionStore_MissingKeyDevEnv(t *testing.T) {
	os.Unsetenv("SESSION_ENCRYPTION_KEY") //nolint:errcheck
	t.Cleanup(func() { os.Unsetenv("SESSION_ENCRYPTION_KEY") }) //nolint:errcheck
	cfg := &config.BFFConfig{
		App:     config.AppConfig{Environment: "dev"},
		Session: config.SessionConfig{Prefix: "test:"},
	}
	store, ttl, err := initSessionStore(cfg, nil, slog.Default())
	require.NoError(t, err)
	assert.NotNil(t, store)
	// TTL のデフォルト値（30分）が返されることを確認する
	assert.Equal(t, 30*time.Minute, ttl)
}

// 有効な 32 バイト hex キーを設定した場合に暗号化ストアが正常に初期化されることを確認する。
func TestInitSessionStore_ValidEncryptionKey(t *testing.T) {
	// 64 hex 文字 = 32 バイト AES-256 鍵
	t.Setenv("SESSION_ENCRYPTION_KEY", "e29fee9b9d9fef45b65e6265b11fb2bb6b1e34c301e0e929e2917a5344c20f28")
	cfg := &config.BFFConfig{
		App:     config.AppConfig{Environment: "production"},
		Session: config.SessionConfig{Prefix: "test:", TTL: "1h"},
	}
	store, ttl, err := initSessionStore(cfg, nil, slog.Default())
	require.NoError(t, err)
	assert.NotNil(t, store)
	assert.Equal(t, time.Hour, ttl)
}

// M-14 監査対応: initTracerProvider に Enabled=false を渡すと nil, nil を返すことを確認する。
// ネットワーク接続なしに実行できるユニットテスト。
func TestInitTracerProvider_Disabled(t *testing.T) {
	traceCfg := config.TraceConfig{Enabled: false}
	appCfg := config.AppConfig{Name: "test-service"}
	tp, err := initTracerProvider(context.Background(), traceCfg, appCfg)
	require.NoError(t, err)
	assert.Nil(t, tp)
}

// M-14 監査対応: Enabled=true の場合にトレーサープロバイダーが生成されることを確認する。
// gRPC は遅延接続のため、到達不能なエンドポイントでも初期化は成功する。
func TestInitTracerProvider_Enabled_LazyGRPC(t *testing.T) {
	traceCfg := config.TraceConfig{
		Enabled:      true,
		Endpoint:     "localhost:0", // 到達不能だが gRPC は遅延接続のためエラーにならない
		SampleRate:   1.0,
		OTLPInsecure: true, // テスト環境では TLS なし
	}
	appCfg := config.AppConfig{Name: "test-service"}
	tp, err := initTracerProvider(context.Background(), traceCfg, appCfg)
	// gRPC はレイジーコネクションのため到達不能エンドポイントでも初期化は成功する
	require.NoError(t, err)
	require.NotNil(t, tp)
	// リソースを解放する（シャットダウン時のエラーは無視）
	shutdownCtx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()
	_ = tp.Shutdown(shutdownCtx)
}

// M-14 監査対応: SampleRate が範囲外の場合でもクランプされて正常に動作することを確認する。
func TestInitTracerProvider_SampleRateClamp(t *testing.T) {
	// SampleRate < 0 と > 1 の両方をクランプするパスをカバーする
	for _, rate := range []float64{-1.0, 2.0} {
		traceCfg := config.TraceConfig{
			Enabled:      true,
			Endpoint:     "localhost:0",
			SampleRate:   rate,
			OTLPInsecure: true,
		}
		appCfg := config.AppConfig{Name: "test"}
		tp, err := initTracerProvider(context.Background(), traceCfg, appCfg)
		require.NoError(t, err)
		require.NotNil(t, tp)
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
		_ = tp.Shutdown(shutdownCtx)
		cancel()
	}
}

// M-14 監査対応: コンテキストがキャンセルされると retryOIDCDiscovery が速やかに終了することを確認する。
// 接続できない discovery URL で既にキャンセル済みのコンテキストを渡す。
func TestRetryOIDCDiscovery_ContextCancelled(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	// 即座にキャンセルしてループの最初の select で終了させる
	cancel()

	oauthClient := oauth.NewClient(
		context.Background(),
		"http://127.0.0.1:0/unreachable",
		"client-id", "secret",
		"http://localhost/callback",
		nil,
		// タイムアウトを短くして万が一接続が試みられた場合も速やかに失敗させる
		oauth.WithHTTPTimeout(100*time.Millisecond),
	)
	var ready atomic.Bool

	// goroutine で実行して timeout 付きで完了を待つ
	done := make(chan struct{})
	go func() {
		retryOIDCDiscovery(ctx, oauthClient, slog.Default(), &ready)
		close(done)
	}()

	select {
	case <-done:
		// 正常にキャンセルで終了した
	case <-time.After(2 * time.Second):
		t.Fatal("retryOIDCDiscovery はキャンセル済みコンテキストで速やかに終了するべきです")
	}
	// キャンセルで終了した場合は oidcReady は false のまま
	assert.False(t, ready.Load())
}

// M-14 監査対応: retryOIDCDiscovery が discovery 失敗後にコンテキストキャンセルで終了することを確認する。
// インターバル中のキャンセルで select が即座に ctx.Done() を選択することを検証する。
func TestRetryOIDCDiscovery_CancelDuringWait(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())

	oauthClient := oauth.NewClient(
		context.Background(),
		"http://127.0.0.1:0/unreachable",
		"client-id", "secret",
		"http://localhost/callback",
		nil,
		oauth.WithHTTPTimeout(50*time.Millisecond),
	)
	var ready atomic.Bool

	done := make(chan struct{})
	go func() {
		retryOIDCDiscovery(ctx, oauthClient, slog.Default(), &ready)
		close(done)
	}()

	// goroutine が起動してインターバル待機に入った後にキャンセルする
	// select の ctx.Done() が time.After より優先されて終了する
	time.Sleep(10 * time.Millisecond)
	cancel()

	select {
	case <-done:
		// コンテキストキャンセルで正常終了した
	case <-time.After(3 * time.Second):
		t.Fatal("retryOIDCDiscovery はコンテキストキャンセルで速やかに終了するべきです")
	}
	assert.False(t, ready.Load())
}
