// 本ファイルは tier1 Go の **t1-workflow Pod** の起動エントリポイント。
//
// 設計: docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/01_tier1全体配置.md
//       docs/02_構想設計/02_tier1設計/
// 関連 ID: IMP-DIR-002 / IMP-BUILD-002 / FR-T1-WORKFLOW / ADR-TIER1-001 / ADR-TIER1-003 / ADR-RULE-002
//
// 担当 API（docs 正典）:
//   - WorkflowService（k1s0.tier1.workflow.v1）
//
// scope:
//   - :50001 で listen（flag で上書き、docs 正典 EXPOSE 50001）
//   - 標準 gRPC health protocol 応答 + reflection
//   - SIGINT / SIGTERM で graceful shutdown
//   - Temporal adapter を環境変数で結線（TEMPORAL_HOSTPORT / TEMPORAL_NAMESPACE）
//   - Dapr Workflow adapter を環境変数で結線（DAPR_GRPC_ENDPOINT / DAPR_WORKFLOW_COMPONENT）
//   - 環境変数未設定時は in-memory backend で fallback（dev / CI 用途）
//
// production / dev / CI の挙動分岐:
//   - TEMPORAL_HOSTPORT が設定されている → 実 Temporal frontend に gRPC 接続して Workflow を扱う
//   - TEMPORAL_HOSTPORT が未設定           → InMemoryTemporal で起動（process 内 永続のみ）
//   - DAPR_GRPC_ENDPOINT が設定されている  → 実 Dapr sidecar の Workflow Beta1 API を叩く
//   - DAPR_GRPC_ENDPOINT が未設定          → InMemoryWorkflow で起動（process 内 永続のみ）

// パッケージ宣言。`go build ./cmd/workflow` で t1-workflow Pod 用バイナリを生成する。
package main

// 標準ライブラリと共通 runtime を import する。
import (
	// adapter 初期化に context を渡す。
	"context"
	// 依存先 probe error 比較用の errors.Is。
	"errors"
	// listen address を flag で受け取り、ConfigMap での上書きに備える。
	"flag"
	// 起動 / shutdown / エラーログを stderr に出す。
	"log"
	// HTTP/JSON 互換 gateway 用。
	"net/http"
	// 環境変数読出。
	"os"
	// SDK error 文字列フォールバック判定。
	"strings"
	// HealthService.Readiness の probe ごと timeout 制御。
	"time"

	// Dapr SDK Client（Workflow Beta1 API へ接続するために必要）。
	daprclient "github.com/dapr/go-sdk/client"
	// Dapr Workflow adapter（FR-T1-WORKFLOW-001 短期向け）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/daprwf"
	// Temporal adapter（長期向け）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/temporal"
	// 共通ランタイム（gRPC bootstrap + health + graceful shutdown）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// HealthService.Readiness 用 DependencyProbe 型。
	"github.com/k1s0/k1s0/src/tier1/go/internal/health"
	// t1-workflow Pod の handler（WorkflowService 単独）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/workflow"
)

// :50001 は docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md（EXPOSE 50001）正典準拠。
const defaultListen = ":50001"

// HTTP/JSON 互換 gateway の既定 listen address。
const defaultHTTPListen = ":50081"

