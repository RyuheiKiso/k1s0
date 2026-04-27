// stock-reconciler サービスのエントリポイント。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   在庫の差分検出 / 同期バッチ。HTTP `POST /reconcile/{sku}` を受け、k1s0 State から
//   現在の在庫値を読み、外部システム差分を計算し、PubSub `stock.reconciled` を発火する。
//   ユースケース層で k1s0 SDK を経由するため tier1 facade の語彙のみに依存する。

package main

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// log 出力（リリース時点 は標準 log、リリース時点 で OTel logger に切替）。
	"log"
	// signal handling。
	"os"
	"os/signal"
	"syscall"

	// Application 層 UseCase。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/application/usecases"
	// Api 層 HTTP server。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/api"
	// Config ローダ。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/config"
	// Infrastructure 層（Repository 実装 / 外部 client）。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/infrastructure/external"
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/infrastructure/persistence"
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
		log.Fatalf("stock-reconciler: failed to load config: %v", err)
	}
	// graceful shutdown の context を組み立てる。
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	// 終了時に解放する。
	defer stop()
	// OpenTelemetry を初期化する（OTLP endpoint 未設定なら no-op）。
	otelShutdown, err := sharedotel.Init(ctx, sharedotel.Config{
		ServiceName:    "stock-reconciler",
		ServiceVersion: cfg.ServiceVersion,
		Environment:    cfg.Environment,
		OTLPEndpoint:   cfg.OTLPEndpoint,
	})
	// OTel 初期化失敗時は exit code 1。
	if err != nil {
		// 観測性は重要なため fail-fast。
		log.Fatalf("stock-reconciler: failed to init otel: %v", err)
	}
	// 終了時に flush する。
	defer func() {
		// shutdown 専用 context（main の context が cancel 済でも flush するため）。
		shutdownCtx, cancel := context.WithCancel(context.Background())
		// 解放する。
		defer cancel()
		// flush を実施する。
		if shutdownErr := otelShutdown(shutdownCtx); shutdownErr != nil {
			// shutdown エラーは exit code を変えず log のみに残す。
			log.Printf("stock-reconciler: otel shutdown error: %v", shutdownErr)
		}
	}()
	// k1s0 SDK Client を初期化する（Infrastructure 層に注入）。
	k1s0Client, err := external.NewK1s0Client(ctx, cfg.K1s0)
	// 接続失敗は fail-fast。
	if err != nil {
		// log + exit。
		log.Fatalf("stock-reconciler: failed to dial k1s0 facade: %v", err)
	}
	// 接続を defer で閉じる。
	defer func() {
		// Close エラーは log のみ。
		if closeErr := k1s0Client.Close(); closeErr != nil {
			// 影響は限定的のため warn 相当。
			log.Printf("stock-reconciler: k1s0 client close error: %v", closeErr)
		}
	}()
	// Domain Repository 実装を構築する（リリース時点 は in-memory + k1s0 State 二段 backing）。
	repo := persistence.NewK1s0StateRepository(k1s0Client, cfg.K1s0.StoreName)
	// Application 層 UseCase を組み立てる。
	useCase := usecases.NewReconcileUseCase(repo, k1s0Client, cfg.PubSub)
	// Api 層の HTTP server を起動する。
	server := api.NewServer(useCase, cfg.HTTP)
	// 起動ログ。
	log.Printf("stock-reconciler: listening on %s", cfg.HTTP.Addr)
	// HTTP server を起動する（ctx が cancel されるまでブロック）。
	if runErr := server.Run(ctx); runErr != nil {
		// 異常終了時は exit code 1。
		log.Fatalf("stock-reconciler: server error: %v", runErr)
	}
	// 正常終了。
	_ = os.Stdout.Sync()
}
