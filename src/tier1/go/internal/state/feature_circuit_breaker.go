// 本ファイルは Feature API の Circuit Breaker ルール実装（FR-T1-FEATURE-003）。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/11_Feature_API.md FR-T1-FEATURE-003
//     - Prometheus クエリを条件に指定可能
//     - 閾値超過から false 化まで 30 秒以内
//     - 自動切戻時の Audit 記録
//
// 実装方針:
//   FlagDefinition proto には circuit_breaker フィールドが無いため、proto を破壊せず
//   side-channel として override store を導入する。flag_key → forced (value, reason, until)
//   の thread-safe map で「強制 false 化」状態を保持する。
//
//   featureHandler.Evaluate* は adapter 呼出前に override を確認する。override が
//   ある間は adapter を呼ばずに forced value + Reason を返す（fail-soft 経路と同じ）。
//
//   Prometheus 評価は MetricThresholdSource interface でプラガブルにする。本実装は
//   interface 定義 + 評価ループ + override 設定のみ提供し、実 Prometheus クライアント
//   結線は cmd/state/main.go で env 経由で wire する想定（PROMETHEUS_URL 設定時のみ有効化）。

package state

import (
	"context"
	"log"
	"sync"
	"time"

	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	"github.com/k1s0/k1s0/src/tier1/go/internal/otel"
	otellog "go.opentelemetry.io/otel/log"
)

// FeatureCBRule は flag に紐付ける Circuit Breaker ルール 1 件。
//
// PromQL は外部クエリエンジンで評価され、結果が Threshold を超えると flag が
// FalseValue で強制上書きされる。RecoverAfter 経過後に override を解除し、
// 通常の評価経路に戻す（自動切戻）。
type FeatureCBRule struct {
	// 紐付け先 flag_key（<tenant>.<component>.<feature>）。
	FlagKey string
	// 評価条件（PromQL またはそれと等価な query 文字列）。
	PromQL string
	// 閾値（query 結果がこれを超えると override 発動）。
	Threshold float64
	// 比較演算子（"gt"=超過、"lt"=未満。既定は "gt"）。
	Comparator string
	// 自動切戻までの待機時間（既定 5 分）。
	RecoverAfter time.Duration
	// 強制値（boolean flag のみ：false にする / number は 0 にする等）。今回は bool false 固定運用。
	ForcedFalse bool
}

// MetricThresholdSource は Prometheus 評価の抽象化。実装は cmd 側で wire する。
//
// Evaluate(ctx, rule) は (現在値, error) を返す。error は network failure や
// PromQL 構文不正など。
type MetricThresholdSource interface {
	Evaluate(ctx context.Context, rule FeatureCBRule) (float64, error)
}

// FeatureFlagOverrideStore は flag_key → 強制値 のレジストリ。
//
// 並行安全な map で、handler 評価時の lock-free read を可能にするために RWMutex を使う。
// 実体は read-mostly（評価は秒単位、override 変更は分単位）。
type FeatureFlagOverrideStore struct {
	mu        sync.RWMutex
	overrides map[string]forcedFlag
}

// forcedFlag は強制値の保持構造。
type forcedFlag struct {
	// 強制値（現状 false 固定だが、将来拡張のため bool で保持）。
	value bool
	// 自動切戻時刻（過ぎたら override が無効）。
	until time.Time
	// 理由文字列（audit / metadata.reason に使う）。
	reason string
}

// NewFeatureFlagOverrideStore は空の override store を生成する。
func NewFeatureFlagOverrideStore() *FeatureFlagOverrideStore {
	return &FeatureFlagOverrideStore{overrides: map[string]forcedFlag{}}
}

// Force は flag を value で強制上書きする。until 経過後は自動的に解除される。
func (s *FeatureFlagOverrideStore) Force(flagKey string, value bool, reason string, until time.Time) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.overrides[flagKey] = forcedFlag{value: value, until: until, reason: reason}
}