// プロセスエントリポイント。flag パース、Temporal 結線、common.Run への委譲を行う。
func main() {
	// listen address の上書き flag を定義（既定 :50001）。
	addr := flag.String("listen", defaultListen, "gRPC server listen address")
	// HTTP/JSON 互換 gateway の listen address。空文字 / "off" で起動しない。
	httpAddr := flag.String("http-listen", defaultHTTPListen, "HTTP/JSON gateway listen address (empty or \"off\" disables)")
	// flag 解析を起動直後に確定させる。
	flag.Parse()

	// Temporal Client を環境変数または in-memory backend で構築する。
	temporalClient, err := newTemporalClient(context.Background())
	// 接続失敗（実 Temporal 経路のみ）は即時 exit(1)。
	if err != nil {
		// 失敗ログを stderr に書く。
		log.Fatalf("t1-workflow: temporal client init: %v", err)
	}
	// Pod 終了時に Client を解放する。
	defer func() {
		// Close エラーは ログのみ（exit code に影響させない）。
		if cerr := temporalClient.Close(); cerr != nil {
			// 失敗を stderr に残す。
			log.Printf("t1-workflow: temporal client close: %v", cerr)
		}
	}()

	// 短期 Workflow 用 Dapr Client を構築する（DAPR_GRPC_ENDPOINT 設定で production 経路）。
	daprWorkflowClient, daprWorkflowCloser, err := newDaprWorkflowAdapter()
	if err != nil {
		// Dapr SDK 初期化失敗は即時 exit(1)。
		log.Fatalf("t1-workflow: dapr workflow client init: %v", err)
	}
	// Pod 終了時に Client を解放する（in-memory backend なら no-op）。
	defer daprWorkflowCloser()

	// WorkflowService が依存する adapter を構築する。
	// 短期 = Dapr Workflow Beta1（DAPR_GRPC_ENDPOINT 経由 production / 未設定なら in-memory）、
	// 長期 = Temporal の 2 系統を並行注入し、Start handler が backend hint で振り分ける。
	deps := workflow.Deps{
		// Temporal（長期）。
		WorkflowAdapter: temporal.NewWorkflowAdapter(temporalClient),
		// Dapr Workflow（短期）。production / in-memory のどちらでも handler 側変更不要。
		DaprAdapter: daprWorkflowClient,
	}

	// HTTP/JSON 互換 gateway を別 goroutine で起動する（共通規約 §「HTTP/JSON 互換」）。
	httpServer := startWorkflowHTTPGatewayIfEnabled(*httpAddr, deps)
	defer func() {
		if httpServer != nil {
			shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()
			if err := httpServer.Shutdown(shutdownCtx); err != nil {
				log.Printf("t1-workflow: http gateway shutdown: %v", err)
			}
		}
	}()

	// Pod メタデータを構築する（WorkflowService 登録）。
	pod := common.Pod{
		// Pod 論理名。ログ出力で "tier1/workflow" として表示される。
		Name: "workflow",
		// 既定 listen address。
		DefaultListen: defaultListen,
		// service 登録 hook。WorkflowService を登録する。
		Register: workflow.Register(deps),
		// HealthService 用 Pod バージョン。release ビルドでは ldflags で上書きする想定。
		Version: common.DefaultVersion,
		// HealthService.Readiness で並列実行する依存先 probe。
		// workflow Pod は Temporal（長期）と Dapr Workflow（短期）の 2 系統に依存する。
		Probes: []health.DependencyProbe{
			{
				// dependencies map のキーは "temporal" を採用する。
				Name: "temporal",
				// Temporal Client の到達性を 2 秒以内で確認する。
				Check: func(ctx context.Context) error {
					// 過剰な待機を避けるため probe ごとに timeout を 2 秒に絞る。
					checkCtx, cancel := context.WithTimeout(ctx, 2*time.Second)
					// 関数末尾で必ず cancel する。
					defer cancel()
					// in-memory backend は常に nil、production は frontend に DescribeWorkflowExecution を投げる。
					return temporalClient.Ping(checkCtx)
				},
			},
			{
				// dependencies map のキーは "dapr-workflow" を採用する。
				Name: "dapr-workflow",
				// Dapr Workflow adapter は Ping を直接持たないため、センチネル workflow への
				// GetStatus を呼んで NotFound 応答で到達性を判定する。
				Check: func(ctx context.Context) error {
					// 過剰な待機を避けるため probe ごとに timeout を 2 秒に絞る。
					checkCtx, cancel := context.WithTimeout(ctx, 2*time.Second)
					// 関数末尾で必ず cancel する。
					defer cancel()
					// 実 sidecar / in-memory どちらでもセンチネル ID は存在しないため NotFound が返る期待。
					_, err := daprWorkflowClient.GetStatus(checkCtx, daprwf.GetStatusRequest{
						// テナント不問のセンチネル workflow ID。
						WorkflowID: "_k1s0_health_probe",
					})
					// nil（偶然存在）は到達 OK として扱う。
					if err == nil {
						// nil で reachable=true。
						return nil
					}
					// in-memory / production 双方の NotFound センチネルを到達 OK 扱い。
					if errors.Is(err, daprwf.ErrNotFound) {
						// nil で reachable=true。
						return nil
					}
					// 文字列 fallback（gRPC NotFound / "not found" 系）も到達 OK。
					if msg := err.Error(); strings.Contains(msg, "NotFound") || strings.Contains(msg, "not found") {
						// nil で reachable=true。
						return nil
					}
					// それ以外は network / 認証 / TLS 系エラー扱いで reachable=false に倒す。
					return err
				},
			},
		},
	}

	// 共通 runtime に委譲する。エラー時は log.Fatalf で非ゼロ終了する。
	if err := common.Run(pod, *addr); err != nil {
		// fatal log は stderr + exit(1) を 1 行で行う Go の慣用。
		log.Fatalf("t1-workflow: %v", err)
	}
}

