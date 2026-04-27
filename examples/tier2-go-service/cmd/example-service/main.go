// 本ファイルは tier2 Go サービスの Golden Path 完動例。
// k1s0 Go SDK の State 動詞統一 facade を使い、HTTP /sample-write endpoint で
// Valkey に書き込むサンプル。実際の業務サービスは internal/{domain,application,
// infrastructure} に分離するが、本 example は 1 ファイルで読み切れる構成にする。

// パッケージ宣言。`go build ./cmd/example-service` でバイナリを生成する。
package main

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// CLI flag 取得。
	"flag"
	// HTTP server 実装。
	"net/http"
	// 標準ログ出力。
	"log"
	// シグナル受信。
	"os"
	"os/signal"
	"syscall"
	// graceful shutdown のタイムアウト。
	"time"

	// k1s0 高水準 SDK facade。
	"github.com/k1s0/sdk-go/k1s0"
)

// プロセスエントリポイント。
func main() {
	// listen address の上書き flag を定義。
	addr := flag.String("listen", ":8080", "HTTP server listen address")
	// k1s0 tier1 facade の gRPC 接続先（dev 既定: localhost:50001）。
	tier1Target := flag.String("tier1-target", "localhost:50001", "tier1 facade gRPC target")
	// テナント識別。
	tenantID := flag.String("tenant-id", "tenant-example", "Tenant ID")
	subject := flag.String("subject", "tier2-example-service", "Subject")
	flag.Parse()

	// k1s0 SDK Client を生成する。
	client, err := k1s0.New(context.Background(), k1s0.Config{
		Target:   *tier1Target,
		TenantID: *tenantID,
		Subject:  *subject,
		// dev は平文（prod は UseTLS=true 必須）。
		UseTLS: false,
	})
	if err != nil {
		log.Fatalf("k1s0 sdk init: %v", err)
	}
	defer client.Close()

	// HTTP handler を組み立てる。
	mux := http.NewServeMux()

	// /healthz: 単純な疎通確認。
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		// 200 OK を返却する。
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})

	// /readyz: tier1 facade との疎通も含めた健全性確認（リリース時点は単純）。
	mux.HandleFunc("/readyz", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ready"))
	})

	// /sample-write: tier1 State API への書き込みサンプル（k1s0 SDK 利用デモ）。
	mux.HandleFunc("/sample-write", func(w http.ResponseWriter, r *http.Request) {
		// HTTP context を tier1 RPC にも伝搬する。
		ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
		defer cancel()
		// State.Save を呼び出す（valkey-default Store の "tier2-example/last-call" キーに current time を書く）。
		etag, err := client.State().Save(ctx, "valkey-default", "tier2-example/last-call",
			[]byte(time.Now().UTC().Format(time.RFC3339)))
		if err != nil {
			http.Error(w, "state save: "+err.Error(), http.StatusBadGateway)
			return
		}
		// 成功時は新 ETag を返却する。
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("saved, etag=" + etag))
	})

	// HTTP server を組み立てる。
	srv := &http.Server{
		Addr:    *addr,
		Handler: mux,
		// 平和なタイムアウト（DoS 軽減）。
		ReadHeaderTimeout: 5 * time.Second,
	}

	// 別 goroutine で Listen & Serve を起動する。
	errCh := make(chan error, 1)
	go func() {
		log.Printf("tier2-example-service: HTTP listening on %s", *addr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			errCh <- err
		}
	}()

	// シグナル待ち。
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	select {
	case sig := <-sigCh:
		log.Printf("received signal %s, shutting down", sig)
	case err := <-errCh:
		log.Fatalf("http server: %v", err)
	}

	// graceful shutdown（25s 上限、tier1 facade と整合）。
	ctx, cancel := context.WithTimeout(context.Background(), 25*time.Second)
	defer cancel()
	if err := srv.Shutdown(ctx); err != nil {
		log.Printf("graceful shutdown error: %v", err)
	}
	log.Printf("tier2-example-service: shutdown complete")
}
