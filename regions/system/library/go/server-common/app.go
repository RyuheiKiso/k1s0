package servercommon

import (
	"context"
	"fmt"
	"net/http"
	"time"
)

// defaultShutdownTimeout はグレースフルシャットダウンのデフォルトタイムアウト。
const defaultShutdownTimeout = 15 * time.Second

// AppConfig は K1s0App の設定を保持する。
type AppConfig struct {
	ServiceName string
	HTTPAddr    string
	GRPCAddr    string
}

// K1s0App はサービスの起動・停止を管理するアプリケーションビルダー。
// WithXxx メソッドで各コンポーネントを設定し、Build() で起動準備を完了する。
type K1s0App struct {
	cfg         AppConfig
	httpServer  *http.Server
	shutdownFns []func(context.Context) error
}

// NewApp は K1s0App を生成する。
func NewApp(cfg AppConfig) *K1s0App {
	return &K1s0App{cfg: cfg}
}

// WithHTTPHandler は HTTP ハンドラを設定する。
// デフォルトの /healthz, /readyz エンドポイントが自動的に追加される。
func (a *K1s0App) WithHTTPHandler(mux http.Handler) *K1s0App {
	addr := a.cfg.HTTPAddr
	if addr == "" {
		addr = ":8080"
	}
	a.httpServer = &http.Server{
		Addr:              addr,
		Handler:           mux,
		ReadHeaderTimeout: 10 * time.Second,
	}
	return a
}

// WithShutdownFn は追加のシャットダウン処理を登録する。
// DB 接続のクローズなどに使用する。
func (a *K1s0App) WithShutdownFn(fn func(context.Context) error) *K1s0App {
	a.shutdownFns = append(a.shutdownFns, fn)
	return a
}

// Run はサービスを起動し、ctx がキャンセルされるまで待機する。
// 終了時はグレースフルシャットダウンを実行する。
func (a *K1s0App) Run(ctx context.Context) error {
	errCh := make(chan error, 1)

	if a.httpServer != nil {
		go func() {
			if err := a.httpServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
				errCh <- fmt.Errorf("http server: %w", err)
			}
		}()
	}

	select {
	case <-ctx.Done():
	case err := <-errCh:
		return err
	}

	return a.shutdown()
}

// shutdown はグレースフルシャットダウンを実行する。
func (a *K1s0App) shutdown() error {
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()

	if a.httpServer != nil {
		if err := a.httpServer.Shutdown(ctx); err != nil {
			return fmt.Errorf("http server shutdown: %w", err)
		}
	}

	for _, fn := range a.shutdownFns {
		if err := fn(ctx); err != nil {
			return err
		}
	}
	return nil
}
