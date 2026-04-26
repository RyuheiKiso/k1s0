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
//   - :50051 で listen（flag で上書き）
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
	"github.com/k1s0/k1s0/src/tier1/go/internal/server/runtime"
)

// :50051 は gRPC の de facto デフォルト。docs/02_構想設計/02_tier1設計/ で確定後 ConfigMap で上書き予定。
const defaultListen = ":50051"

// プロセスエントリポイント。flag パースと runtime.Run への委譲のみを行う。
func main() {
	// listen address の上書き flag を定義（既定 :50051、後で ConfigMap → envvar → flag の優先順で読む）。
	addr := flag.String("listen", defaultListen, "gRPC server listen address")
	// flag 解析を起動直後に確定させる。
	flag.Parse()

	// Pod メタデータを構築する（service 登録は plan 04-07 / 04-14 で実装）。
	pod := runtime.Pod{
		// Pod 論理名。ログ出力で "tier1/workflow" として表示される。
		Name: "workflow",
		// 既定 listen address。flag 未指定時に runtime.Run へ渡される値の参照用。
		DefaultListen: defaultListen,
		// service 登録 hook。Workflow handler 実装が揃うまで nil。
		Register: nil,
		// 構造体リテラルを閉じる。
	}

	// 共通 runtime に委譲する。エラー時は log.Fatalf で非ゼロ終了する。
	if err := runtime.Run(pod, *addr); err != nil {
		// fatal log は stderr + exit(1) を 1 行で行う Go の慣用。
		log.Fatalf("t1-workflow: %v", err)
		// if 分岐を閉じる。
	}
	// main 関数を閉じる。
}