// Lookup は flag_key の現在 override を返す。
//
// 戻り値:
//   - found: override が存在し、かつ until 未到達なら true
//   - value: 強制値
//   - reason: 強制理由（"CIRCUIT_BREAKER" など）
func (s *FeatureFlagOverrideStore) Lookup(flagKey string) (value bool, reason string, found bool) {
	s.mu.RLock()
	f, ok := s.overrides[flagKey]
	s.mu.RUnlock()
	if !ok {
		return false, "", false
	}
	if time.Now().After(f.until) {
		// 期限切れ → write lock で削除（GC 兼ねる）。
		s.mu.Lock()
		delete(s.overrides, flagKey)
		s.mu.Unlock()
		return false, "", false
	}
	return f.value, f.reason, true
}

// Clear は flag_key の override を即時解除する（手動切戻用）。
func (s *FeatureFlagOverrideStore) Clear(flagKey string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	delete(s.overrides, flagKey)
}

// Snapshot は現在の全 override の flag_key 一覧を返す（observability / debug 用）。
func (s *FeatureFlagOverrideStore) Snapshot() []string {
	s.mu.RLock()
	defer s.mu.RUnlock()
	out := make([]string, 0, len(s.overrides))
	for k := range s.overrides {
		out = append(out, k)
	}
	return out
}

// FeatureCircuitBreakerEvaluator は Circuit Breaker ルールを定期評価する。
//
// 動作:
//   - 30 秒（Interval）ごとに全ルールを順次評価
//   - PromQL 結果が閾値超過 → override store に強制 false を書き込む
//   - 既に override がある場合は上書き（until を更新）
//   - 閾値内に戻った場合は何もしない（自動切戻は until 経過で発火）
//   - 各 override 設定時に Audit emitter があれば audit 1 件発火
type FeatureCircuitBreakerEvaluator struct {
	// 評価対象ルール（起動時固定、動的追加は SetRules で）。
	rules []FeatureCBRule
	// rules を保護する mutex（SetRules で書き換え）。
	rulesMu sync.RWMutex
	// PromQL 評価源。
	source MetricThresholdSource
	// override 書き込み先。
	store *FeatureFlagOverrideStore
	// 評価間隔（既定 30 秒）。FR-T1-FEATURE-003 の「閾値超過から false 化まで 30 秒以内」に対応。
	interval time.Duration
	// audit 発火（nil なら audit 無し）。
	audit otel.LogEmitter
	// shutdown 用 cancel。
	cancel context.CancelFunc
	// goroutine 終了同期。
	wg sync.WaitGroup
}

// NewFeatureCircuitBreakerEvaluator は評価器を生成する。
// interval が 0 以下なら 30 秒（FR-T1-FEATURE-003 受け入れ基準値）が使われる。
func NewFeatureCircuitBreakerEvaluator(source MetricThresholdSource, store *FeatureFlagOverrideStore, interval time.Duration) *FeatureCircuitBreakerEvaluator {
	if interval <= 0 {
		interval = 30 * time.Second
	}
	return &FeatureCircuitBreakerEvaluator{
		source:   source,
		store:    store,
		interval: interval,
	}
}

// SetRules は評価対象ルールを差替える（thread-safe）。
func (e *FeatureCircuitBreakerEvaluator) SetRules(rules []FeatureCBRule) {
	e.rulesMu.Lock()
	defer e.rulesMu.Unlock()
	e.rules = make([]FeatureCBRule, len(rules))
	copy(e.rules, rules)
}

// SetAuditEmitter は override 設定時の audit emitter を差替える。nil で無効化。
func (e *FeatureCircuitBreakerEvaluator) SetAuditEmitter(emitter otel.LogEmitter) {
	e.audit = emitter
}

// Start は評価ループを別 goroutine で起動する。Stop が呼ばれるまで動き続ける。
func (e *FeatureCircuitBreakerEvaluator) Start(ctx context.Context) {
	if e.cancel != nil {
		return // 既に start 済
	}
	child, cancel := context.WithCancel(ctx)
	e.cancel = cancel
	e.wg.Add(1)
	go e.run(child)
}

// Stop は評価ループを停止し、goroutine の終了を待つ。
func (e *FeatureCircuitBreakerEvaluator) Stop() {
	if e.cancel == nil {
		return
	}
	e.cancel()
	e.wg.Wait()
	e.cancel = nil
}

// run は評価ループ本体。ticker で interval ごとに evaluateOnce を呼ぶ。
func (e *FeatureCircuitBreakerEvaluator) run(ctx context.Context) {
	defer e.wg.Done()
	t := time.NewTicker(e.interval)
	defer t.Stop()
	// 起動直後にも 1 回評価する（30 秒間放置を避ける）。
	e.evaluateOnce(ctx)
	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			e.evaluateOnce(ctx)
		}
	}
}

