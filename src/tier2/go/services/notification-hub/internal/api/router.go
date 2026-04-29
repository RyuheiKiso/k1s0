// HTTP server とルーティング。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

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

	// 設定。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/config"
	// dispatch UseCase。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/application/usecases"
	// 共通 JWT 認証 middleware（docs §共通規約「認証認可」、tier2 全 Go サービス共通）。
	t2auth "github.com/k1s0/k1s0/src/tier2/go/shared/auth"
)

// Server は HTTP server を保持する構造体。
type Server struct {
	// http.Server 実体。
	httpServer *http.Server
	// 設定。
	cfg config.HTTPConfig
}

// NewServer は HTTP server を構築する。
func NewServer(useCase *usecases.DispatchUseCase, cfg config.HTTPConfig) *Server {
	// 公開エンドポイント用の subrouter を組み立てる（auth middleware を必須にする）。
	authMux := http.NewServeMux()
	// dispatch handler を組み立てる。
	dh := newDispatchHandler(useCase)
	// /notify は JWT 必須。
	authMux.HandleFunc("POST /notify", dh.handleDispatch)
	// liveness / readiness は probe で auth 不要なので外側 mux に置く。
	mux := http.NewServeMux()
	// liveness probe。
	mux.HandleFunc("GET /healthz", handleLiveness)
	// readiness probe。
	mux.HandleFunc("GET /readyz", handleReadiness)
	// 公開エンドポイントは auth middleware で wrap する（docs §共通規約「認証認可」、
	// T2_AUTH_MODE 環境変数で off / hmac / jwks の 3 mode を選択）。
	mux.Handle("/notify", t2auth.Required()(authMux))
	// http.Server を組み立てる。
	srv := &http.Server{
		// listen address。
		Addr: cfg.Addr,
		// ServeMux を Handler に設定する。
		Handler: withRecover(mux),
		// read timeout。
		ReadTimeout: time.Duration(cfg.ReadTimeoutSec) * time.Second,
		// write timeout。
		WriteTimeout: time.Duration(cfg.WriteTimeoutSec) * time.Second,
		// idle timeout。
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
		// ErrServerClosed は graceful shutdown 経由なら正常扱い。
		if err != nil && !errors.Is(err, http.ErrServerClosed) {
			// listen 失敗は呼出元に通知する。
			errCh <- err
			// channel close で終了を伝える。
			close(errCh)
			// 関数終了。
			return
		}
		// 正常終了は nil。
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
	// 200 OK を返す。
	w.WriteHeader(http.StatusOK)
	// 本文。
	_, _ = w.Write([]byte("ok"))
}

// handleReadiness は readiness probe ハンドラ。
func handleReadiness(w http.ResponseWriter, _ *http.Request) {
	// 200 OK を返す。
	w.WriteHeader(http.StatusOK)
	// 本文。
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
				// 汎用エラーを返す（PII 漏洩防止）。
				_, _ = w.Write([]byte(`{"error":{"code":"E-T2-INTERNAL","message":"internal error"}}`))
			}
		}()
		// 後段 handler を呼ぶ。
		next.ServeHTTP(w, r)
	})
}
