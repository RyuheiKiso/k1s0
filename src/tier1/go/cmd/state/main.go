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

	// Dapr adapter（State / PubSub / Binding / Invoke / Feature の 5 building block 共通 Client）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// 共通ランタイム（gRPC bootstrap + health + graceful shutdown）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// t1-state Pod の handler（5 公開 API のオーケストレータ）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/state"
)

// :50001 は docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md（EXPOSE 50001）正典準拠。
// Dapr sidecar 経由の app-port も 50001 を期待（dapr.io/app-port=50001）。
const defaultListen = ":50001"

// プロセスエントリポイント。flag パースと adapter 初期化、common.Run への委譲を行う。
func main() {
	// listen address の上書き flag を定義（既定 :50001、後で ConfigMap → envvar → flag の優先順で読む）。
	addr := flag.String("listen", defaultListen, "gRPC server listen address")
	// Dapr sidecar address の flag（既定は dapr.go 側 defaultDaprAddress、空文字で adapter 既定値を使う）。
	daprAddr := flag.String("dapr-address", "", "Dapr sidecar gRPC address (empty = use adapter default)")
	// flag 解析を起動直後に確定させる。
	flag.Parse()

	// Dapr Client を起動時に初期化する（lazy 不可、health check 開始時点で初期化済が docs 正典）。
	daprClient, err := dapr.New(context.Background(), dapr.Config{SidecarAddress: *daprAddr})
	// 初期化失敗は即時 exit(1)。
	if err != nil {
		// 失敗ログを stderr に書く。
		log.Fatalf("t1-state: dapr client init: %v", err)
		// if 分岐を閉じる。
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

	// Pod メタデータを構築する（5 API すべて Register hook で登録）。
	pod := common.Pod{
		// Pod 論理名。ログ出力で "tier1/state" として表示される。
		Name: "state",
		// 既定 listen address。
		DefaultListen: defaultListen,
		// service 登録 hook。5 公開 API（ServiceInvoke / State / PubSub / Binding / Feature）を登録する。
		Register: state.Register(deps),
		// 構造体リテラルを閉じる。
	}

	// 共通 runtime に委譲する。エラー時は log.Fatalf で非ゼロ終了する。
	if err := common.Run(pod, *addr); err != nil {
		// fatal log は stderr + exit(1) を 1 行で行う Go の慣用。
		log.Fatalf("t1-state: %v", err)
		// if 分岐を閉じる。
	}
	// main 関数を閉じる。
}