// startWorkflowHTTPGatewayIfEnabled は HTTP/JSON 互換 gateway を別 goroutine で起動する。
// addr が空文字 / "off" なら起動せず nil を返す。
//
// 認証は本 gateway 単体では行わず、Service Mesh（Istio Ambient mTLS）または gRPC
// AuthInterceptor の前段配置で外部認証を担保する運用を前提とする（共通規約 §「認証と認可」）。
func startWorkflowHTTPGatewayIfEnabled(addr string, deps workflow.Deps) *http.Server {
	if addr == "" || addr == "off" {
		log.Printf("t1-workflow: HTTP/JSON gateway disabled (--http-listen=%q)", addr)
		return nil
	}
	g := common.NewHTTPGateway()
	g.RegisterWorkflowRoutes(workflow.MakeHTTPHandlers(workflow.NewWorkflowServiceServer(deps)))
	srv := &http.Server{
		Addr:              addr,
		Handler:           g.Handler(),
		ReadHeaderTimeout: 5 * time.Second,
	}
	go func() {
		log.Printf("t1-workflow: HTTP/JSON gateway listening on %s", addr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Printf("t1-workflow: http gateway: %v", err)
		}
	}()
	return srv
}

// newDaprWorkflowAdapter は環境変数 DAPR_GRPC_ENDPOINT の有無で
// 実 Dapr sidecar 結線 / in-memory backend を切替えて WorkflowAdapter を返す。
//
// 環境変数:
//   DAPR_GRPC_ENDPOINT      — Dapr sidecar gRPC アドレス（例: "localhost:50001"）
//   DAPR_WORKFLOW_COMPONENT — 使用する Workflow Component 名（既定: "dapr"）
//
// 戻り値の closer は Pod 終了時に必ず呼ぶ（in-memory backend では no-op）。
func newDaprWorkflowAdapter() (daprwf.WorkflowAdapter, func(), error) {
	// DAPR_GRPC_ENDPOINT が未設定なら in-memory backend を返す。
	addr := os.Getenv("DAPR_GRPC_ENDPOINT")
	if addr == "" {
		// stderr に fallback モードを 1 行ログする。
		log.Printf("t1-workflow: DAPR_GRPC_ENDPOINT not set, using in-memory Dapr Workflow backend (dev/CI mode)")
		// in-memory backend を返却する（closer は no-op）。
		return daprwf.NewInMemoryWorkflow(), func() {}, nil
	}
	// 実 Dapr sidecar に接続する。
	sdk, err := daprclient.NewClientWithAddress(addr)
	if err != nil {
		return nil, func() {}, err
	}
	// production 経路のログ。
	log.Printf("t1-workflow: dapr workflow backend = sidecar at %s", addr)
	// Component 名は環境変数で上書き可能（既定 "dapr"）。
	component := os.Getenv("DAPR_WORKFLOW_COMPONENT")
	// SDK の GRPCClient は daprWorkflowClient narrow interface を満たす。
	return daprwf.NewProduction(sdk, component), func() { sdk.Close() }, nil
}

// newTemporalClient は環境変数 TEMPORAL_HOSTPORT の有無で実 / in-memory を切替えて Client を生成する。
//
// 環境変数:
//   TEMPORAL_HOSTPORT  — Temporal frontend gRPC アドレス（例: "temporal-frontend.k1s0-data.svc:7233"）
//   TEMPORAL_NAMESPACE — Temporal namespace（既定: "k1s0"）
//
// hostPort が空文字の場合は in-memory backend を起動する。dev / CI で外部依存なしに
// `go run cmd/workflow` で gRPC ハンドラが実値（in-memory での workflow 状態）を返せる。
func newTemporalClient(ctx context.Context) (*temporal.Client, error) {
	// hostPort が未設定なら in-memory backend を起動する。
	hostPort := os.Getenv("TEMPORAL_HOSTPORT")
	// hostPort 未設定時は in-memory に fallback する。
	if hostPort == "" {
		// stderr に in-memory モード起動を 1 行ログする。
		log.Printf("t1-workflow: TEMPORAL_HOSTPORT not set, using in-memory backend (dev/CI mode)")
		// in-memory backend を返却する。
		return temporal.NewClientWithInMemory(), nil
	}
	// 実 Temporal frontend に接続する。
	return temporal.New(ctx, temporal.Config{
		// frontend gRPC アドレス。
		HostPort: hostPort,
		// namespace。
		Namespace: os.Getenv("TEMPORAL_NAMESPACE"),
	})
}
