// 本ファイルは tier1 Go の **t1-state Pod** の起動エントリポイント。
//
// 設計: docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/01_tier1全体配置.md
//       docs/02_構想設計/02_tier1設計/
// 関連 ID: IMP-DIR-002 / IMP-BUILD-002 / DS-SW-COMP-023（5 Dapr 系 API Router）/
//          DS-SW-COMP-037 / 038（Log Adapter / Metrics Emitter は同 Pod 常駐）/
//          FR-T1-INVOKE / FR-T1-STATE / FR-T1-PUBSUB / FR-T1-BINDING / FR-T1-FEATURE /
//          FR-T1-LOG / FR-T1-TELEMETRY / ADR-TIER1-001 / ADR-TIER1-003
//
// 担当 API（src/tier1/README.md の Pod 構成表に従い 7 公開 API）:
//   - ServiceInvokeService（k1s0.tier1.serviceinvoke.v1）— Dapr 系
//   - StateService（k1s0.tier1.state.v1）— Dapr 系
//   - PubSubService（k1s0.tier1.pubsub.v1）— Dapr 系
//   - BindingService（k1s0.tier1.binding.v1）— Dapr 系
//   - FeatureService（k1s0.tier1.feature.v1）— flagd 直結
//   - LogService（k1s0.tier1.log.v1）— OTel Collector 直結（plan 04-13）
//   - TelemetryService（k1s0.tier1.telemetry.v1）— OTel Collector 直結（plan 04-13）
//
// scope（リリース時点最小骨格）:
//   - :50001 で listen（flag で上書き、docs 正典 EXPOSE 50001）
//   - 標準 gRPC health protocol 応答 + reflection
//   - SIGINT / SIGTERM で graceful shutdown
//
// 未実装（plan 04-02 〜 04-13 で追加、7 API すべて本 Pod に登録済 skeleton）:
//   - 5 Dapr 系 handler の実 building block 接続（plan 04-04 〜 04-12）
//   - flagd への Feature Flag 評価（plan 04-13）
//   - OTel Collector への Log / Telemetry 流出経路（plan 04-13）
//   - retry / circuit-breaker / config（plan 04-02）

// パッケージ宣言。`go build ./cmd/state` で t1-state Pod 用バイナリを生成する。
package main

// 標準ライブラリと共通 runtime を import する。
import (
	// 起動コンテキスト（adapter 初期化に渡す）。
	"context"
	// listen address を flag で受け取り、ConfigMap での上書きに備える。
	"flag"
	// 起動 / shutdown / エラーログを stderr に出す（OTel logger は plan 04-02 で導入）。
	"log"
	// HTTP server bootstrap（HTTP/JSON 互換 gateway 用）。
	"net/http"
	// stdout への OTel JSON Lines 出力先。
	"os"
	// HealthService.Readiness の probe ごと timeout 制御に使う。
	"time"

	// Dapr adapter（State / PubSub / Binding / Invoke / Feature の 5 building block 共通 Client）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// 共通ランタイム（gRPC bootstrap + health + graceful shutdown）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// HealthService.Readiness 用 DependencyProbe 型。
	"github.com/k1s0/k1s0/src/tier1/go/internal/health"
	// 共通 OTel emitter（Log / Metric / Trace）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/otel"
	// t1-state Pod の handler（5 公開 API のオーケストレータ）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/state"
)

// :50001 は docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md（EXPOSE 50001）正典準拠。
// Dapr sidecar 経由の app-port も 50001 を期待（dapr.io/app-port=50001）。
const defaultListen = ":50001"

// HTTP/JSON 互換 gateway の既定 listen address（共通規約 §「HTTP/JSON 互換」）。
// gRPC 50001 とポート分離して同 Pod に同居させる（Service 側で多 port 公開する想定）。
// 環境変数 / flag が空文字なら HTTP gateway を起動しない（gRPC 経路のみで運用する選択も許容）。
const defaultHTTPListen = ":50081"