// evaluateOnce は全ルールを 1 回評価する。
func (e *FeatureCircuitBreakerEvaluator) evaluateOnce(ctx context.Context) {
	e.rulesMu.RLock()
	rules := make([]FeatureCBRule, len(e.rules))
	copy(rules, e.rules)
	e.rulesMu.RUnlock()
	for _, rule := range rules {
		val, err := e.source.Evaluate(ctx, rule)
		if err != nil {
			// 評価不能はスキップ（次回 retry）。Prometheus 一時障害で誤発火しないように。
			log.Printf("tier1/feature: cb rule %q evaluation failed: %v", rule.FlagKey, err)
			continue
		}
		if e.exceedsThreshold(rule, val) {
			until := time.Now().Add(rule.recoverDuration())
			e.store.Force(rule.FlagKey, !rule.ForcedFalse, "CIRCUIT_BREAKER", until)
			e.emitAudit(ctx, rule, val)
		}
	}
}

// exceedsThreshold は rule.Comparator に従って閾値超過判定する。
func (e *FeatureCircuitBreakerEvaluator) exceedsThreshold(rule FeatureCBRule, val float64) bool {
	switch rule.Comparator {
	case "lt":
		return val < rule.Threshold
	default:
		// "gt" / 未指定 / 不明はすべて超過判定として扱う（既定動作）。
		return val > rule.Threshold
	}
}

// emitAudit は override 設定を audit emitter に通知する（FR-T1-FEATURE-003 受け入れ基準
// 「自動切戻時の Audit 記録」）。emitter 不在なら no-op。
func (e *FeatureCircuitBreakerEvaluator) emitAudit(ctx context.Context, rule FeatureCBRule, val float64) {
	if e.audit == nil {
		return
	}
	_ = e.audit.Emit(ctx, otel.LogEntry{
		Timestamp:    time.Now().UnixNano(),
		Severity:     otellog.SeverityWarn,
		SeverityText: "WARN",
		Body:         "feature flag forced to false by circuit breaker",
		Attributes: map[string]string{
			"flag_key":   rule.FlagKey,
			"promql":     rule.PromQL,
			"threshold":  formatFloat(rule.Threshold),
			"observed":   formatFloat(val),
			"action":     "FEATURE_CIRCUIT_BREAKER_TRIPPED",
			"comparator": rule.Comparator,
		},
	})
}

// recoverDuration は rule.RecoverAfter の正規化（既定 5 分）。
func (rule FeatureCBRule) recoverDuration() time.Duration {
	if rule.RecoverAfter > 0 {
		return rule.RecoverAfter
	}
	return 5 * time.Minute
}

// formatFloat は float64 を簡潔な文字列に整形する（strconv 直依存を避けるため最小実装）。
func formatFloat(v float64) string {
	// 簡易整形: 整数部 + 小数部 4 桁。
	intPart := int64(v)
	frac := v - float64(intPart)
	if frac < 0 {
		frac = -frac
	}
	// 小数部 4 桁を 10000 倍して int に。
	fracInt := int64(frac * 10000)
	// 符号
	sign := ""
	if v < 0 {
		sign = "-"
		if intPart < 0 {
			intPart = -intPart
		}
	}
	return sign + intToASCII(intPart) + "." + intToASCIIPad(fracInt, 4)
}

// intToASCII は int64 を 10 進文字列に変換する（fmt.Sprintf 依存軽減用）。
func intToASCII(n int64) string {
	if n == 0 {
		return "0"
	}
	buf := [20]byte{}
	i := len(buf)
	for n > 0 {
		i--
		buf[i] = byte('0' + n%10)
		n /= 10
	}
	return string(buf[i:])
}

// intToASCIIPad は int64 を pad 桁ゼロ埋めの 10 進文字列に変換する。
func intToASCIIPad(n int64, pad int) string {
	s := intToASCII(n)
	for len(s) < pad {
		s = "0" + s
	}
	return s
}

// 静的に common パッケージへの依存を維持する（Audit 経路で利用予定の lint 対策）。
var _ = common.IdempotencyKey
