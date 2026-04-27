// notification-hub サービスのエントリポイント。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   通知ハブ。HTTP `POST /notify` または PubSub `notification.requested` を受け、
//   テンプレ展開のうえ k1s0 Binding 経由で email / slack / webhook 等に配信する。
//   tier2 の代表的な「外部出力」サービス。

package main

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// log 出力。
	"log"
	// signal handling。
	"os"
	"os/signal"
	"syscall"

	// Application 層 UseCase。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/application/usecases"
	// Api 層 HTTP server。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/api"
	// Config ローダ。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/config"
	// Infrastructure 層（SDK ラッパー）。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/infrastructure/external"
	// shared OTel ヘルパ。
	sharedotel "github.com/k1s0/k1s0/src/tier2/go/shared/otel"
)

// main は DI 構築 + サーバ起動を担う。
func main() {
	// 設定をロードする。
	cfg, err := config.Load()
	// 設定不備は即終了する。
	if err != nil {
		// stderr に出して exit code 1。
		log.Fatalf("notification-hub: failed to load config: %v", err)
	}
	// graceful shutdown の context を組み立てる。
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	// 終了時に解放する。
	defer stop()
	// OpenTelemetry を初期化する。
	otelShutdown, err := sharedotel.Init(ctx, sharedotel.Config{
		ServiceName:    "notification-hub",
		ServiceVersion: cfg.ServiceVersion,
		Environment:    cfg.Environment,
		OTLPEndpoint:   cfg.OTLPEndpoint,
	})
	// 失敗は fail-fast。
	if err != nil {
		// log + exit。
		log.Fatalf("notification-hub: failed to init otel: %v", err)
	}
	// shutdown を defer する。
	defer func() {
		// shutdown 専用 context。
		shutdownCtx, cancel := context.WithCancel(context.Background())
		// 解放する。
		defer cancel()
		// flush。
		if shutdownErr := otelShutdown(shutdownCtx); shutdownErr != nil {
			// log のみ。
			log.Printf("notification-hub: otel shutdown error: %v", shutdownErr)
		}
	}()
	// k1s0 SDK Client を初期化する。
	k1s0Client, err := external.NewK1s0Client(ctx, cfg.K1s0)
	// 接続失敗は fail-fast。
	if err != nil {
		// log + exit。
		log.Fatalf("notification-hub: failed to dial k1s0 facade: %v", err)
	}
	// 接続を defer で閉じる。
	defer func() {
		// Close エラーは log のみ。
		if closeErr := k1s0Client.Close(); closeErr != nil {
			// 影響は限定的。
			log.Printf("notification-hub: k1s0 client close error: %v", closeErr)
		}
	}()
	// Application 層 UseCase を組み立てる。
	useCase := usecases.NewDispatchUseCase(k1s0Client, cfg.Bindings)
	// Api 層 HTTP server を起動する。
	server := api.NewServer(useCase, cfg.HTTP)
	// 起動ログ。
	log.Printf("notification-hub: listening on %s", cfg.HTTP.Addr)
	// HTTP server を起動する。
	if runErr := server.Run(ctx); runErr != nil {
		// 異常終了は exit code 1。
		log.Fatalf("notification-hub: server error: %v", runErr)
	}
	// 正常終了。
	_ = os.Stdout.Sync()
}
