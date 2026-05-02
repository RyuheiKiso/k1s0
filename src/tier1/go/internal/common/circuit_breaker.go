// 本ファイルは tier1 facade の Circuit Breaker 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/01_Service_Invoke_API.md FR-T1-INVOKE-004
//     - 連続失敗回数の閾値、half-open 遷移までの時間を Component YAML で設定可能
//     - Circuit Breaker の状態（closed / open / half-open）を Prometheus で可視化
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md §「エラー型 K1s0Error」
//     - Unavailable / ResourceExhausted は再試行可能、retry_after_ms 必須
//
// 状態機械:
//   closed   → 通常動作。連続失敗が threshold に達すると open に遷移。
//   open     → 全呼出を即時 Unavailable で弾く。half-open-after 経過後 half-open へ。
//   half-open → 1 件だけ probe 呼出を許可。成功で closed、失敗で open に戻す。
//
// 同時並行性:
//   sync.Mutex で state / counters を保護。Allow() / RecordSuccess() / RecordFailure() は
//   全部 lock を取る。ホットパスでは microsecond オーダの critical section に収まる。
//
// 観測性:
//   状態遷移は MetricEmitter 越しに Gauge `tier1_invoke_circuit_breaker_state`
//   （0=closed / 1=open / 2=half-open）として publish される。Prometheus 経由で
//   Grafana で可視化する運用を想定（DS-SW-COMP-038 / NFR-E-MON-001）。

package common

import (
	// 試行カウントの atomic 操作（ホットパス最適化）。
	"sync"
	// 半 open までの時間計算。
	"time"
)

// CBState は Circuit Breaker の論理状態。
type CBState int32

const (
	// CBClosed は通常動作。失敗が threshold まで蓄積されるまで全呼出を許可する。
	CBClosed CBState = 0
	// CBOpen は全呼出を即拒否する。half-open-after 経過後に CBHalfOpen へ遷移する。
	CBOpen CBState = 1
	// CBHalfOpen は probe 呼出を 1 件だけ許可する。成功で CBClosed、失敗で CBOpen。
	CBHalfOpen CBState = 2
)

// String は CBState を可視化用文字列に変換する。
func (s CBState) String() string {
	switch s {
	case CBClosed:
		return "closed"
	case CBOpen:
		return "open"
	case CBHalfOpen:
		return "half-open"
	default:
		return "unknown"
	}
}

// CBConfig は Circuit Breaker のチューニングパラメータ。
type CBConfig struct {
	// FailureThreshold は連続失敗回数の閾値。これを超えると closed → open に遷移する。
	// 0 以下なら 5（既定値）が使われる。
	FailureThreshold int
	// HalfOpenAfter は open 状態で待機する時間。経過後 half-open に自動遷移する。
	// 0 以下なら 30 秒（既定値）が使われる。
	HalfOpenAfter time.Duration
	// HalfOpenMaxProbes は half-open 中に同時許可する probe 数。
	// 0 以下なら 1（既定値）が使われる。
	HalfOpenMaxProbes int
}

// DefaultCBConfig は共通規約の既定値（5 連続失敗 / 30 秒 half-open / 1 probe）。
func DefaultCBConfig() CBConfig {
	return CBConfig{
		FailureThreshold:  5,
		HalfOpenAfter:     30 * time.Second,
		HalfOpenMaxProbes: 1,
	}
}

// CBStateObserver は CB 状態変更を観測する callback。テストおよび metric publish で利用する。
type CBStateObserver func(name string, state CBState)

// CircuitBreaker は state machine と並行制御を担う。
//
// インスタンスは「呼出先（target）」単位で構築する。複数 target を扱う場合は
// CircuitBreakerRegistry（後述）が name → *CircuitBreaker を管理する。
type CircuitBreaker struct {
	// 識別名（target / appId など、metric label に使う）。
	name string
	// チューニングパラメータ。
	cfg CBConfig
	// state / counters を保護する mutex。
	mu sync.Mutex
	// 現在の状態。
	state CBState
	// closed 状態での連続失敗カウント（成功で 0 にリセット）。
	failures int
	// open 状態に入った時刻。half-open 遷移判定に使う。
	openedAt time.Time
	// half-open 中に許可済の probe 数。
	probesInFlight int
	// 状態変更時に呼ばれる observer（nil 可）。
	observer CBStateObserver
}

