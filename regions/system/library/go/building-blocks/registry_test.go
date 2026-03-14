package buildingblocks

import (
	"context"
	"testing"
)

// テスト用のコンポーネント実装。
type testComponent struct {
	name    string
	version string
}

func newTestComponent(name string) *testComponent {
	return &testComponent{name: name, version: "1.0.0"}
}

func (c *testComponent) Name() string { return c.name }
func (c *testComponent) Version() string { return c.version }
func (c *testComponent) Init(_ context.Context, _ Metadata) error { return nil }
func (c *testComponent) Close(_ context.Context) error { return nil }
func (c *testComponent) Status(_ context.Context) ComponentStatus { return StatusReady }

// コンポーネントを登録して名前で取得できることを確認する。
func TestRegistry_RegisterAndGet(t *testing.T) {
	r := NewComponentRegistry()
	c := newTestComponent("comp-1")

	if err := r.Register(c); err != nil {
		t.Fatalf("Register failed: %v", err)
	}

	got, ok := r.Get("comp-1")
	if !ok {
		t.Fatal("Get returned false for registered component")
	}
	if got.Name() != "comp-1" {
		t.Errorf("expected name 'comp-1', got '%s'", got.Name())
	}
}

// 同名のコンポーネントを重複登録するとエラーが返ることを確認する。
func TestRegistry_RegisterDuplicate(t *testing.T) {
	r := NewComponentRegistry()
	r.Register(newTestComponent("dup"))

	err := r.Register(newTestComponent("dup"))
	if err == nil {
		t.Fatal("expected error on duplicate registration, got nil")
	}
}

// 存在しない名前でコンポーネントを取得すると false が返ることを確認する。
func TestRegistry_GetNotFound(t *testing.T) {
	r := NewComponentRegistry()
	_, ok := r.Get("missing")
	if ok {
		t.Fatal("expected false for missing component")
	}
}

// InitAll が登録済みの全コンポーネントを初期化することを確認する。
func TestRegistry_InitAll(t *testing.T) {
	r := NewComponentRegistry()
	r.Register(newTestComponent("a"))
	r.Register(newTestComponent("b"))

	if err := r.InitAll(context.Background()); err != nil {
		t.Fatalf("InitAll failed: %v", err)
	}
}

// CloseAll が登録済みの全コンポーネントをクローズすることを確認する。
func TestRegistry_CloseAll(t *testing.T) {
	r := NewComponentRegistry()
	r.Register(newTestComponent("a"))

	if err := r.CloseAll(context.Background()); err != nil {
		t.Fatalf("CloseAll failed: %v", err)
	}
}

// StatusAll が全コンポーネントのステータスを返すことを確認する。
func TestRegistry_StatusAll(t *testing.T) {
	r := NewComponentRegistry()
	r.Register(newTestComponent("a"))
	r.Register(newTestComponent("b"))

	statuses := r.StatusAll(context.Background())
	if len(statuses) != 2 {
		t.Fatalf("expected 2 statuses, got %d", len(statuses))
	}
	for _, name := range []string{"a", "b"} {
		if statuses[name] != StatusReady {
			t.Errorf("expected Ready for '%s', got '%s'", name, statuses[name])
		}
	}
}
