// 本ファイルは tier1 facade のテナント単位レート制限 interceptor。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「レート制限とクォータ」:
//       全 API はテナント単位の RPS 上限と同時接続数上限を受ける（NFR-E-NW-004）。
//       既定値:
//         Free / Trial:  50 RPS、同時接続  10
//         Standard:     500 RPS、同時接続 100
//         Enterprise: 5,000 RPS、同時接続 1,000
//       超過時は ResourceExhausted を retry_after_ms 付きで返す。
//       バーストは 2 倍まで 10 秒間許容する（token bucket）。
//   docs/03_要件定義/30_非機能要件/E_セキュリティ.md NFR-E-NW-004
//
// 実装方針:
//   - テナント別 token bucket（rate, burst）を sync.Map で保持する
//   - lazy 生成 / 直近未使用テナントは expirers で GC（メモリ上限 / DoS 緩和）
//   - 単一 Pod 内だけで完結（multi-replica 共有は Valkey backend が望ましいが、
//     production の正確な per-tenant RPS は ingress / Envoy Rate Limit Service に任せる
//     設計が docs 共通規約と整合する。本 interceptor は tier1 内部の追加保護層）
//   - tenant_id 不在 / unknown は適用対象外（AuthInterceptor が拒否済みのため到達しない）
//
// 既定値:
//   docs 既定の Standard プラン（500 RPS、burst 1000）を採用。env で上書き可:
//     TIER1_RATELIMIT_RPS=<int>      （0 で無効化）
//     TIER1_RATELIMIT_BURST=<int>    （未指定なら RPS の 2 倍）

package common