// NewCircuitBreaker は名前と設定から CircuitBreaker を生成する。
// 0 以下の設定値は DefaultCBConfig の既定値で補完する。
func NewCircuitBreaker(name string, cfg CBConfig) *CircuitBreaker {
	def := DefaultCBConfig()
	if cfg.FailureThreshold <= 0 {
		cfg.FailureThreshold = def.FailureThreshold
	}
	if cfg.HalfOpenAfter <= 0 {
		cfg.HalfOpenAfter = def.HalfOpenAfter
	}
	if cfg.HalfOpenMaxProbes <= 0 {
		cfg.HalfOpenMaxProbes = def.HalfOpenMaxProbes
	}
	return &CircuitBreaker{
		name:  name,
		cfg:   cfg,
		state: CBClosed,
	}
}

// SetObserver は状態変更通知 callback を設定する。metric publish や audit hook 用。
// nil をセットすると無効化される。
func (cb *CircuitBreaker) SetObserver(o CBStateObserver) {
	cb.mu.Lock()
	defer cb.mu.Unlock()
	cb.observer = o
}

// Name は識別名を返す（metric label / log で使う）。
func (cb *CircuitBreaker) Name() string {
	return cb.name
}

// State は現在の論理状態を返す（参照のみ）。
func (cb *CircuitBreaker) State() CBState {
	cb.mu.Lock()
	defer cb.mu.Unlock()
	return cb.evalStateLocked(time.Now())
}

// evalStateLocked は open → half-open の自動遷移を評価する。lock 内呼出専用。
func (cb *CircuitBreaker) evalStateLocked(now time.Time) CBState {
	// open のときだけ時間判定する。
	if cb.state == CBOpen && now.Sub(cb.openedAt) >= cb.cfg.HalfOpenAfter {
		// open → half-open。
		cb.transitionLocked(CBHalfOpen)
	}
	return cb.state
}

// transitionLocked は state を遷移し、observer に通知する。lock 内呼出専用。
func (cb *CircuitBreaker) transitionLocked(next CBState) {
	if cb.state == next {
		return
	}
	cb.state = next
	// state-specific reset。
	switch next {
	case CBClosed:
		cb.failures = 0
		cb.probesInFlight = 0
	case CBOpen:
		cb.openedAt = time.Now()
		cb.probesInFlight = 0
	case CBHalfOpen:
		cb.probesInFlight = 0
	}
	// observer を lock 内で呼ぶ（簡略化のため、observer は重い処理を避ける契約）。
	if cb.observer != nil {
		cb.observer(cb.name, next)
	}
}

// Allow は呼出を許可するか判定する。
//
// 戻り値:
//   - true: 呼出可能（呼出後に必ず RecordSuccess / RecordFailure を 1 回呼ぶ責務）
//   - false: 拒否（caller は Unavailable + retry_after_ms = HalfOpenAfter で返す）
func (cb *CircuitBreaker) Allow() bool {
	cb.mu.Lock()
	defer cb.mu.Unlock()
	state := cb.evalStateLocked(time.Now())
	switch state {
	case CBClosed:
		return true
	case CBHalfOpen:
		// half-open は probe を MaxProbes 件まで許可する。
		if cb.probesInFlight < cb.cfg.HalfOpenMaxProbes {
			cb.probesInFlight++
			return true
		}
		// 既に probe 上限に達している場合は拒否する。
		return false
	default:
		// open は全拒否。
		return false
	}
}

// RecordSuccess は成功した呼出の結果を記録する。
//
// closed: 連続失敗カウンタをリセット
// half-open: closed に遷移してカウンタリセット
// open: 通常 Allow=false なので呼ばれないが、defensive に no-op
func (cb *CircuitBreaker) RecordSuccess() {
	cb.mu.Lock()
	defer cb.mu.Unlock()
	switch cb.state {
	case CBClosed:
		cb.failures = 0
	case CBHalfOpen:
		// probe 成功で closed に遷移する。
		cb.transitionLocked(CBClosed)
	}
}

// RecordFailure は失敗した呼出の結果を記録する。
//
// closed: 連続失敗カウンタを ++、threshold 超で open に遷移
// half-open: 即 open に戻す（probe 失敗 → 再開はもう一度 HalfOpenAfter 待つ）
// open: 既に open なので no-op
func (cb *CircuitBreaker) RecordFailure() {
	cb.mu.Lock()
	defer cb.mu.Unlock()
	switch cb.state {
	case CBClosed:
		cb.failures++
		if cb.failures >= cb.cfg.FailureThreshold {
			cb.transitionLocked(CBOpen)
		}
	case CBHalfOpen:
		cb.transitionLocked(CBOpen)
	}
}

// HalfOpenAfter は open 中の caller が retry_after_ms を返す際の参考値を提供する。
func (cb *CircuitBreaker) HalfOpenAfter() time.Duration {
	return cb.cfg.HalfOpenAfter
}

