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

// newMockRedisPubSubClient はハンドラーマップを初期化した mockRedisPubSubClient を生成する。
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

// TestRedisPubSub_InitAndStatus は Init 前後でステータスが Uninitialized → Ready に遷移することを検証する。
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

// TestRedisPubSub_NameVersion は Name と Version が正しい値を返すことを検証する。
func TestRedisPubSub_NameVersion(t *testing.T) {
	ps := NewRedisPubSub("my-redis-ps", newMockRedisPubSubClient())
	if ps.Name() != "my-redis-ps" {
		t.Errorf("unexpected Name: %q", ps.Name())
	}
	if ps.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", ps.Version())
	}
}

// TestRedisPubSub_Publish はメッセージを Publish してもエラーが発生しないことを検証する。
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

// TestRedisPubSub_PublishError は Redis クライアントが Publish エラーを返す場合にエラーになることを検証する。
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

// TestRedisPubSub_Subscribe はサブスクライブ後にメッセージが配信されチャネルから受信できることを検証する。
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

// TestRedisPubSub_SubscribeError は Redis クライアントが Subscribe エラーを返す場合にエラーになることを検証する。
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

// TestRedisPubSub_BufferDropOnFull はチャネルバッファが満杯のときにメッセージがドロップされてもパニックが発生しないことを検証する。
func TestRedisPubSub_BufferDropOnFull(t *testing.T) {
	client := newMockRedisPubSubClient()
	ps := NewRedisPubSub("redis-ps", client)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	ch, _ := ps.Subscribe(ctx, "t")

	// バッファ（64件）を超えるメッセージを送りドロップが起きてもパニックしないことを確認する。
	for range 70 {
		_ = client.deliver(ctx, "t", []byte("msg"))
	}

	// チャネルに64件以下しか入っていないことを確認する。
	if len(ch) > 64 {
		t.Errorf("channel exceeded buffer size: %d", len(ch))
	}
}

// TestRedisPubSub_Close は Close 後にステータスが StatusClosed になりクライアントがクローズされることを検証する。
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
