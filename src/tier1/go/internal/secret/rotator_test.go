// 本ファイルは Rotator（FR-T1-SECRETS-004）の単体テスト。
//
// 検証観点:
//   - parseRotationSchedule の正常系 / 異常系
//   - Targets() がスケジュール内容を反映する
//   - Start → ticker 発火で Rotate が呼ばれる（fake adapter で観測）
//   - Stop で goroutine が graceful 終了する

package secret

import (
	"context"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/openbao"
)

// rotatorFakeAdapter は SecretsAdapter の最小モック。Rotate 呼出回数を記録する。
type rotatorFakeAdapter struct {
	mu          sync.Mutex
	rotateCalls atomic.Int32
	lastReq     openbao.SecretRotateRequest
	// 返却 version。
	version int32
}

func (f *rotatorFakeAdapter) Get(_ context.Context, _ openbao.SecretGetRequest) (openbao.SecretGetResponse, error) {
	return openbao.SecretGetResponse{}, nil
}
func (f *rotatorFakeAdapter) BulkGet(_ context.Context, _ []string, _ string) (map[string]openbao.SecretGetResponse, error) {
	return nil, nil
}
func (f *rotatorFakeAdapter) ListAndGet(_ context.Context, _ string) (map[string]openbao.SecretGetResponse, error) {
	return nil, nil
}
func (f *rotatorFakeAdapter) Rotate(_ context.Context, req openbao.SecretRotateRequest) (openbao.SecretGetResponse, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.rotateCalls.Add(1)
	f.lastReq = req
	f.version++
	return openbao.SecretGetResponse{Version: f.version}, nil
}

// 正常系: 単一エントリ。
func TestParseRotationSchedule_Single(t *testing.T) {
	got, err := parseRotationSchedule("t1/db-password@30s")
	if err != nil {
		t.Fatalf("parse err: %v", err)
	}
	if len(got) != 1 || got[0].tenantID != "t1" || got[0].name != "db-password" || got[0].interval != 30*time.Second {
		t.Errorf("unexpected: %+v", got)
	}
}

// 正常系: 複数エントリ。
func TestParseRotationSchedule_Multiple(t *testing.T) {
	got, err := parseRotationSchedule("t1/db-password@1h, t2/api-key@5m")
	if err != nil {
		t.Fatalf("parse err: %v", err)
	}
	if len(got) != 2 {
		t.Fatalf("got %d targets, want 2", len(got))
	}
	if got[1].tenantID != "t2" || got[1].name != "api-key" || got[1].interval != 5*time.Minute {
		t.Errorf("unexpected entry 1: %+v", got[1])
	}
}

// 空文字 / 空白のみは空 slice。
func TestParseRotationSchedule_Empty(t *testing.T) {
	for _, s := range []string{"", "   ", "\t"} {
		got, err := parseRotationSchedule(s)
		if err != nil {
			t.Fatalf("parse err for %q: %v", s, err)
		}
		if len(got) != 0 {
			t.Errorf("expected empty for %q, got %+v", s, got)
		}
	}
}

// 異常系: 形式不正。
func TestParseRotationSchedule_InvalidFormat(t *testing.T) {
	bad := []string{
		"no-at-sign",
		"@1h",
		"t1/n@notduration",
		"t1/n@0s",
		"t1/n@-1s",
		"t1/@30s",
		"/n@30s",
	}
	for _, s := range bad {
		if _, err := parseRotationSchedule(s); err == nil {
			t.Errorf("expected error for %q", s)
		}
	}
}

// nil / 空 schedule で Start しても goroutine は立ち上がらない。
func TestRotator_NoOpWhenEmpty(t *testing.T) {
	r, err := NewRotatorFromEnv(&rotatorFakeAdapter{}, "")
	if err != nil {
		t.Fatalf("constructor err: %v", err)
	}
	r.Start(context.Background())
	// targets 0 で起動しないので Stop も no-op。
	r.Stop()
}

// Start → tick で adapter.Rotate が呼ばれる。
func TestRotator_TicksFire(t *testing.T) {
	adapter := &rotatorFakeAdapter{}
	// 50ms 間隔で fire させる（テストで観測しやすい間隔）。
	r, err := NewRotatorFromEnv(adapter, "t1/db@50ms")
	if err != nil {
		t.Fatalf("constructor err: %v", err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	r.Start(ctx)
	// 200ms 待つ → tick が 3 回程度発火するはず。
	time.Sleep(200 * time.Millisecond)
	cancel()
	r.Stop()
	calls := adapter.rotateCalls.Load()
	if calls < 2 {
		t.Errorf("expected at least 2 ticks in 200ms with 50ms interval, got %d", calls)
	}
	if adapter.lastReq.TenantID != "t1" || adapter.lastReq.Name != "db" {
		t.Errorf("unexpected last req: %+v", adapter.lastReq)
	}
}