// CircuitBreakerRegistry は target → *CircuitBreaker の thread-safe レジストリ。
//
// invoke handler は appId（呼出先）ごとに breaker を確保する。同 appId への複数並行
// 呼出は同一 breaker を共有し、失敗連動する。テナントを横断して同 appId を共有する
// 設計（appId 自体に L2 prefix が乗る前提のため、テナント越境はそもそも起きない）。
type CircuitBreakerRegistry struct {
	mu       sync.RWMutex
	cfg      CBConfig
	bs       map[string]*CircuitBreaker
	observer CBStateObserver
}

// NewCircuitBreakerRegistry は CB レジストリを生成する。observer は metric publish 用。
func NewCircuitBreakerRegistry(cfg CBConfig, observer CBStateObserver) *CircuitBreakerRegistry {
	return &CircuitBreakerRegistry{
		cfg:      cfg,
		bs:       make(map[string]*CircuitBreaker),
		observer: observer,
	}
}

// Get は name に対応する breaker を取り出す（無ければ生成）。
func (r *CircuitBreakerRegistry) Get(name string) *CircuitBreaker {
	// 既存 breaker を lock-free で先に試す。
	r.mu.RLock()
	if b, ok := r.bs[name]; ok {
		r.mu.RUnlock()
		return b
	}
	r.mu.RUnlock()
	// 新規生成は write lock。
	r.mu.Lock()
	defer r.mu.Unlock()
	// 二重 check（race 解消）。
	if b, ok := r.bs[name]; ok {
		return b
	}
	b := NewCircuitBreaker(name, r.cfg)
	if r.observer != nil {
		b.SetObserver(r.observer)
	}
	r.bs[name] = b
	return b
}

// Snapshot は全 breaker の現在状態を返す（observability / debug 用）。
func (r *CircuitBreakerRegistry) Snapshot() map[string]CBState {
	r.mu.RLock()
	defer r.mu.RUnlock()
	out := make(map[string]CBState, len(r.bs))
	for k, b := range r.bs {
		out[k] = b.State()
	}
	return out
}

// LoadCBConfigFromEnv は env から CBConfig を読む。
//
// 環境変数:
//   - TIER1_CB_FAILURE_THRESHOLD: 連続失敗閾値（既定 5）
//   - TIER1_CB_HALF_OPEN_AFTER:   open → half-open の待機時間（time.ParseDuration 形式、既定 30s）
//   - TIER1_CB_HALF_OPEN_PROBES:  half-open 中の最大 probe 数（既定 1）
//
// 不正値は warn ログを残して既定値にフォールバックする（FR-T1-INVOKE-004「Component
// YAML で設定可能」を env 経由で実現）。
func LoadCBConfigFromEnv(getenv func(string) string, logf func(format string, args ...any)) CBConfig {
	cfg := DefaultCBConfig()
	if v := getenv("TIER1_CB_FAILURE_THRESHOLD"); v != "" {
		if n, err := parsePositiveInt(v); err == nil && n > 0 {
			cfg.FailureThreshold = n
		} else if logf != nil {
			logf("tier1/cb: invalid TIER1_CB_FAILURE_THRESHOLD=%q, using default %d", v, cfg.FailureThreshold)
		}
	}
	if v := getenv("TIER1_CB_HALF_OPEN_AFTER"); v != "" {
		if d, err := time.ParseDuration(v); err == nil && d > 0 {
			cfg.HalfOpenAfter = d
		} else if logf != nil {
			logf("tier1/cb: invalid TIER1_CB_HALF_OPEN_AFTER=%q, using default %v", v, cfg.HalfOpenAfter)
		}
	}
	if v := getenv("TIER1_CB_HALF_OPEN_PROBES"); v != "" {
		if n, err := parsePositiveInt(v); err == nil && n > 0 {
			cfg.HalfOpenMaxProbes = n
		} else if logf != nil {
			logf("tier1/cb: invalid TIER1_CB_HALF_OPEN_PROBES=%q, using default %d", v, cfg.HalfOpenMaxProbes)
		}
	}
	return cfg
}

// parsePositiveInt は ASCII 数字のみの正の整数を parse する（strconv 依存軽減用）。
func parsePositiveInt(s string) (int, error) {
	if s == "" {
		return 0, errEmptyInt
	}
	var n int64
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c < '0' || c > '9' {
			return 0, errInvalidInt
		}
		n = n*10 + int64(c-'0')
		if n > 1<<31-1 {
			return 0, errOverflowInt
		}
	}
	return int(n), nil
}

// 解析エラー sentinel。
var (
	errEmptyInt    = cbErr("empty integer string")
	errInvalidInt  = cbErr("invalid integer character")
	errOverflowInt = cbErr("integer overflow")
)

// cbErr は string をそのままラップする error 型。
type cbErr string

// Error は error interface を満たす。
func (e cbErr) Error() string { return string(e) }

