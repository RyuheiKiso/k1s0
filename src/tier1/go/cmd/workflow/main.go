// 本ファイルは tier1 Go の **t1-workflow Pod** の起動エントリポイント。
//
// 設計: docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/01_tier1全体配置.md
//       docs/02_構想設計/02_tier1設計/
// 関連 ID: IMP-DIR-002 / IMP-BUILD-002 / FR-T1-WORKFLOW / ADR-TIER1-001 / ADR-TIER1-003
//
// 担当 API（docs 正典）:
//   - WorkflowService（k1s0.tier1.workflow.v1）
//
// scope（リリース時点最小骨格）:
//   - :50001 で listen（flag で上書き、docs 正典 EXPOSE 50001）
//   - 標準 gRPC health protocol 応答 + reflection
//   - SIGINT / SIGTERM で graceful shutdown
//
// 未実装（plan 04-07 / 04-14 で追加）:
//   - WorkflowService ハンドラ実装（Dapr Workflow / Temporal 振り分け）
//   - Temporal client（internal/temporal/）連携
//   - workflow 振り分け YAML（plan 04-14）

// パッケージ宣言。`go build ./cmd/workflow` で t1-workflow Pod 用バイナリを生成する。
package main

// 標準ライブラリと共通 runtime を import する。
import (
	// listen address を flag で受け取り、ConfigMap での上書きに備える。
	"flag"
	// 起動 / shutdown / エラーログを stderr に出す（OTel logger は plan 04-02 で導入）。
	"log"

	// 共通ランタイム（gRPC bootstrap + health + graceful shutdown）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// t1-workflow Pod の handler（WorkflowService 単独）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/workflow"
)

// :50001 は docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md（EXPOSE 50001）正典準拠。
// Dapr sidecar 経由の app-port も 50001 を期待（dapr.io/app-port=50001）。
const defaultListen = ":50001"

// プロセスエントリポイント。flag パースと common.Run への委譲のみを行う。
func main() {
	// listen address の上書き flag を定義（既定 :50001、後で ConfigMap → envvar → flag の優先順で読む）。
	addr := flag.String("listen", defaultListen, "gRPC server listen address")
	// flag 解析を起動直後に確定させる。
	flag.Parse()

	// Pod メタデータを構築する（WorkflowService 登録、Dapr Workflow / Temporal 振り分けは plan 04-07 / 04-14）。
	pod := common.Pod{
		// Pod 論理名。ログ出力で "tier1/workflow" として表示される。
		Name: "workflow",
		// 既定 listen address。flag 未指定時に common.Run へ渡される値の参照用。
		DefaultListen: defaultListen,
		// service 登録 hook。WorkflowService を登録。
		// production では `temporal.New(ctx, temporal.Config{HostPort: ..., Namespace: "k1s0"})` で
		// Client を作り、`workflow.Deps{WorkflowAdapter: temporal.NewWorkflowAdapter(client)}` を渡す。
		// 本最小骨格 main では adapter 未注入で動かす（後段の Temporal bootstrap で結線）。
		Register: workflow.Register(workflow.Deps{}),
		// 構造体リテラルを閉じる。
	}

	// 共通 runtime に委譲する。エラー時は log.Fatalf で非ゼロ終了する。
	if err := common.Run(pod, *addr); err != nil {
		// fatal log は stderr + exit(1) を 1 行で行う Go の慣用。
		log.Fatalf("t1-workflow: %v", err)
		// if 分岐を閉じる。
	}
	// main 関数を閉じる。
}
