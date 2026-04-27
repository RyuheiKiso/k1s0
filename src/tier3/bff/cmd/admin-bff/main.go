// admin-bff のエントリポイント。
//
// portal-bff と異なり、認可は role:admin を要求する（管理者のみアクセス可）。
// REST API のみ提供（GraphQL は portal-bff のみ）。

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
	// k1s0 SDK ラッパー。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/k1s0client"
	// REST router。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/rest"
	// shared OTel ヘルパ。
	sharedotel "github.com/k1s0/k1s0/src/tier3/bff/internal/shared/otel"
)

func main() {
	cfg, err := config.Load("admin-bff")
	if err != nil {
		log.Fatalf("admin-bff: failed to load config: %v", err)
	}
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()
	otelShutdown, err := sharedotel.Init(ctx, sharedotel.Config{
		ServiceName:    cfg.AppName,
		ServiceVersion: cfg.ServiceVersion,
		Environment:    cfg.Environment,
		OTLPEndpoint:   cfg.OTLPEndpoint,
	})
	if err != nil {
		log.Fatalf("admin-bff: failed to init otel: %v", err)
	}
	defer func() {
		shutdownCtx, cancel := context.WithCancel(context.Background())
		defer cancel()
		if shutdownErr := otelShutdown(shutdownCtx); shutdownErr != nil {
			log.Printf("admin-bff: otel shutdown error: %v", shutdownErr)
		}
	}()
	client, err := k1s0client.New(ctx, cfg.K1s0)
	if err != nil {
		log.Fatalf("admin-bff: failed to dial k1s0 facade: %v", err)
	}
	defer func() {
		if closeErr := client.Close(); closeErr != nil {
			log.Printf("admin-bff: k1s0 client close error: %v", closeErr)
		}
	}()
	mux := http.NewServeMux()
	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})
	mux.HandleFunc("GET /readyz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ready"))
	})
	// REST（認可: role=admin）。
	router := rest.NewRouter(client)
	restMux := http.NewServeMux()
	router.Register(restMux)
	mux.Handle("/api/", auth.Required("admin")(restMux))
	// HTTP server。
	srv := &http.Server{
		Addr:         cfg.HTTP.Addr,
		Handler:      mux,
		ReadTimeout:  time.Duration(cfg.HTTP.ReadTimeoutSec) * time.Second,
		WriteTimeout: time.Duration(cfg.HTTP.WriteTimeoutSec) * time.Second,
		IdleTimeout:  60 * time.Second,
	}
	errCh := make(chan error, 1)
	go func() {
		log.Printf("admin-bff: listening on %s", cfg.HTTP.Addr)
		err := srv.ListenAndServe()
		if err != nil && !errors.Is(err, http.ErrServerClosed) {
			errCh <- err
			close(errCh)
			return
		}
		errCh <- nil
		close(errCh)
	}()
	select {
	case err := <-errCh:
		if err != nil {
			log.Fatalf("admin-bff: listen error: %v", err)
		}
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		if shutdownErr := srv.Shutdown(shutdownCtx); shutdownErr != nil {
			log.Printf("admin-bff: shutdown error: %v", shutdownErr)
		}
	}
	_ = os.Stdout.Sync()
}
