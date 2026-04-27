// portal-bff のエントリポイント。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md

package main

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// errors。
	"errors"
	// log。
	"log"
	// HTTP。
	"net/http"
	// signal handling。
	"os"
	"os/signal"
	"syscall"
	// timeout。
	"time"

	// 認可 middleware。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/auth"
	// 設定。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/config"
	// GraphQL resolver。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/graphql"
	// k1s0 SDK ラッパー。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/k1s0client"
	// REST router。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/rest"
	// shared OTel ヘルパ。
	sharedotel "github.com/k1s0/k1s0/src/tier3/bff/internal/shared/otel"
)

// main は DI 構築 + サーバ起動。
func main() {
	// 設定をロードする。
	cfg, err := config.Load("portal-bff")
	if err != nil {
		log.Fatalf("portal-bff: failed to load config: %v", err)
	}
	// graceful shutdown 用の context。
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()
	// OTel 初期化。
	otelShutdown, err := sharedotel.Init(ctx, sharedotel.Config{
		ServiceName:    cfg.AppName,
		ServiceVersion: cfg.ServiceVersion,
		Environment:    cfg.Environment,
		OTLPEndpoint:   cfg.OTLPEndpoint,
	})
	if err != nil {
		log.Fatalf("portal-bff: failed to init otel: %v", err)
	}
	defer func() {
		shutdownCtx, cancel := context.WithCancel(context.Background())
		defer cancel()
		if shutdownErr := otelShutdown(shutdownCtx); shutdownErr != nil {
			log.Printf("portal-bff: otel shutdown error: %v", shutdownErr)
		}
	}()
	// k1s0 SDK Client を初期化する。
	client, err := k1s0client.New(ctx, cfg.K1s0)
	if err != nil {
		log.Fatalf("portal-bff: failed to dial k1s0 facade: %v", err)
	}
	defer func() {
		if closeErr := client.Close(); closeErr != nil {
			log.Printf("portal-bff: k1s0 client close error: %v", closeErr)
		}
	}()
	// HTTP mux を組み立てる。
	mux := http.NewServeMux()
	// liveness / readiness は認可不要で公開する。
	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})
	mux.HandleFunc("GET /readyz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ready"))
	})
	// GraphQL（認証必須）。
	resolver := graphql.NewResolver(client)
	mux.Handle("POST /graphql", auth.Required("user")(resolver.Handler()))
	// REST（認証必須）。
	router := rest.NewRouter(client)
	// REST ルートを別の mux にいったん登録してから auth でラップする。
	restMux := http.NewServeMux()
	router.Register(restMux)
	mux.Handle("/api/", auth.Required("user")(restMux))
	// HTTP server を組み立てる。
	srv := &http.Server{
		Addr:         cfg.HTTP.Addr,
		Handler:      mux,
		ReadTimeout:  time.Duration(cfg.HTTP.ReadTimeoutSec) * time.Second,
		WriteTimeout: time.Duration(cfg.HTTP.WriteTimeoutSec) * time.Second,
		IdleTimeout:  60 * time.Second,
	}
	// 起動 goroutine。
	errCh := make(chan error, 1)
	go func() {
		log.Printf("portal-bff: listening on %s", cfg.HTTP.Addr)
		err := srv.ListenAndServe()
		if err != nil && !errors.Is(err, http.ErrServerClosed) {
			errCh <- err
			close(errCh)
			return
		}
		errCh <- nil
		close(errCh)
	}()
	// 終了待機。
	select {
	case err := <-errCh:
		if err != nil {
			log.Fatalf("portal-bff: listen error: %v", err)
		}
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		if shutdownErr := srv.Shutdown(shutdownCtx); shutdownErr != nil {
			log.Printf("portal-bff: shutdown error: %v", shutdownErr)
		}
	}
	_ = os.Stdout.Sync()
}
