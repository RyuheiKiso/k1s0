// 本ファイルは RateLimitInterceptor の単体テスト。
//
// 検証観点:
//   - cfg.RPS=0 は no-op（pass-through）
//   - tenant 単位で独立した bucket（A の枯渇が B に波及しない）
//   - burst を超えると ResourceExhausted を返す
//   - SkipMethods は rate limit 対象外
//   - 時間経過で refill が走り再び allow になる

package common

import (
	"context"
	"testing"
	"time"

	"google.golang.org/grpc"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// cfg.RPS=0 は no-op。
func TestRateLimitInterceptor_Disabled_PassThrough(t *testing.T) {
	icpt := RateLimitInterceptor(RateLimitConfig{RPS: 0})
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	for i := 0; i < 1000; i++ {
		_, err := icpt(context.Background(), &fakeRequest{ctx: &fakeTenantContext{tenantID: "T"}}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
			return "ok", nil
		})
		if err != nil {
			t.Fatalf("disabled mode should pass through, got %v at i=%d", err, i)
		}
	}
}

// burst を超えると ResourceExhausted、retry_after_ms 付き。
func TestRateLimitInterceptor_BurstExceeded_ResourceExhausted(t *testing.T) {
	cfg := RateLimitConfig{RPS: 100, Burst: 3}
	icpt := RateLimitInterceptor(cfg)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	req := &fakeRequest{ctx: &fakeTenantContext{tenantID: "T-A"}}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) { return "ok", nil }
	// 最初の burst 分（3 件）は通る。
	for i := 0; i < 3; i++ {
		if _, err := icpt(context.Background(), req, info, handler); err != nil {
			t.Fatalf("request %d should pass: %v", i, err)
		}
	}
	// 4 件目は拒否される（refill が無視できる速度で連発するため）。
	_, err := icpt(context.Background(), req, info, handler)
	if status.Code(err) != grpccodes.ResourceExhausted {
		t.Fatalf("expected ResourceExhausted, got %v", err)
	}
}

// 異なるテナントは独立 bucket。
func TestRateLimitInterceptor_TenantsIndependent(t *testing.T) {
	cfg := RateLimitConfig{RPS: 100, Burst: 2}
	icpt := RateLimitInterceptor(cfg)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) { return "ok", nil }
	// tenant A: burst 2 を使い切る。
	reqA := &fakeRequest{ctx: &fakeTenantContext{tenantID: "T-A"}}
	for i := 0; i < 2; i++ {
		if _, err := icpt(context.Background(), reqA, info, handler); err != nil {
			t.Fatalf("A request %d: %v", i, err)
		}
	}
	// 3 件目の A は拒否。
	if _, err := icpt(context.Background(), reqA, info, handler); status.Code(err) != grpccodes.ResourceExhausted {
		t.Fatalf("A 3rd should be exhausted: %v", err)
	}
	// tenant B は影響なし、burst 2 まで通る。
	reqB := &fakeRequest{ctx: &fakeTenantContext{tenantID: "T-B"}}
	for i := 0; i < 2; i++ {
		if _, err := icpt(context.Background(), reqB, info, handler); err != nil {
			t.Fatalf("B request %d (independent bucket): %v", i, err)
		}
	}
}

// SkipMethods（health）は rate limit 対象外。burst 超でも通る。
func TestRateLimitInterceptor_SkipMethods(t *testing.T) {
	cfg := RateLimitConfig{
		RPS: 100, Burst: 1,
		SkipMethods: map[string]bool{"/grpc.health.v1.Health/Check": true},
	}
	icpt := RateLimitInterceptor(cfg)
	info := &grpc.UnaryServerInfo{FullMethod: "/grpc.health.v1.Health/Check"}
	for i := 0; i < 100; i++ {
		_, err := icpt(context.Background(), nil, info, func(ctx context.Context, req interface{}) (interface{}, error) {
			return "ok", nil
		})
		if err != nil {
			t.Fatalf("skip method should bypass limit, got %v at i=%d", err, i)
		}
	}
}

// time 経過で refill されると再び allow になる。
func TestRateLimitInterceptor_RefillsOverTime(t *testing.T) {
	limiter := &rateLimiter{rate: 100, burst: 1, idleTTL: time.Minute}
	now := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	b := limiter.getBucket("T", now)
	// 1 回目は通る。
	if !b.allow(now, 100, 1) {
		t.Fatalf("1st should allow")
	}
	// 即座の 2 回目は拒否（burst=1）。
	if b.allow(now, 100, 1) {
		t.Fatalf("2nd should reject")
	}
	// 11ms 後（rate=100/sec → 1.1 token 蓄積）に再度 allow。
	later := now.Add(11 * time.Millisecond)
	if !b.allow(later, 100, 1) {
		t.Fatalf("after refill should allow")
	}
}

// gcOnce は idle 超過の bucket を削除する。
func TestRateLimitInterceptor_GC(t *testing.T) {
	limiter := &rateLimiter{rate: 100, burst: 1, idleTTL: time.Second}
	now := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	limiter.getBucket("T1", now)
	// 1 秒経過しただけでは消えない（idleTTL ぴったり）。
	limiter.gcOnce(now.Add(time.Second))
	if _, ok := limiter.buckets.Load("T1"); !ok {
		t.Fatalf("bucket should still exist at idleTTL boundary")
	}
	// 2 秒経過で消える。
	limiter.gcOnce(now.Add(2 * time.Second))
	if _, ok := limiter.buckets.Load("T1"); ok {
		t.Fatalf("bucket should be evicted after idleTTL")
	}
}
