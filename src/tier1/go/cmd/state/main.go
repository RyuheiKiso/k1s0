// 本ファイルは tier1 Go の **t1-state Pod** の起動エントリポイント。
//
// 設計: docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/01_tier1全体配置.md
//       docs/02_構想設計/02_tier1設計/
// 関連 ID: IMP-DIR-002 / IMP-BUILD-002 / DS-SW-COMP-023（5 API Router）/
//          FR-T1-INVOKE / FR-T1-STATE / FR-T1-PUBSUB / FR-T1-BINDING / FR-T1-FEATURE /
//          ADR-TIER1-001 / ADR-TIER1-003
//
// 担当 API（docs 正典 DS-SW-COMP-023、5 API 統合 Router Pod）:
//   - ServiceInvokeService（k1s0.tier1.serviceinvoke.v1）
//   - StateService（k1s0.tier1.state.v1）
//   - PubSubService（k1s0.tier1.pubsub.v1）
//   - BindingService（k1s0.tier1.binding.v1）
//   - FeatureService（k1s0.tier1.feature.v1）
//
// scope（リリース時点最小骨格）:
//   - :50001 で listen（flag で上書き、docs 正典 EXPOSE 50001）
//   - 標準 gRPC health protocol 応答 + reflection
//   - SIGINT / SIGTERM で graceful shutdown
//
// 未実装（plan 04-02 〜 04-13 で追加、5 API すべて本 Pod に登録予定）:
//   - ServiceInvokeService / StateService / PubSubService / BindingService / FeatureService ハンドラ実装
//   - Dapr Go SDK 経由の各 building block 接続
//   - OTel / retry / circuit-breaker / config

// パッケージ宣言。`go build ./cmd/state` で t1-state Pod 用バイナリを生成する。
package main

// 標準ライブラリと共通 runtime を import する。
import (
	// listen address を flag で受け取り、ConfigMap での上書きに備える。
	"flag"
	// 起動 / shutdown / エラーログを stderr に出す（OTel logger は plan 04-02 で導入）。
	"log"

	// 共通ランタイム（gRPC bootstrap + health + graceful shutdown）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
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

	// Pod メタデータを構築する（service 登録は plan 04-04 / 04-05 / 04-11 / 04-12 / 04-10 で実装、5 API）。
	pod := common.Pod{
		// Pod 論理名。ログ出力で "tier1/state" として表示される。
		Name: "state",
		// 既定 listen address。flag 未指定時に common.Run へ渡される値の参照用。
		DefaultListen: defaultListen,
		// service 登録 hook。5 API（ServiceInvoke / State / PubSub / Binding / Feature）handler 実装が揃うまで nil。
		Register: nil,
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
