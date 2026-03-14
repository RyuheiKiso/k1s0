package buildingblocks

import (
	"context"
	"errors"
	"testing"
	"time"
)

// mockRedisPubSubClient は RedisPubSubClient のテスト用モック実装。
// Subscribe 時に登録されたハンドラーをトピック別に保持し、
// テストから任意のタイミングでメッセージを配信できる。
type mockRedisPubSubClient struct {
	handlers map[string]func(ctx context.Context, payload []byte) error
	pubErr   error
	subErr   error
	closed   bool
}

func newMockRedisPubSubClient() *mockRedisPubSubClient {
	return &mockRedisPubSubClient{
		handlers: make(map[string]func(ctx context.Context, payload []byte) error),
	}
}

func (m *mockRedisPubSubClient) Publish(_ context.Context, _ string, _ []byte) error {
	return m.pubErr
}

func (m *mockRedisPubSubClient) Subscribe(_ context.Context, topic string, handler func(ctx context.Context, payload []byte) error) error {
	if m.subErr != nil {
		return m.subErr
	}
	m.handlers[topic] = handler
	return nil
}

func (m *mockRedisPubSubClient) Close() error {
	m.closed = true
	return nil
}

// deliver はテスト用: 登録済みハンドラーへペイロードを配信する。
func (m *mockRedisPubSubClient) deliver(ctx context.Context, topic string, payload []byte) error {
	h, ok := m.handlers[topic]
	if !ok {
		return nil
	}
	return h(ctx, payload)
}

func TestRedisPubSub_InitAndStatus(t *testing.T) {
	ps := NewRedisPubSub("redis-ps", newMockRedisPubSubClient())
	ctx := context.Background()

	if ps.Status(ctx) != StatusUninitialized {
		t.Errorf("expected StatusUninitialized, got %s", ps.Status(ctx))
	}
	if err := ps.Init(ctx, Metadata{}); err != nil {
		t.Fatalf("Init failed: %v", err)
	}
	if ps.Status(ctx) != StatusReady {
		t.Errorf("expected StatusReady, got %s", ps.Status(ctx))
	}
}

func TestRedisPubSub_NameVersion(t *testing.T) {
	ps := NewRedisPubSub("my-redis-ps", newMockRedisPubSubClient())
	if ps.Name() != "my-redis-ps" {
		t.Errorf("unexpected Name: %q", ps.Name())
	}
	if ps.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", ps.Version())
	}
}

func TestRedisPubSub_Publish(t *testing.T) {
	client := newMockRedisPubSubClient()
	ps := NewRedisPubSub("redis-ps", client)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	msg := &Message{Topic: "events", Data: []byte("hello")}
	if err := ps.Publish(ctx, msg); err != nil {
		t.Fatalf("Publish failed: %v", err)
	}
}

func TestRedisPubSub_PublishError(t *testing.T) {
	client := newMockRedisPubSubClient()
	client.pubErr = errors.New("redis error")
	ps := NewRedisPubSub("redis-ps", client)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	err := ps.Publish(ctx, &Message{Topic: "t", Data: []byte("d")})
	if err == nil {
		t.Fatal("expected error")
	}
}

func TestRedisPubSub_Subscribe(t *testing.T) {
	client := newMockRedisPubSubClient()
	ps := NewRedisPubSub("redis-ps", client)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	ch, err := ps.Subscribe(ctx, "events")
	if err != nil {
		t.Fatalf("Subscribe failed: %v", err)
	}

	if err := client.deliver(ctx, "events", []byte("payload")); err != nil {
		t.Fatalf("deliver failed: %v", err)
	}

	select {
	case received := <-ch:
		if received.Topic != "events" {
			t.Errorf("expected Topic 'events', got %q", received.Topic)
		}
		if string(received.Data) != "payload" {
			t.Errorf("expected Data 'payload', got %q", received.Data)
		}
		if received.Timestamp.IsZero() {
			t.Error("expected Timestamp to be set")
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for message")
	}
}

func TestRedisPubSub_SubscribeError(t *testing.T) {
	client := newMockRedisPubSubClient()
	client.subErr = errors.New("subscribe failed")
	ps := NewRedisPubSub("redis-ps", client)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	_, err := ps.Subscribe(ctx, "events")
	if err == nil {
		t.Fatal("expected error")
	}
}

func TestRedisPubSub_BufferDropOnFull(t *testing.T) {
	client := newMockRedisPubSubClient()
	ps := NewRedisPubSub("redis-ps", client)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	ch, _ := ps.Subscribe(ctx, "t")

	// バッファ（64件）を超えるメッセージを送りドロップが起きてもパニックしないことを確認する。
	for i := 0; i < 70; i++ {
		_ = client.deliver(ctx, "t", []byte("msg"))
	}

	// チャネルに64件以下しか入っていないことを確認する。
	if len(ch) > 64 {
		t.Errorf("channel exceeded buffer size: %d", len(ch))
	}
}

func TestRedisPubSub_Close(t *testing.T) {
	client := newMockRedisPubSubClient()
	ps := NewRedisPubSub("redis-ps", client)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	if err := ps.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if ps.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", ps.Status(ctx))
	}
	if !client.closed {
		t.Error("expected client to be closed")
	}
}
