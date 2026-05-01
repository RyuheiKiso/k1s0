// 本ファイルは tier1 Go の retry / backoff ユーティリティ。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md §「冪等性と再試行」
//     - クライアント SDK: Unavailable / ResourceExhausted に対して指数バックオフ
//       （初回 100ms、最大 3 回、jitter ±20%）
//     - DeadlineExceeded はサーバ到達前のキャンセルとみなして再試行可能
//     - retry_after_ms（K1s0Error）が返された場合は SDK がその値を優先
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/01_Service_Invoke_API.md FR-T1-INVOKE-003
// 関連 ID: IMP-RELIABILITY-* / NFR-B-PERF-* / FR-T1-INVOKE-003
//
// 役割:
//   - RetryConfig: 戦略パラメータ（最大試行回数、初回遅延、上限遅延、jitter 比）
//   - Do[T]: 任意 RPC を retry でラップする汎用ヘルパ。gRPC status code をもとに
//     retryable 判定し、retry_after_ms（K1s0Error）が trailer / status detail に
//     含まれる場合は exponential backoff より優先する。

package common

import (
	// retry の context cancel と deadline 監視。
	"context"
	// retry-after を返す K1s0Error 用の status details Unmarshal。
	"errors"
	// 指数バックオフの jitter 用乱数（math/rand v2 でテスト時の再現性を確保）。
	"math/rand/v2"
	// 指数バックオフの遅延制御。
	"time"

	// gRPC status / code（retryable 判定）。
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// RetryConfig は retry 戦略を表す。各 API ハンドラで個別に上書き可能。
type RetryConfig struct {
	// 最大試行回数（初回 1 + retry N-1 回）。0 以下なら retry 無効（initial 1 回のみ）。
	MaxAttempts int
	// 初回 retry 前の待機時間（基準値）。指数バックオフでこれが倍々に伸びる。
	InitialDelay time.Duration
	// 上限の待機時間。指数増加が暴走しないようキャップする。
	MaxDelay time.Duration
	// jitter 比率（0.0〜1.0、0.5 で ±50%）。thundering herd 回避のため必須。
	JitterRatio float64
	// IsRetryable はエラーごとに retry 可否を判定する。nil の場合は DefaultIsRetryable を使う。
	// 副作用のある操作（State.Set 等）から DeadlineExceeded を retry 対象から外す等で使う。
	IsRetryable func(error) bool
}

// DefaultRetry は共通規約の既定戦略（3 試行 / 100ms 起点 / ±20% jitter）を返す。
//
// SLO 予算（500ms / API）との整合: 100ms + 200ms = 300ms 累積遅延（最悪ケース）が
// FacadeBudget(500ms) 未満となる。
func DefaultRetry() RetryConfig {
	// 共通規約の既定値を構築して返す。
	return RetryConfig{
		// 初回 + 2 回 retry = 計 3 試行。
		MaxAttempts: 3,
		// 初回 retry の遅延 100ms。
		InitialDelay: 100 * time.Millisecond,
		// 上限 400ms（指数増加 100→200→400 のキャップ）。
		MaxDelay: 400 * time.Millisecond,
		// 共通規約は ±20% を既定とする（thundering herd 回避）。
		JitterRatio: 0.2,
		// IsRetryable は nil の場合 DefaultIsRetryable が使われる。
		IsRetryable: nil,
	}
}

// DefaultIsRetryable は共通規約の retryable エラー判定。
//
// 共通規約 §「冪等性と再試行」に従い、Unavailable / ResourceExhausted /
// DeadlineExceeded を retryable とする。それ以外は即座に呼出側へ返す。
//
// 注意: 副作用のある操作で DeadlineExceeded を retry すると重複実行のリスクが
// あるため、書込系 RPC は IsRetryable をオーバーライドして DeadlineExceeded を
// 除外することが望ましい。
func DefaultIsRetryable(err error) bool {
	// nil（成功）は retry しない。
	if err == nil {
		return false
	}
	// gRPC status を取得する。
	st, ok := status.FromError(err)
	// status 取得不可（plain error）は retryable 不能と判定する。
	if !ok {
		return false
	}
	// 共通規約の既定 retryable 集合を判定する。
	switch st.Code() {
	case codes.Unavailable, codes.ResourceExhausted, codes.DeadlineExceeded:
		return true
	default:
		return false
	}
}

// IdempotentRetryable は副作用なし RPC 用の retryable 判定（DefaultIsRetryable と同等）。
// State.Get / Secrets.Get / Decision.Evaluate / Feature.Evaluate 等で使う。
func IdempotentRetryable(err error) bool {
	return DefaultIsRetryable(err)
}

// MutationRetryable は副作用あり RPC 用の retryable 判定。
//
// DeadlineExceeded を除外する: サーバ到達後のキャンセル可能性を考慮して、
// 副作用が「発生したかも知れない」状態での retry は重複実行リスクがある。
// State.Set / PubSub.Publish / Workflow.Start / Secrets.Rotate / Binding.Send /
// Audit.Write など書込系 RPC で使う。
func MutationRetryable(err error) bool {
	// nil（成功）は retry しない。
	if err == nil {
		return false
	}
	// gRPC status を取得する。
	st, ok := status.FromError(err)
	// status 取得不可（plain error）は retryable 不能と判定する。
	if !ok {
		return false
	}
	// 副作用ありは Unavailable / ResourceExhausted のみ retryable。
	switch st.Code() {
	case codes.Unavailable, codes.ResourceExhausted:
		return true
	default:
		return false
	}
}

// retryAfterFromError は gRPC status の details に含まれる retry_after_ms を取り出す。
//
// 共通規約 §「冪等性と再試行」: retry_after_ms（K1s0Error）が返された場合は
// 指数バックオフより優先する。details に google.rpc.RetryInfo が含まれる場合に対応する。
//
// 戻り値:
//   - first: retry_after の Duration。0 なら指定なし。
func retryAfterFromError(err error) time.Duration {
	// status 取得を試みる。
	st, ok := status.FromError(err)
	// status 取得不可なら 0 を返す。
	if !ok {
		return 0
	}
	// details を走査して RetryInfo を探す。
	for _, d := range st.Details() {
		// google.rpc.RetryInfo（duck-type）かどうかを判定する。
		// 完全な genproto/googleapis/rpc/errdetails 依存を避けるため、interface assertion で
		// GetRetryDelay() time.Duration の有無のみで判定する。
		if ri, isRetryInfo := d.(errdetailsRetryInfo); isRetryInfo && ri != nil {
			// RetryDelay が設定されていればそれを返す。
			return ri.GetRetryDelay()
		}
	}
	// details に retry_after が含まれない場合は 0。
	return 0
}

// computeBackoff は試行回数に基づいて待機時間を計算する。
//
// 引数:
//   - attempt: 1-based 試行番号（1 が「初回失敗後の retry 前待機」）
//   - cfg: retry 戦略
//
// 計算式:
//   - base = InitialDelay * 2^(attempt-1)
//   - cap  = min(base, MaxDelay)
//   - jitter ratio が r のとき、最終待機 = cap * (1 + Uniform(-r, +r))
func computeBackoff(attempt int, cfg RetryConfig) time.Duration {
	// 1-based を 0-based に変換する。
	exp := attempt - 1
	if exp < 0 {
		exp = 0
	}
	// 指数増加（2^exp）を計算する（int 範囲安全のため上限を予めキャップ）。
	multiplier := 1 << uint(exp)
	// nominal 値を計算する。
	nominal := cfg.InitialDelay * time.Duration(multiplier)
	// MaxDelay でキャップする。
	if cfg.MaxDelay > 0 && nominal > cfg.MaxDelay {
		nominal = cfg.MaxDelay
	}
	// jitter 比率が 0 なら nominal をそのまま返す。
	if cfg.JitterRatio <= 0 {
		return nominal
	}
	// jitter 量を [-r, +r] の Uniform 乱数で計算する。
	delta := (rand.Float64()*2 - 1) * cfg.JitterRatio
	// 1 + delta を nominal に乗じて返す（負値ガード）。
	final := time.Duration(float64(nominal) * (1 + delta))
	if final < 0 {
		return 0
	}
	return final
}

// Do は任意の RPC を retry でラップする汎用ヘルパ。
//
// 動作:
//   - fn() を最大 cfg.MaxAttempts 回試行する
//   - 成功 (err==nil) または non-retryable error で即返却
//   - retryable error の場合、computeBackoff で待機してから次試行
//   - server が retry_after_ms を返した場合は computeBackoff より優先
//   - ctx が cancel/deadline に達したら即返却（最後の err を含む）
//
// FR-T1-INVOKE-003 / 共通規約 §「冪等性と再試行」を満たす。
func Do[T any](ctx context.Context, cfg RetryConfig, fn func() (T, error)) (T, error) {
	// retryable 判定関数を解決する（未指定時は Default）。
	isRetryable := cfg.IsRetryable
	if isRetryable == nil {
		isRetryable = DefaultIsRetryable
	}
	// MaxAttempts が 0 以下の場合は 1 回試行のみ（retry 無効）。
	maxAttempts := cfg.MaxAttempts
	if maxAttempts < 1 {
		maxAttempts = 1
	}
	// 直近のエラーを保持する（全失敗時に呼出側へ返す）。
	var lastErr error
	// 試行を回す。
	for attempt := 1; attempt <= maxAttempts; attempt++ {
		// 試行前に context が cancel/deadline していないか確認する。
		if err := ctx.Err(); err != nil {
			// 以前の lastErr があればそれを優先（retry が context により打ち切られた典型ケース）。
			if lastErr != nil {
				var zero T
				return zero, lastErr
			}
			var zero T
			return zero, err
		}
		// 実 RPC を呼ぶ。
		out, err := fn()
		// 成功なら即返却。
		if err == nil {
			return out, nil
		}
		// 直近エラーを更新する。
		lastErr = err
		// retry 不能なら即返却。
		if !isRetryable(err) {
			var zero T
			return zero, err
		}
		// 最終試行で失敗した場合は retry 不要なので即返却。
		if attempt >= maxAttempts {
			var zero T
			return zero, err
		}
		// 待機時間を決定する。サーバが retry_after を返した場合は優先。
		backoff := retryAfterFromError(err)
		if backoff <= 0 {
			backoff = computeBackoff(attempt, cfg)
		}
		// context cancel/deadline と backoff を競合させて待機する。
		timer := time.NewTimer(backoff)
		select {
		case <-ctx.Done():
			// context が打ち切られた場合は last err を返す（context error より具体的な情報）。
			timer.Stop()
			var zero T
			return zero, lastErr
		case <-timer.C:
			// 通常の backoff 経過。次試行へ。
		}
	}
	// 想定到達不能（ループ内で必ず return する）。defensive に lastErr を返す。
	if lastErr == nil {
		lastErr = errors.New("tier1/common: retry loop exhausted without error")
	}
	var zero T
	return zero, lastErr
}

// errdetailsRetryInfo は google.rpc.RetryInfo の最小 interface。
//
// 完全な依存（google.golang.org/genproto/googleapis/rpc/errdetails）を避けるため、
// status detail に含まれた値の duck-type 取得のみを行う。
type errdetailsRetryInfo interface {
	// RetryDelay は次試行までの推奨遅延を返す。
	GetRetryDelay() time.Duration
}
