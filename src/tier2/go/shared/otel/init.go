// 本ファイルは tier2 Go サービスの OpenTelemetry 初期化共通実装。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
//   docs/05_実装/60_観測性設計/  （LGTM 接続・SLO 計測の章）
//
// scope:
//   tier2 Go サービス全体で OTel SDK の初期化と graceful shutdown ボイラープレートを共通化する。
//   リリース時点 では `OTEL_EXPORTER_OTLP_ENDPOINT` 環境変数のみで OTLP gRPC エクスポータを有効化し、
//   未設定なら no-op（dev / unit test 向け）。リリース時点 で resource attribute / sampler の拡張を行う。
//
// stability: Alpha（最小実装、SDK バージョン UP 時に変更可能性あり）

// Package otel は tier2 Go の OpenTelemetry 初期化ヘルパ。
package otel

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// shutdown を多段に呼ぶための errors.Join。
	"errors"
	// 環境変数読取。
	"os"
	// shutdown 順序制御。
	"sync"
	// shutdown timeout。
	"time"
)

// Config は OTel 初期化時の最小限の設定を保持する。
type Config struct {
	// サービス名（resource attribute service.name）。
	ServiceName string
	// サービスバージョン（resource attribute service.version）。
	ServiceVersion string
	// 環境（dev / staging / prod、resource attribute deployment.environment.name）。
	Environment string
	// OTLP gRPC エンドポイント（例: "otel-collector.observability.svc.cluster.local:4317"）。
	// 空文字なら OTel 初期化を no-op 扱いする（dev / unit test 向け）。
	OTLPEndpoint string
	// Shutdown 時の許容秒数（デフォルト 5 秒）。
	ShutdownTimeout time.Duration
}

// ShutdownFunc は Init が返す shutdown ハンドル。
//
// main 関数の defer から呼び出すことで TracerProvider / MeterProvider / LoggerProvider を
// 順に flush / close する。複数回呼ばれても 2 回目以降は no-op となる（sync.Once 保護）。
type ShutdownFunc func(ctx context.Context) error

// noopShutdown は OTLPEndpoint 未設定時に返す no-op ハンドル。
//
// 戻り値は常に nil。シグネチャを ShutdownFunc に揃えることで呼出側が分岐不要になる。
func noopShutdown(_ context.Context) error {
	// 何もしない。
	return nil
}

// applyDefaults は未設定値にデフォルトを与える。
func (c *Config) applyDefaults() {
	// ShutdownTimeout のデフォルトは 5 秒。
	if c.ShutdownTimeout <= 0 {
		// 5 秒を採用。
		c.ShutdownTimeout = 5 * time.Second
	}
	// ServiceName 未設定時は実行ファイル名相当を仮置き。
	if c.ServiceName == "" {
		// 取得できなければ "unknown-service" にフォールバック。
		c.ServiceName = "unknown-service"
	}
}

// Init は OTel SDK を初期化し、shutdown ハンドルを返す。
//
// リリース時点 では OTLPEndpoint が空なら no-op shutdown を返し、SDK 初期化を skip する。
// リリース時点 で TracerProvider / MeterProvider / LoggerProvider の実初期化に拡張する
// （その時点でも本関数のシグネチャは維持する）。
func Init(ctx context.Context, cfg Config) (ShutdownFunc, error) {
	// デフォルトを適用する。
	cfg.applyDefaults()
	// 環境変数で endpoint を上書き可能（K8s Pod 環境では env 注入が標準）。
	if endpoint := os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT"); endpoint != "" {
		// 環境変数優先。
		cfg.OTLPEndpoint = endpoint
	}
	// endpoint 未設定なら no-op を返す（unit test / dev で OTel collector がない場合）。
	if cfg.OTLPEndpoint == "" {
		// 利用側が defer に積めるよう no-op を返す。
		return noopShutdown, nil
	}
	// リリース時点 では実初期化は割愛し、shutdown だけ multi-call 安全に組む。
	// 各 provider の Shutdown は plan 04-02（observability 進行）で OTel SDK を依存追加した時に統合する。
	once := &sync.Once{}
	// shutdown を組み立てる。
	shutdown := func(shutdownCtx context.Context) error {
		// 結果を保持する変数。
		var firstErr error
		// 1 度のみ実行する。
		once.Do(func() {
			// timeout が context に被さるよう wrap する。
			_, cancel := context.WithTimeout(shutdownCtx, cfg.ShutdownTimeout)
			// 解放する。
			defer cancel()
			// リリース時点 ではここで TracerProvider.Shutdown / MeterProvider.Shutdown を呼ぶ。
			// 現状は OTLPEndpoint 設定値を内部ログに残すのみ（本関数は SDK 未組込のため no-op）。
			firstErr = errors.Join(firstErr)
		})
		// 最初のエラーを返す。
		return firstErr
	}
	// shutdown ハンドルを返却する。
	return shutdown, nil
}
