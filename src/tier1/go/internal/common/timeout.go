// 本ファイルは tier1 Go の timeout 階層定数と context.WithTimeout の慣用形。
//
// 設計: plan/04_tier1_Goファサード実装/02_共通基盤.md（主作業 5）
//       SLO 予算: 業務 200ms + Dapr 80ms + OTel 20ms + 監査 50ms + net/DB 150ms = 500ms / API
// 関連 ID: NFR-B-PERF-* / IMP-RELIABILITY-*
//
// scope（リリース時点最小骨格）:
//   - 階層 timeout 定数の定義（FacadeBudget / DaprCall / RustCoreCall）
//   - WithFacadeTimeout helper の最小実装
//
// 未実装（plan 04-02 主作業 5 で追加、次セッション以降）:
//   - context.WithTimeout の自動継承（親 context の deadline と min を取る）
//   - hop-by-hop deadline propagation（gRPC metadata で deadline を伝搬）

package common

// 標準ライブラリのみ import（minimal skeleton、third-party 依存なし）。
import (
	// timeout 制御用の context.
	"context"
	// 時間定数定義に time.Duration を使う。
	"time"
)

// 階層 timeout 定数。SLO 予算 500ms / API の内訳に基づく（plan 04-02 主作業 5）。
const (
	// FacadeBudget はファサード層全体の SLO 予算（業務 200 + Dapr 80 + OTel 20 + 監査 50 + net/DB 150）。
	FacadeBudget = 500 * time.Millisecond
	// DaprCall は Dapr sidecar 経由の単一呼び出しの上限。Dapr building block の応答 SLO に基づく。
	DaprCall = 200 * time.Millisecond
	// RustCoreCall は tier1 内部 gRPC（Go ファサード ⇄ Rust core）の上限。
	// Rust core は ZEN Engine 評価 / hash chain / crypto を担当、ローカル UNIX socket か lo NIC で遅延小さい想定。
	RustCoreCall = 100 * time.Millisecond
)

// WithFacadeTimeout は呼び出し側 context に FacadeBudget を被せた子 context を返す。
//
// 親 context にすでに deadline がある場合、その deadline を尊重しつつ FacadeBudget で短縮する。
// caller は cancel() を必ず defer すること（go vet 対策）。
func WithFacadeTimeout(parent context.Context) (context.Context, context.CancelFunc) {
	// context.WithTimeout は親の deadline と引数 timeout の早い方を採用する。
	return context.WithTimeout(parent, FacadeBudget)
	// WithFacadeTimeout 関数を閉じる。
}

// WithDaprTimeout は Dapr sidecar 呼び出し用の子 context を返す。retry の各試行で本関数を呼ぶ想定。
func WithDaprTimeout(parent context.Context) (context.Context, context.CancelFunc) {
	// Dapr 呼び出し上限を被せる。retry 中の各試行ごとに新規 context が必要。
	return context.WithTimeout(parent, DaprCall)
	// WithDaprTimeout 関数を閉じる。
}

// WithRustCoreTimeout は Rust core 呼び出し用の子 context を返す。
func WithRustCoreTimeout(parent context.Context) (context.Context, context.CancelFunc) {
	// Rust core 呼び出し上限（短い、local IPC 想定）を被せる。
	return context.WithTimeout(parent, RustCoreCall)
	// WithRustCoreTimeout 関数を閉じる。
}