import (
	"context"
	"os"
	"strconv"
	"sync"
	"time"

	"google.golang.org/grpc"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// RateLimitConfig は AuthInterceptor の挙動を制御する。
type RateLimitConfig struct {
	// RPS は許容定常レート（per tenant、tokens / second）。0 で interceptor 無効化。
	RPS float64
	// Burst は token bucket 容量（瞬間最大）。docs 既定は RPS の 2 倍（10 秒間バースト）。
	Burst float64
	// SkipMethods は rate limit 対象外（health / reflection など）。
	SkipMethods map[string]bool
	// IdleEvictTTL を超えてアクセスのないテナントの bucket は内部から GC する。
	IdleEvictTTL time.Duration
}

// LoadRateLimitConfigFromEnv は env から RateLimitConfig を構築する。
// TIER1_RATELIMIT_RPS=0 / 未指定 で interceptor は no-op として返却される。
func LoadRateLimitConfigFromEnv() RateLimitConfig {
	rps, _ := strconv.ParseFloat(os.Getenv("TIER1_RATELIMIT_RPS"), 64)
	burst, _ := strconv.ParseFloat(os.Getenv("TIER1_RATELIMIT_BURST"), 64)
	if burst <= 0 && rps > 0 {
		// docs 規約: バースト 2 倍まで 10 秒間。
		burst = rps * 2
	}
	return RateLimitConfig{
		RPS:   rps,
		Burst: burst,
		SkipMethods: map[string]bool{
			"/grpc.health.v1.Health/Check":                                   true,
			"/grpc.health.v1.Health/Watch":                                   true,
			"/grpc.reflection.v1.ServerReflection/ServerReflectionInfo":      true,
			"/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo": true,
			"/k1s0.tier1.health.v1.HealthService/Liveness":                   true,
			"/k1s0.tier1.health.v1.HealthService/Readiness":                  true,
		},
		IdleEvictTTL: 5 * time.Minute,
	}
}

// tokenBucket は per-tenant の token bucket 状態。
type tokenBucket struct {
	// 現在の token 残量（float でドリフト最小化）。
	tokens float64
	// 直近 update 時刻（refill 計算用）。
	lastRefill time.Time
	// 直近アクセス時刻（idle eviction 用）。
	lastAccess time.Time
	// 1 bucket 内の操作を直列化する。
	mu sync.Mutex
}

// allow は 1 トークンを消費しようとし、可能なら true を返す。
// rate / burst は bucket 共通設定として渡す（per-tenant 設定が docs では想定されていない）。
func (b *tokenBucket) allow(now time.Time, rate, burst float64) bool {
	b.mu.Lock()
	defer b.mu.Unlock()
	// refill: 経過秒 × rate を加算（burst で cap）。
	elapsed := now.Sub(b.lastRefill).Seconds()
	if elapsed > 0 {
		b.tokens += elapsed * rate
		if b.tokens > burst {
			b.tokens = burst
		}
		b.lastRefill = now
	}
	b.lastAccess = now
	if b.tokens < 1.0 {
		return false
	}
	b.tokens -= 1.0
	return true
}

// rateLimiter は tenant_id → tokenBucket map を保持する。
type rateLimiter struct {
	rate    float64
	burst   float64
	idleTTL time.Duration
	// sync.Map は per-key store/load の競合が少ない用途に向く。
	buckets sync.Map // map[string]*tokenBucket
}

// getBucket は tenant に対応する bucket を返す。lazy 生成。
func (l *rateLimiter) getBucket(tenantID string, now time.Time) *tokenBucket {
	if v, ok := l.buckets.Load(tenantID); ok {
		return v.(*tokenBucket)
	}
	// 初回作成時は burst で満タンを与える（最初のリクエストは即受理）。
	nb := &tokenBucket{tokens: l.burst, lastRefill: now, lastAccess: now}
	actual, _ := l.buckets.LoadOrStore(tenantID, nb)
	return actual.(*tokenBucket)
}

// gcOnce は idleTTL を超えてアクセスのない bucket を削除する。
// 起動側 goroutine から定期呼出する想定（テストでは直接呼べる）。
func (l *rateLimiter) gcOnce(now time.Time) {
	l.buckets.Range(func(key, value any) bool {
		b := value.(*tokenBucket)
		b.mu.Lock()
		idle := now.Sub(b.lastAccess) > l.idleTTL
		b.mu.Unlock()
		if idle {
			l.buckets.Delete(key)
		}
		return true
	})
}

// retryAfterMs は次に 1 token が refill されるまでの ms を返す（ResourceExhausted の retry_after_ms 用）。
func retryAfterMs(rate float64) int64 {
	if rate <= 0 {
		return 1000 // 安全側 1 秒
	}
	// 1 token = 1/rate 秒。少しオーバヘッドを入れて切り上げ。
	return int64(1000.0/rate + 1)
}

// RateLimitInterceptor は tenant 単位の token bucket を使う Unary Server Interceptor を返す。
// cfg.RPS <= 0 のときは no-op interceptor を返す（既存テスト / 早期 dev 互換）。
func RateLimitInterceptor(cfg RateLimitConfig) grpc.UnaryServerInterceptor {
	if cfg.RPS <= 0 {
		return func(ctx context.Context, req interface{}, _ *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
			return handler(ctx, req)
		}
	}
	idleTTL := cfg.IdleEvictTTL
	if idleTTL <= 0 {
		idleTTL = 5 * time.Minute
	}
	limiter := &rateLimiter{rate: cfg.RPS, burst: cfg.Burst, idleTTL: idleTTL}
	// 5 分おきに idle 削除を回す（process 終了時 goroutine リーク無し: process 終了で停止）。
	go func() {
		ticker := time.NewTicker(idleTTL)
		defer ticker.Stop()
		for now := range ticker.C {
			limiter.gcOnce(now)
		}
	}()
	return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		// SkipMethods（health / reflection）は素通り。
		if cfg.SkipMethods[info.FullMethod] {
			return handler(ctx, req)
		}
		// tenant_id を解決（AuthInfo > TenantContext）。
		var tenantID string
		if ai, ok := AuthFromContext(ctx); ok {
			tenantID = ai.TenantID
		}
		if tenantID == "" {
			tenantID = extractTenantID(req)
		}
		// tenant_id 不在は rate limit 対象外（AuthInterceptor が前段で弾く前提）。
		if tenantID == "" {
			return handler(ctx, req)
		}
		bucket := limiter.getBucket(tenantID, time.Now())
		if !bucket.allow(time.Now(), cfg.RPS, cfg.Burst) {
			// docs §「エラー型 K1s0Error」: ResourceExhausted は retry_after_ms 必須。
			return nil, status.Errorf(grpccodes.ResourceExhausted,
				"tier1: tenant %q rate limit exceeded (%.0f rps, retry_after_ms=%d)",
				tenantID, cfg.RPS, retryAfterMs(cfg.RPS))
		}
		return handler(ctx, req)
	}
}
