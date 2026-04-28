// 本ファイルは CachedSecretsAdapter の単体テスト。
//
// 検証観点:
//   - cache hit で base 呼出が省略される（FR-T1-SECRETS-001 性能要件）
//   - TTL 経過後は base が再呼出される
//   - Rotate 成功時に同 secret の latest cache が invalidate される
//   - BulkGet / ListAndGet は cache を経由せず常に base を呼ぶ

package openbao

import (
	"context"
	"errors"
	"sync"
	"testing"
	"time"
)

// callCounter は base SecretsAdapter の呼出回数を集計する fake 実装。
type callCounter struct {
	mu          sync.Mutex
	getCalls    int
	rotateCalls int
	bulkCalls   int
	listCalls   int
	// Get で返す値。
	getResponse SecretGetResponse
	// Get で返す err（nil 時は getResponse を返す）。
	getErr error
}

// 4 メソッドを SecretsAdapter として実装する。
func (c *callCounter) Get(_ context.Context, _ SecretGetRequest) (SecretGetResponse, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.getCalls++
	if c.getErr != nil {
		return SecretGetResponse{}, c.getErr
	}
	return c.getResponse, nil
}

func (c *callCounter) BulkGet(_ context.Context, _ []string, _ string) (map[string]SecretGetResponse, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.bulkCalls++
	return map[string]SecretGetResponse{}, nil
}

func (c *callCounter) ListAndGet(_ context.Context, _ string) (map[string]SecretGetResponse, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.listCalls++
	return map[string]SecretGetResponse{}, nil
}

func (c *callCounter) Rotate(_ context.Context, _ SecretRotateRequest) (SecretGetResponse, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.rotateCalls++
	return SecretGetResponse{Version: 2}, nil
}

// cache hit で base が呼ばれないことを検証する。
func TestCache_GetHitSkipsBase(t *testing.T) {
	base := &callCounter{getResponse: SecretGetResponse{Values: map[string]string{"k": "v"}, Version: 1}}
	c := NewCachedSecretsAdapter(base, 1*time.Hour)
	// 1 回目: miss → base 1。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "t"}); err != nil {
		t.Fatalf("first Get: %v", err)
	}
	// 2 回目: hit → base 1（変わらず）。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "t"}); err != nil {
		t.Fatalf("second Get: %v", err)
	}
	// base 呼出は 1 回のはず。
	if base.getCalls != 1 {
		t.Errorf("getCalls: got %d, want 1", base.getCalls)
	}
}

// cache miss → 別キーは hit しない。
func TestCache_DifferentKeyMisses(t *testing.T) {
	base := &callCounter{getResponse: SecretGetResponse{Values: map[string]string{"k": "v"}}}
	c := NewCachedSecretsAdapter(base, 1*time.Hour)
	// tenant T1 で取得する。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "T1"}); err != nil {
		t.Fatalf("first Get: %v", err)
	}
	// tenant T2（別キー）は cache に無いので miss する。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "T2"}); err != nil {
		t.Fatalf("second Get: %v", err)
	}
	// base 呼出は 2 回。
	if base.getCalls != 2 {
		t.Errorf("getCalls: got %d, want 2", base.getCalls)
	}
}

// TTL 経過後は base が再呼出される。
func TestCache_TTLExpiry(t *testing.T) {
	base := &callCounter{getResponse: SecretGetResponse{Values: map[string]string{"k": "v"}}}
	// TTL を 10ms に短縮してテストする。
	c := NewCachedSecretsAdapter(base, 10*time.Millisecond)
	// 1 回目: miss。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "T"}); err != nil {
		t.Fatalf("first Get: %v", err)
	}
	// 期限切れまで待つ。
	time.Sleep(20 * time.Millisecond)
	// 2 回目: 期限切れで miss → base が再呼出される。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "T"}); err != nil {
		t.Fatalf("second Get: %v", err)
	}
	if base.getCalls != 2 {
		t.Errorf("getCalls: got %d, want 2", base.getCalls)
	}
}

// Rotate 成功時に同 tenant_id / name の latest cache が invalidate される。
func TestCache_RotateInvalidatesLatest(t *testing.T) {
	base := &callCounter{getResponse: SecretGetResponse{Values: map[string]string{"k": "v"}, Version: 1}}
	c := NewCachedSecretsAdapter(base, 1*time.Hour)
	// 先に latest を cache に乗せる（version=0）。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "T"}); err != nil {
		t.Fatalf("first Get: %v", err)
	}
	// hit を確認する（base 呼出は 1 回のまま）。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "T"}); err != nil {
		t.Fatalf("second Get: %v", err)
	}
	if base.getCalls != 1 {
		t.Fatalf("hit not effective: got %d, want 1", base.getCalls)
	}
	// Rotate を実行する（成功時に invalidate される）。
	if _, err := c.Rotate(context.Background(), SecretRotateRequest{Name: "n", TenantID: "T"}); err != nil {
		t.Fatalf("rotate: %v", err)
	}
	// latest を再取得 → invalidate されているので miss → base が呼ばれる。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "T"}); err != nil {
		t.Fatalf("third Get: %v", err)
	}
	if base.getCalls != 2 {
		t.Errorf("after rotate: getCalls=%d, want 2 (cache should be invalidated)", base.getCalls)
	}
}

// BulkGet / ListAndGet は cache を bypass する。
func TestCache_BulkAndListBypassCache(t *testing.T) {
	base := &callCounter{getResponse: SecretGetResponse{}}
	c := NewCachedSecretsAdapter(base, 1*time.Hour)
	// BulkGet を 2 回呼ぶ → base 2 回。
	if _, err := c.BulkGet(context.Background(), []string{"a"}, "T"); err != nil {
		t.Fatalf("BulkGet 1: %v", err)
	}
	if _, err := c.BulkGet(context.Background(), []string{"a"}, "T"); err != nil {
		t.Fatalf("BulkGet 2: %v", err)
	}
	if base.bulkCalls != 2 {
		t.Errorf("bulkCalls: got %d, want 2", base.bulkCalls)
	}
	// ListAndGet も同様。
	if _, err := c.ListAndGet(context.Background(), "T"); err != nil {
		t.Fatalf("ListAndGet 1: %v", err)
	}
	if _, err := c.ListAndGet(context.Background(), "T"); err != nil {
		t.Fatalf("ListAndGet 2: %v", err)
	}
	if base.listCalls != 2 {
		t.Errorf("listCalls: got %d, want 2", base.listCalls)
	}
}

// エラーは cache に入らない。
func TestCache_ErrorNotCached(t *testing.T) {
	base := &callCounter{getErr: errors.New("backend error")}
	c := NewCachedSecretsAdapter(base, 1*time.Hour)
	// 1 回目: error。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "T"}); err == nil {
		t.Fatal("expected error")
	}
	// 2 回目: cache されていないので base が再呼出される。
	if _, err := c.Get(context.Background(), SecretGetRequest{Name: "n", TenantID: "T"}); err == nil {
		t.Fatal("expected error")
	}
	if base.getCalls != 2 {
		t.Errorf("getCalls: got %d, want 2 (error should not be cached)", base.getCalls)
	}
}
