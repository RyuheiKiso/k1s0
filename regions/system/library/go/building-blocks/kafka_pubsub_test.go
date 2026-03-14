package buildingblocks

import (
	"context"
	"errors"
	"testing"
	"time"
)

// mockKafkaProducer は KafkaEventProducer のテスト用モック実装。
type mockKafkaProducer struct {
	published []KafkaEventEnvelope
	err       error
	closed    bool
}

func (m *mockKafkaProducer) Publish(_ context.Context, event KafkaEventEnvelope) error {
	if m.err != nil {
		return m.err
	}
	m.published = append(m.published, event)
	return nil
}

func (m *mockKafkaProducer) Close() error {
	m.closed = true
	return nil
}

// mockKafkaConsumer は KafkaEventConsumer のテスト用モック実装。
// Subscribe 時に登録されたハンドラーをフィールドとして保持し、
// テストから任意のタイミングで呼び出せる。
type mockKafkaConsumer struct {
	handlers map[string]KafkaEventHandler
	err      error
	closed   bool
}

// newMockKafkaConsumer はハンドラーマップを初期化した mockKafkaConsumer を生成する。
func newMockKafkaConsumer() *mockKafkaConsumer {
	return &mockKafkaConsumer{handlers: make(map[string]KafkaEventHandler)}
}

func (m *mockKafkaConsumer) Subscribe(_ context.Context, topic string, handler KafkaEventHandler) error {
	if m.err != nil {
		return m.err
	}
	m.handlers[topic] = handler
	return nil
}

func (m *mockKafkaConsumer) Close() error {
	m.closed = true
	return nil
}

// deliver はテスト用: 登録済みハンドラーへメッセージを配信する。
func (m *mockKafkaConsumer) deliver(ctx context.Context, topic string, env KafkaEventEnvelope) error {
	h, ok := m.handlers[topic]
	if !ok {
		return nil
	}
	return h(ctx, env)
}

// TestKafkaPubSub_InitAndStatus は Init 前後でステータスが Uninitialized → Ready に遷移することを検証する。
func TestKafkaPubSub_InitAndStatus(t *testing.T) {
	ps := NewKafkaPubSub("kafka", &mockKafkaProducer{}, newMockKafkaConsumer())
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

// TestKafkaPubSub_NameVersion は Name と Version が正しい値を返すことを検証する。
func TestKafkaPubSub_NameVersion(t *testing.T) {
	ps := NewKafkaPubSub("my-kafka", &mockKafkaProducer{}, nil)
	if ps.Name() != "my-kafka" {
		t.Errorf("unexpected Name: %q", ps.Name())
	}
	if ps.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", ps.Version())
	}
}

// TestKafkaPubSub_Publish はメッセージを Publish するとプロデューサーへ正しいエンベロープが渡されることを検証する。
func TestKafkaPubSub_Publish(t *testing.T) {
	producer := &mockKafkaProducer{}
	ps := NewKafkaPubSub("kafka", producer, nil)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	msg := &Message{Topic: "orders", Data: []byte("payload"), ID: "msg-1", Metadata: map[string]string{"x": "y"}}
	if err := ps.Publish(ctx, msg); err != nil {
		t.Fatalf("Publish failed: %v", err)
	}

	if len(producer.published) != 1 {
		t.Fatalf("expected 1 published event, got %d", len(producer.published))
	}
	env := producer.published[0]
	if env.Topic != "orders" {
		t.Errorf("expected Topic 'orders', got %q", env.Topic)
	}
	if env.Key != "msg-1" {
		t.Errorf("expected Key 'msg-1', got %q", env.Key)
	}
	if string(env.Payload) != "payload" {
		t.Errorf("expected Payload 'payload', got %q", env.Payload)
	}
	if env.Headers["x"] != "y" {
		t.Errorf("expected Header x='y', got %q", env.Headers["x"])
	}
}

// TestKafkaPubSub_PublishError はプロデューサーがエラーを返す場合に Publish がエラーになることを検証する。
func TestKafkaPubSub_PublishError(t *testing.T) {
	producer := &mockKafkaProducer{err: errors.New("kafka error")}
	ps := NewKafkaPubSub("kafka", producer, nil)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	err := ps.Publish(ctx, &Message{Topic: "t", Data: []byte("d")})
	if err == nil {
		t.Fatal("expected error")
	}
}

// TestKafkaPubSub_Subscribe はサブスクライブ後にメッセージが配信されチャネルから受信できることを検証する。
func TestKafkaPubSub_Subscribe(t *testing.T) {
	consumer := newMockKafkaConsumer()
	ps := NewKafkaPubSub("kafka", &mockKafkaProducer{}, consumer)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	ch, err := ps.Subscribe(ctx, "events")
	if err != nil {
		t.Fatalf("Subscribe failed: %v", err)
	}

	// ハンドラーが登録されていることを確認し、テストメッセージを配信する。
	env := KafkaEventEnvelope{Topic: "events", Key: "k1", Payload: []byte("hello")}
	if err := consumer.deliver(ctx, "events", env); err != nil {
		t.Fatalf("deliver failed: %v", err)
	}

	select {
	case received := <-ch:
		if received.ID != "k1" {
			t.Errorf("expected ID 'k1', got %q", received.ID)
		}
		if string(received.Data) != "hello" {
			t.Errorf("expected Data 'hello', got %q", received.Data)
		}
		if received.Timestamp.IsZero() {
			t.Error("expected Timestamp to be set")
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for message")
	}
}

// TestKafkaPubSub_SubscribeWithoutConsumer はコンシューマーが nil のときに Subscribe がエラーになることを検証する。
func TestKafkaPubSub_SubscribeWithoutConsumer(t *testing.T) {
	ps := NewKafkaPubSub("kafka", &mockKafkaProducer{}, nil)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	_, err := ps.Subscribe(ctx, "events")
	if err == nil {
		t.Fatal("expected error when consumer is nil")
	}
}

// TestKafkaPubSub_Close は Close 後にステータスが StatusClosed になり、プロデューサーとコンシューマーが両方クローズされることを検証する。
func TestKafkaPubSub_Close(t *testing.T) {
	producer := &mockKafkaProducer{}
	consumer := newMockKafkaConsumer()
	ps := NewKafkaPubSub("kafka", producer, consumer)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	if err := ps.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if ps.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", ps.Status(ctx))
	}
	if !producer.closed {
		t.Error("expected producer to be closed")
	}
	if !consumer.closed {
		t.Error("expected consumer to be closed")
	}
}

// TestKafkaPubSub_CloseProducerOnly はコンシューマーなしの場合でも Close が正常に完了しプロデューサーがクローズされることを検証する。
func TestKafkaPubSub_CloseProducerOnly(t *testing.T) {
	producer := &mockKafkaProducer{}
	ps := NewKafkaPubSub("kafka", producer, nil)
	ctx := context.Background()
	_ = ps.Init(ctx, Metadata{})

	if err := ps.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if !producer.closed {
		t.Error("expected producer to be closed")
	}
}
