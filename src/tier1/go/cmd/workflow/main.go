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
//   - 環境変数未設定時は in-memory backend で fallback（dev / CI 用途）
//
// production / dev / CI の挙動分岐:
//   - TEMPORAL_HOSTPORT が設定されている → 実 Temporal frontend に gRPC 接続して Workflow を扱う
//   - TEMPORAL_HOSTPORT が未設定           → InMemoryTemporal で起動（process 内 永続のみ）

// パッケージ宣言。`go build ./cmd/workflow` で t1-workflow Pod 用バイナリを生成する。
package main

// 標準ライブラリと共通 runtime を import する。
import (
	// adapter 初期化に context を渡す。
	"context"
	// listen address を flag で受け取り、ConfigMap での上書きに備える。
	"flag"
	// 起動 / shutdown / エラーログを stderr に出す。
	"log"
	// 環境変数読出。
	"os"

	// Temporal adapter（本 Pod 専用）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/temporal"
	// 共通ランタイム（gRPC bootstrap + health + graceful shutdown）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// t1-workflow Pod の handler（WorkflowService 単独）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/workflow"
)

// :50001 は docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md（EXPOSE 50001）正典準拠。
const defaultListen = ":50001"

// プロセスエントリポイント。flag パース、Temporal 結線、common.Run への委譲を行う。
func main() {
	// listen address の上書き flag を定義（既定 :50001）。
	addr := flag.String("listen", defaultListen, "gRPC server listen address")
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

	// WorkflowService が依存する adapter を構築する。
	deps := workflow.Deps{
		// Temporal Client から WorkflowAdapter を生成する。
		WorkflowAdapter: temporal.NewWorkflowAdapter(temporalClient),
	}

	// Pod メタデータを構築する（WorkflowService 登録）。
	pod := common.Pod{
		// Pod 論理名。ログ出力で "tier1/workflow" として表示される。
		Name: "workflow",
		// 既定 listen address。
		DefaultListen: defaultListen,
		// service 登録 hook。WorkflowService を登録する。
		Register: workflow.Register(deps),
	}

	// 共通 runtime に委譲する。エラー時は log.Fatalf で非ゼロ終了する。
	if err := common.Run(pod, *addr); err != nil {
		// fatal log は stderr + exit(1) を 1 行で行う Go の慣用。
		log.Fatalf("t1-workflow: %v", err)
	}
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
