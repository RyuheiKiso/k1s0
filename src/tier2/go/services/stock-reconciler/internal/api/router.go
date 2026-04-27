// HTTP server とルーティング。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   Application 層の UseCase を HTTP エンドポイントとして公開する。
//   chi / gin 等の外部依存を避け、リリース時点 では net/http 標準のみで構成する
//   （依存最小化と起動時間短縮を優先。リリース時点 で chi に移行）。

// Package api は HTTP / gRPC エンドポイントを集約する Api 層。
package api

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// 標準 errors。
	"errors"
	// HTTP server。
	"net/http"
	// timeout 設定。
	"time"

	// 設定（HTTP listen address / timeout）。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/config"
	// reconcile UseCase。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/application/usecases"
)

// Server は HTTP server を保持する構造体。
type Server struct {
	// http.Server 実体。
	httpServer *http.Server
	// 設定。
	cfg config.HTTPConfig
}

// NewServer は HTTP server を構築する。
func NewServer(useCase *usecases.ReconcileUseCase, cfg config.HTTPConfig) *Server {
	// ServeMux を組み立てる。
	mux := http.NewServeMux()
	// reconcile handler を組み立てる。
	rh := newReconcileHandler(useCase)
	// 公開エンドポイントを登録する。
	mux.HandleFunc("POST /reconcile/{sku}", rh.handleReconcile)
	// liveness probe（K8s 起動確認）。
	mux.HandleFunc("GET /healthz", handleLiveness)
	// readiness probe（接続準備確認、リリース時点 は単純化）。
	mux.HandleFunc("GET /readyz", handleReadiness)
	// http.Server を組み立てる。
	srv := &http.Server{
		// listen address。
		Addr: cfg.Addr,
		// ServeMux を Handler に設定する。
		Handler: withRecover(mux),
		// read timeout を秒数から duration に変換。
		ReadTimeout: time.Duration(cfg.ReadTimeoutSec) * time.Second,
		// write timeout を秒数から duration に変換。
		WriteTimeout: time.Duration(cfg.WriteTimeoutSec) * time.Second,
		// idle timeout は固定 60 秒。
		IdleTimeout: 60 * time.Second,
	}
	// Server 構造体を返す。
	return &Server{httpServer: srv, cfg: cfg}
}

// Run は HTTP server を起動し、ctx が cancel されたら graceful shutdown を試みる。
func (s *Server) Run(ctx context.Context) error {
	// 起動エラーを受信する channel。
	errCh := make(chan error, 1)
	// goroutine で listen する。
	go func() {
		// ListenAndServe で同期 block する。
		err := s.httpServer.ListenAndServe()
		// ErrServerClosed は graceful shutdown で正常扱い。
		if err != nil && !errors.Is(err, http.ErrServerClosed) {
			// listen 失敗は呼出元に通知する。
			errCh <- err
			// channel close で終了を伝える。
			close(errCh)
			// 関数終了。
			return
		}
		// 正常終了は nil を送る。
		errCh <- nil
		// channel を閉じる。
		close(errCh)
	}()
	// 終了 / シグナル待機を多重化する。
	select {
	// 起動エラー。
	case err := <-errCh:
		// listen 失敗をそのまま返す。
		return err
	// シグナル受信または親 context cancel。
	case <-ctx.Done():
		// graceful shutdown を試みる。
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		// timeout を解放する。
		defer cancel()
		// shutdown を実行する。
		if err := s.httpServer.Shutdown(shutdownCtx); err != nil {
			// shutdown 失敗は呼出元に通知。
			return err
		}
		// shutdown 完了。
		return nil
	}
}

// handleLiveness は liveness probe ハンドラ。
func handleLiveness(w http.ResponseWriter, _ *http.Request) {
	// 200 OK でバイトを返す。
	w.WriteHeader(http.StatusOK)
	// 本文 "ok"。
	_, _ = w.Write([]byte("ok"))
}

// handleReadiness は readiness probe ハンドラ。
//
// リリース時点 は liveness と同等。リリース時点 で k1s0 接続性チェックを追加する。
func handleReadiness(w http.ResponseWriter, _ *http.Request) {
	// 200 OK でバイトを返す。
	w.WriteHeader(http.StatusOK)
	// 本文 "ready"。
	_, _ = w.Write([]byte("ready"))
}

// withRecover はパニック復旧 middleware。
func withRecover(next http.Handler) http.Handler {
	// HandlerFunc を返す。
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// defer でパニックを補足する。
		defer func() {
			// recover で値を取得する。
			if rec := recover(); rec != nil {
				// 500 で応答する。
				w.WriteHeader(http.StatusInternalServerError)
				// クライアントに最小情報のみ返す（PII 漏洩防止）。
				_, _ = w.Write([]byte(`{"error":{"code":"E-T2-INTERNAL","message":"internal error"}}`))
			}
		}()
		// 後段 handler を呼ぶ。
		next.ServeHTTP(w, r)
	})
}
