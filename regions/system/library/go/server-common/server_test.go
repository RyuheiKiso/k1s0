package servercommon

import (
	"context"
	"net"
	"net/http"
	"testing"
	"time"
)

// NewServer が HTTP ハンドラー付きの Server を正しく構築できることを確認する。
func TestNewServerHTTPOnly(t *testing.T) {
	mux := http.NewServeMux()
	mux.HandleFunc("/test", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})

	cfg := ServerConfig{HTTPAddr: ":0"} // ランダムポートを使用する
	server := NewServer(cfg, mux, nil)

	if server == nil {
		t.Fatal("expected non-nil Server")
	}
	if server.httpServer == nil {
		t.Fatal("expected non-nil http.Server")
	}
	if server.grpcServer != nil {
		t.Error("expected nil grpcServer when not provided")
	}
}

// NewServer が HTTPAddr 未指定の場合にデフォルト ":8080" を設定することを確認する。
func TestNewServerDefaultAddr(t *testing.T) {
	cfg := ServerConfig{}
	server := NewServer(cfg, http.NewServeMux(), nil)

	if server.httpServer.Addr != ":8080" {
		t.Errorf("expected default addr ':8080', got '%s'", server.httpServer.Addr)
	}
}

// NewServer が gRPC サーバー付きで構築できることを確認する。
func TestNewServerWithGRPC(t *testing.T) {
	cfg := ServerConfig{
		HTTPAddr: ":0",
		GRPCAddr: ":0",
	}
	grpc := &mockGRPCServer{}
	server := NewServer(cfg, http.NewServeMux(), grpc)

	if server.grpcServer == nil {
		t.Error("expected grpcServer to be set")
	}
}

// mockGRPCServer は GRPCServable インターフェースのモック実装。
type mockGRPCServer struct {
	serveCalled       bool
	gracefulStopCalls int
}

func (m *mockGRPCServer) Serve(lis net.Listener) error {
	m.serveCalled = true
	// リスナーを閉じて即座にリターンする（テスト用）
	lis.Close()
	return nil
}

func (m *mockGRPCServer) GracefulStop() {
	m.gracefulStopCalls++
}

// Server.Run がコンテキストキャンセルでグレースフルにシャットダウンすることを確認する。
func TestServerRunAndShutdown(t *testing.T) {
	cfg := ServerConfig{HTTPAddr: ":0"}
	server := NewServer(cfg, http.NewServeMux(), nil)

	ctx, cancel := context.WithCancel(context.Background())

	errCh := make(chan error, 1)
	go func() {
		errCh <- server.Run(ctx)
	}()

	// サーバーが起動するまで少し待機する
	time.Sleep(50 * time.Millisecond)

	// コンテキストをキャンセルしてシャットダウンを開始する
	cancel()

	select {
	case err := <-errCh:
		if err != nil {
			t.Errorf("expected nil error on graceful shutdown, got %v", err)
		}
	case <-time.After(5 * time.Second):
		t.Fatal("timeout waiting for server shutdown")
	}
}

// K1s0App が NewApp で正しく構築されることを確認する。
func TestNewApp(t *testing.T) {
	cfg := AppConfig{
		ServiceName: "test-svc",
		HTTPAddr:    ":8080",
	}
	app := NewApp(cfg)
	if app == nil {
		t.Fatal("expected non-nil K1s0App")
	}
	if app.cfg.ServiceName != "test-svc" {
		t.Errorf("expected ServiceName 'test-svc', got '%s'", app.cfg.ServiceName)
	}
}

// WithHTTPHandler が HTTP サーバーを設定し、フルーエントに K1s0App を返すことを確認する。
func TestWithHTTPHandler(t *testing.T) {
	app := NewApp(AppConfig{})
	result := app.WithHTTPHandler(http.NewServeMux())
	if result != app {
		t.Error("expected WithHTTPHandler to return the same app instance (fluent)")
	}
	if app.httpServer == nil {
		t.Error("expected httpServer to be set")
	}
}

// WithHTTPHandler が HTTPAddr 未指定の場合にデフォルト ":8080" を設定することを確認する。
func TestWithHTTPHandlerDefaultAddr(t *testing.T) {
	app := NewApp(AppConfig{})
	app.WithHTTPHandler(http.NewServeMux())
	if app.httpServer.Addr != ":8080" {
		t.Errorf("expected default addr ':8080', got '%s'", app.httpServer.Addr)
	}
}

// WithHTTPHandler が明示的に指定された HTTPAddr を使用することを確認する。
func TestWithHTTPHandlerCustomAddr(t *testing.T) {
	app := NewApp(AppConfig{HTTPAddr: ":9090"})
	app.WithHTTPHandler(http.NewServeMux())
	if app.httpServer.Addr != ":9090" {
		t.Errorf("expected addr ':9090', got '%s'", app.httpServer.Addr)
	}
}

// WithShutdownFn がシャットダウン関数を追加しフルーエントに K1s0App を返すことを確認する。
func TestWithShutdownFn(t *testing.T) {
	app := NewApp(AppConfig{})
	result := app.WithShutdownFn(func(ctx context.Context) error {
		return nil
	})
	if result != app {
		t.Error("expected WithShutdownFn to return the same app instance (fluent)")
	}
	if len(app.shutdownFns) != 1 {
		t.Errorf("expected 1 shutdown function, got %d", len(app.shutdownFns))
	}
}

// K1s0App.Run がコンテキストキャンセルで正常にシャットダウンし、シャットダウン関数が呼ばれることを確認する。
func TestAppRunAndShutdown(t *testing.T) {
	shutdownCalled := false
	app := NewApp(AppConfig{HTTPAddr: ":0"})
	app.WithHTTPHandler(http.NewServeMux())
	app.WithShutdownFn(func(ctx context.Context) error {
		shutdownCalled = true
		return nil
	})

	ctx, cancel := context.WithCancel(context.Background())

	errCh := make(chan error, 1)
	go func() {
		errCh <- app.Run(ctx)
	}()

	// サーバーが起動するまで少し待機する
	time.Sleep(50 * time.Millisecond)

	cancel()

	select {
	case err := <-errCh:
		if err != nil {
			t.Errorf("expected nil error on graceful shutdown, got %v", err)
		}
	case <-time.After(5 * time.Second):
		t.Fatal("timeout waiting for app shutdown")
	}

	if !shutdownCalled {
		t.Error("expected shutdown function to be called")
	}
}