// envOrDefault は env で上書き可能な flag.String 既定値を返す（Helm から env 経由で渡せるように）。
func envOrDefault(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

// プロセスエントリポイント。flag パースと adapter 初期化、common.Run への委譲を行う。
func main() {
	// listen address の上書き flag を定義（既定 :50001、後で ConfigMap → envvar → flag の優先順で読む）。
	addr := flag.String("listen", defaultListen, "gRPC server listen address")
	// HTTP/JSON 互換 gateway の listen address。空文字 / "off" で起動しない。
	httpAddr := flag.String("http-listen", envOrDefault("TIER1_HTTP_LISTEN_ADDR", defaultHTTPListen), "HTTP/JSON gateway listen address (empty or \"off\" disables)")
	// Dapr sidecar address の flag（空文字なら DAPR_GRPC_ENDPOINT 環境変数を参照）。
	daprAddr := flag.String("dapr-address", "", "Dapr sidecar gRPC address (empty = use DAPR_GRPC_ENDPOINT env or in-memory backend)")
	// flag 解析を起動直後に確定させる。
	flag.Parse()

	// Dapr Client を環境変数 / flag / in-memory backend から構築する。
	// production: DAPR_GRPC_ENDPOINT=localhost:50001 を設定すると実 sidecar に接続する。
	// dev / CI: 環境変数 / flag 未設定なら in-memory backend で fallback（外部依存なしで起動可）。
	daprClient, err := newDaprClient(context.Background(), *daprAddr)
	// 初期化失敗は即時 exit(1)。
	if err != nil {
		// 失敗ログを stderr に書く。
		log.Fatalf("t1-state: dapr client init: %v", err)
	}
	// Pod 終了時に Client を解放する。
	defer func() {
		// Close エラーは ログのみ（exit code に影響させない）。
		if cerr := daprClient.Close(); cerr != nil {
			// 失敗を stderr に残す。
			log.Printf("t1-state: dapr client close: %v", cerr)
			// if 分岐を閉じる。
		}
		// defer 関数を閉じる。
	}()

	// 5 公開 API の handler が依存する adapter 集合を構築する。
	deps := state.NewDepsFromClient(daprClient)
	// OTel Log / Metric / Trace の 3 emitter を環境変数判定で stdout / OTLP gRPC のどちらかで構築する。
	// docs 正典 DS-SW-COMP-037（"stdout JSON Lines / OTel Collector / Loki 集約"）と
	// DS-SW-COMP-038（Metrics Emitter）の両経路を満たす:
	//   - OTEL_EXPORTER_OTLP_ENDPOINT 未設定 → stdout JSON Lines（dev / CI / fluentbit 経路）
	//   - OTEL_EXPORTER_OTLP_ENDPOINT 設定済 → OTLP gRPC で Collector 直送
	bundle := otel.NewBundle(context.Background())
	// Pod 終了時に OTLP gRPC Provider を flush + close する（stdout 経路では no-op）。
	defer func() {
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if shutdownErr := bundle.Shutdown(shutdownCtx); shutdownErr != nil {
			log.Printf("t1-state: otel bundle shutdown: %v", shutdownErr)
		}
	}()
	// deps に 3 emitter を注入する（同名フィールド代入）。
	deps.LogEmitter = bundle.LogEmitter
	deps.MetricEmitter = bundle.MetricEmitter
	deps.TraceEmitter = bundle.TraceEmitter

	// HTTP/JSON 互換 gateway を別 goroutine で起動する（共通規約 §「HTTP/JSON 互換」）。
	// flag が空文字 / "off" なら HTTP server を起動しない（純 gRPC 運用への切替）。
	httpServer := startHTTPGatewayIfEnabled(*httpAddr, deps)
	// Pod 終了時に HTTP server も graceful shutdown する。
	defer func() {
		if httpServer != nil {
			shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()
			if err := httpServer.Shutdown(shutdownCtx); err != nil {
				log.Printf("t1-state: http gateway shutdown: %v", err)
			}
		}
	}()

	// Pod メタデータを構築する（5 API すべて Register hook で登録）。
	pod := common.Pod{
		// Pod 論理名。ログ出力で "tier1/state" として表示される。
		Name: "state",
		// 既定 listen address。
		DefaultListen: defaultListen,
		// service 登録 hook。5 公開 API（ServiceInvoke / State / PubSub / Binding / Feature）を登録する。
		Register: state.Register(deps),
		// HealthService 用 Pod バージョン。release ビルドでは ldflags で上書きする想定。
		Version: common.DefaultVersion,
		// HealthService.Readiness で並列実行する依存先 probe。
		// dev/CI（in-memory backend）では Dapr Client は process 内 backend に接続済のため
		// 常に reachable として ping を返す。production の Dapr sidecar 経路でも
		// daprClient.Ping は SDK 側 grpc.Health.Check に転送されるため一貫した結線。
		Probes: []health.DependencyProbe{
			{
				// dependencies map のキーは "dapr" を採用する。
				Name: "dapr",
				// Dapr Client の到達性を 2 秒以内で確認する。
				Check: func(ctx context.Context) error {
					// 過剰な待機を避けるため probe ごとに timeout を 2 秒に絞る。
					checkCtx, cancel := context.WithTimeout(ctx, 2*time.Second)
					// 関数末尾で必ず cancel する。
					defer cancel()
					// dapr.Client.Ping は in-memory backend では即時 nil、production では
					// sidecar に grpc Ping を投げる薄いラッパ。
					return daprClient.Ping(checkCtx)
				},
			},
		},
	}

	// 共通 runtime に委譲する。エラー時は log.Fatalf で非ゼロ終了する。
	if err := common.Run(pod, *addr); err != nil {
		// fatal log は stderr + exit(1) を 1 行で行う Go の慣用。
		log.Fatalf("t1-state: %v", err)
	}
}

// startHTTPGatewayIfEnabled は HTTP/JSON 互換 gateway を別 goroutine で起動する。
// addr が空文字 / "off" なら起動せず nil を返す（純 gRPC 運用）。
//
// 起動する route 集合（Go 側 9 API・25 unary RPC）:
//   - State / PubSub / Binding / Feature / Log / Telemetry / ServiceInvoke
//
// 認証は本 gateway 単体では行わず、Service Mesh（Istio Ambient mTLS）または gRPC AuthInterceptor
// の前段配置で外部認証を担保する運用を前提とする（共通規約 §「認証と認可」）。
// release-initial では Pod-internal 経路として外部公開しない設計が望ましい。
func startHTTPGatewayIfEnabled(addr string, deps state.Deps) *http.Server {
	if addr == "" || addr == "off" {
		log.Printf("t1-state: HTTP/JSON gateway disabled (--http-listen=%q)", addr)
		return nil
	}
	// gRPC server と同じ interceptor chain を HTTP gateway にも適用する
	// （共通規約 §「認証と認可」/§「監査と痕跡」/§「レート制限とクォータ」が
	//  HTTP / gRPC で同一に効くようにする）。
	// AuditInterceptor も含めて 4 段全部適用する。grpc / http 経路で別 emitter
	// instance を生成するため TIER1_AUDIT_MODE=grpc 時は 2 connection 発生するが、
	// release-initial では機能性を優先（最適化は別 PR）。
	g := common.NewHTTPGateway().WithInterceptors(
		common.AuthInterceptor(common.LoadAuthConfigFromEnv()),
		common.RateLimitInterceptor(common.LoadRateLimitConfigFromEnv()),
		common.ObservabilityInterceptor(),
		common.AuditInterceptor(common.LoadAuditEmitterFromEnv()),
	)
	// 9 API の route を一括登録する（unary RPC のみ、stream は gRPC 経路）。
	g.RegisterStateRoutes(state.MakeHTTPHandlers(state.NewStateServiceServer(deps)))
	g.RegisterPubSubRoutes(state.MakeHTTPPubSubHandlers(state.NewPubSubServiceServer(deps)))
	g.RegisterFeatureRoutes(state.MakeHTTPFeatureHandlers(
		state.NewFeatureServiceServer(deps),
		state.NewFeatureAdminServiceServer(deps.FeatureRegistry),
	))
	g.RegisterBindingRoutes(state.MakeHTTPBindingHandlers(state.NewBindingServiceServer(deps)))
	g.RegisterLogRoutes(state.MakeHTTPLogHandlers(state.NewLogServiceServer(deps)))
	g.RegisterTelemetryRoutes(state.MakeHTTPTelemetryHandlers(state.NewTelemetryServiceServer(deps)))
	g.RegisterInvokeRoutes(state.MakeHTTPInvokeHandlers(state.NewInvokeServiceServer(deps)))

	srv := &http.Server{
		Addr:              addr,
		Handler:           g.Handler(),
		ReadHeaderTimeout: 5 * time.Second,
	}
	go func() {
		log.Printf("t1-state: HTTP/JSON gateway listening on %s", addr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Printf("t1-state: http gateway: %v", err)
		}
	}()
	return srv
}

// newDaprClient は flag / DAPR_GRPC_ENDPOINT / in-memory の優先順で Dapr Client を構築する。
//
// 優先順位:
//   1. -dapr-address flag が指定されている → 実 Dapr sidecar に接続する
//   2. DAPR_GRPC_ENDPOINT 環境変数が設定されている → 実 Dapr sidecar に接続する
//   3. いずれも未設定 → in-memory backend で起動する（dev / CI 用途）
//
// production の Pod では Dapr sidecar が injection されているため、env を必ず設定する。
// dev / CI で外部依存なしに `go run cmd/state` できるよう、未設定時は in-memory に fallback する。
func newDaprClient(ctx context.Context, flagAddr string) (*dapr.Client, error) {
	// flag が最優先（明示指定）。
	addr := flagAddr
	// flag 未指定なら環境変数を参照する。
	if addr == "" {
		// 環境変数を取得する。
		addr = os.Getenv("DAPR_GRPC_ENDPOINT")
	}
	// addr 未設定時は in-memory backend を返す。
	if addr == "" {
		// stderr に fallback モードを 1 行ログする。
		log.Printf("t1-state: DAPR_GRPC_ENDPOINT not set, using in-memory Dapr backend (dev/CI mode)")
		// in-memory backend を返却する。
		return dapr.NewClientWithInMemoryBackends(), nil
	}
	// 実 Dapr sidecar に接続する。
	return dapr.New(ctx, dapr.Config{SidecarAddress: addr})
}
