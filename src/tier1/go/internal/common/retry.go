// 本ファイルは tier1 Go の retry / backoff ユーティリティ。
//
// 設計: plan/04_tier1_Goファサード実装/02_共通基盤.md（主作業 3）
//       配置先注記: docs 正典で reliability ディレクトリ未定義（DS-SW-COMP-110 の k1s0-policy は
//       JWT / Tenant / OPA / Rate Limit / 冪等性のみ記載、retry / circuit / timeout は明記なし）。
//       本ファイルは「横断 utility」として `internal/common/` に配置（DS-SW-COMP-108 と整合）。
//       次セッションで docs 正典が確定したら別ディレクトリへの移動を検討する。
// 関連 ID: IMP-RELIABILITY-* / NFR-B-PERF-*
//
// scope（リリース時点最小骨格）:
//   - RetryConfig 型と Default の定義のみ
//   - exponential backoff + jitter の実装は次セッション
//
// 未実装（plan 04-02 主作業 3 で追加、次セッション以降）:
//   - Do[T any](ctx, cfg, fn) (T, error) ジェネリック retry helper
//   - 指数バックオフ計算（100ms / 200ms / 400ms + ±50% jitter）
//   - Idempotent でない RPC は retry 対象外（status code フィルタ）
//   - gRPC status code（codes.Unavailable / codes.DeadlineExceeded 等）に基づく retry 判定

package common

// 標準ライブラリのみ import（minimal skeleton、third-party 依存なし）。
import (
	// 指数バックオフの遅延制御に time.Duration を使う。
	"time"
)

// RetryConfig は retry 戦略を表す。各 API ハンドラ（plan 04-04 〜 04-13）で個別に上書き可能。
type RetryConfig struct {
	// 最大試行回数（初回 1 + retry N-1 回）。0 以下なら retry 無効。
	MaxAttempts int
	// 初回 retry 前の待機時間（基準値）。指数バックオフでこれが倍々に伸びる。
	InitialDelay time.Duration
	// 上限の待機時間。指数増加が暴走しないようキャップする。
	MaxDelay time.Duration
	// jitter 比率（0.0〜1.0、0.5 で ±50%）。thundering herd 回避のため必須。
	JitterRatio float64
	// 構造体定義を閉じる。
}

// DefaultRetry は plan 04-02 主作業 3 のデフォルト retry 戦略（3 回 / 100ms-200ms-400ms / ±50%）。
//
// SLO 予算（500ms / API）との整合: 400ms 上限の retry が 1 段だけ効く想定で、
// 累積遅延が SLO を超えない範囲。circuit-breaker（次セッション）が連動する。
func DefaultRetry() RetryConfig {
	// plan 04-02 主作業 3 のデフォルト値を返す。
	return RetryConfig{
		// 初回 + 2 回 retry = 計 3 試行。
		MaxAttempts: 3,
		// 初回 retry の遅延 100ms。
		InitialDelay: 100 * time.Millisecond,
		// 上限 400ms（次セッションで指数増加 100→200→400 を実装）。
		MaxDelay: 400 * time.Millisecond,
		// ±50% jitter で thundering herd を防ぐ。
		JitterRatio: 0.5,
		// 構造体リテラルを閉じる。
	}
	// DefaultRetry 関数を閉じる。
}
