// pubsub_test パッケージは pubsub パッケージの外部テストを提供する。
// InMemoryPubSub、RedisPubSub、KafkaPubSub の各実装の動作を検証する。
package pubsub_test

import (
	"context"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-pubsub"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// ============================================================
// InMemoryPubSub テスト
// ============================================================

// TestInMemoryPubSub_PublishSubscribe は Subscribe 後に Publish したメッセージをチャネルから受信できることを確認する。
func TestInMemoryPubSub_PublishSubscribe(t *testing.T) {
	ps := pubsub.NewInMemoryPubSub()
	ctx := context.Background()
	require.NoError(t, ps.Init(ctx, pubsub.Metadata{}))

	ch, err := ps.Subscribe(ctx, "events")
	require.NoError(t, err)

	msg := &pubsub.Message{Topic: "events", Data: []byte("hello"), ID: "1"}
	require.NoError(t, ps.Publish(ctx, msg))

	select {
	case received := <-ch:
		assert.Equal(t, "1", received.ID)
		assert.Equal(t, []byte("hello"), received.Data)
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for message")
	}
}

// TestInMemoryPubSub_Close は Close 後にサブスクライバーチャネルが閉じられることを確認する。
func TestInMemoryPubSub_Close(t *testing.T) {
	ps := pubsub.NewInMemoryPubSub()
	ctx := context.Background()
	require.NoError(t, ps.Init(ctx, pubsub.Metadata{}))

	ch, err := ps.Subscribe(ctx, "t")
	require.NoError(t, err)

	require.NoError(t, ps.Close(ctx))
	assert.Equal(t, pubsub.StatusClosed, ps.Status(ctx))

	// チャネルが閉じられていることを確認する。
	_, ok := <-ch
	assert.False(t, ok, "expected channel to be closed")
}

// TestInMemoryPubSub_NonBlocking はバッファが満杯の場合でも Publish がブロックしないことを確認する。
func TestInMemoryPubSub_NonBlocking(t *testing.T) {
	ps := pubsub.NewInMemoryPubSub()
	ctx := context.Background()
	require.NoError(t, ps.Init(ctx, pubsub.Metadata{}))

	ch, err := ps.Subscribe(ctx, "t")
	require.NoError(t, err)

	// バッファ（64件）を超えるメッセージを送信してもパニックしないことを確認する。
	for i := range 70 {
		msg := &pubsub.Message{Topic: "t", Data: []byte("msg"), ID: string(rune('0' + i%10))}
		require.NoError(t, ps.Publish(ctx, msg))
	}

	// チャネルのバッファサイズが上限を超えていないことを確認する。
	assert.LessOrEqual(t, len(ch), 64)
}

// ============================================================
// RedisPubSub テスト用モック
// ============================================================

// mockRedisPubSubClient は RedisPubSubClient のテスト用モック実装。
// Subscribe 時に登録されたハンドラーをトピック別に保持し、
// テストから任意のタイミングでメッセージを配信できる。
type mockRedisPubSubClient struct {
	handlers map[string]func(ctx context.Context, payload []byte) error
	pubCalls int
	subCalls int
	closed   bool
}

// newMockRedisPubSubClient はハンドラーマップを初期化した mockRedisPubSubClient を生成する。
func newMockRedisPubSubClient() *mockRedisPubSubClient {
	return &mockRedisPubSubClient{
		handlers: make(map[string]func(ctx context.Context, payload []byte) error),
	}
}

func (m *mockRedisPubSubClient) Publish(_ context.Context, _ string, _ []byte) error {
	m.pubCalls++
	return nil
}

func (m *mockRedisPubSubClient) Subscribe(_ context.Context, topic string, handler func(ctx context.Context, payload []byte) error) error {
	m.subCalls++
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

// ============================================================
// RedisPubSub テスト
// ============================================================

// TestRedisPubSub_Publish はモッククライアントに対して Publish が呼ばれることを確認する。
func TestRedisPubSub_Publish(t *testing.T) {
	client := newMockRedisPubSubClient()
	ps := pubsub.NewRedisPubSub("redis-ps", client)
	ctx := context.Background()
	require.NoError(t, ps.Init(ctx, pubsub.Metadata{}))

	msg := &pubsub.Message{Topic: "events", Data: []byte("hello")}
	require.NoError(t, ps.Publish(ctx, msg))

	// モッククライアントの Publish が1回呼ばれたことを確認する。
	assert.Equal(t, 1, client.pubCalls)
}

// TestRedisPubSub_Subscribe はモッククライアントにハンドラーが登録されることを確認する。
func TestRedisPubSub_Subscribe(t *testing.T) {
	client := newMockRedisPubSubClient()
	ps := pubsub.NewRedisPubSub("redis-ps", client)
	ctx := context.Background()
	require.NoError(t, ps.Init(ctx, pubsub.Metadata{}))

	ch, err := ps.Subscribe(ctx, "events")
	require.NoError(t, err)
	assert.NotNil(t, ch)

	// モッククライアントの Subscribe が1回呼ばれたことを確認する。
	assert.Equal(t, 1, client.subCalls)

	// ハンドラーを経由してメッセージを受信できることを確認する。
	require.NoError(t, client.deliver(ctx, "events", []byte("payload")))

	select {
	case received := <-ch:
		assert.Equal(t, "events", received.Topic)
		assert.Equal(t, []byte("payload"), received.Data)
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for message")
	}
}

// ============================================================
// KafkaPubSub テスト用モック
// ============================================================

// mockKafkaProducer は KafkaEventProducer のテスト用モック実装。
type mockKafkaProducer struct {
	published []pubsub.KafkaEventEnvelope
	closed    bool
}

func (m *mockKafkaProducer) Publish(_ context.Context, event pubsub.KafkaEventEnvelope) error {
	m.published = append(m.published, event)
	return nil
}

func (m *mockKafkaProducer) Close() error {
	m.closed = true
	return nil
}

// mockKafkaConsumer は KafkaEventConsumer のテスト用モック実装。
// Subscribe 時に登録されたハンドラーをトピック別に保持する。
type mockKafkaConsumer struct {
	handlers map[string]pubsub.KafkaEventHandler
	closed   bool
}

// newMockKafkaConsumer はハンドラーマップを初期化した mockKafkaConsumer を生成する。
func newMockKafkaConsumer() *mockKafkaConsumer {
	return &mockKafkaConsumer{handlers: make(map[string]pubsub.KafkaEventHandler)}
}

func (m *mockKafkaConsumer) Subscribe(_ context.Context, topic string, handler pubsub.KafkaEventHandler) error {
	m.handlers[topic] = handler
	return nil
}

func (m *mockKafkaConsumer) Close() error {
	m.closed = true
	return nil
}

// deliver はテスト用: 登録済みハンドラーへメッセージを配信する。
func (m *mockKafkaConsumer) deliver(ctx context.Context, topic string, env pubsub.KafkaEventEnvelope) error {
	h, ok := m.handlers[topic]
	if !ok {
		return nil
	}
	return h(ctx, env)
}

// ============================================================
// KafkaPubSub テスト
// ============================================================

// TestKafkaPubSub_Publish はモックプロデューサーに対して Publish が呼ばれることを確認する。
func TestKafkaPubSub_Publish(t *testing.T) {
	producer := &mockKafkaProducer{}
	ps := pubsub.NewKafkaPubSub("kafka", producer, nil)
	ctx := context.Background()
	require.NoError(t, ps.Init(ctx, pubsub.Metadata{}))

	msg := &pubsub.Message{Topic: "orders", Data: []byte("payload"), ID: "msg-1", Metadata: map[string]string{"x": "y"}}
	require.NoError(t, ps.Publish(ctx, msg))

	// プロデューサーに1件のメッセージが送信されたことを確認する。
	require.Len(t, producer.published, 1)
	env := producer.published[0]
	assert.Equal(t, "orders", env.Topic)
	assert.Equal(t, "msg-1", env.Key)
	assert.Equal(t, []byte("payload"), env.Payload)
	assert.Equal(t, "y", env.Headers["x"])
}

// TestKafkaPubSub_Subscribe_NoConsumer は consumer=nil の場合に Subscribe がエラーを返すことを確認する。
func TestKafkaPubSub_Subscribe_NoConsumer(t *testing.T) {
	ps := pubsub.NewKafkaPubSub("kafka", &mockKafkaProducer{}, nil)
	ctx := context.Background()
	require.NoError(t, ps.Init(ctx, pubsub.Metadata{}))

	_, err := ps.Subscribe(ctx, "events")
	assert.Error(t, err, "expected error when consumer is nil")
}

// TestKafkaPubSub_Subscribe はモックコンシューマーにハンドラーが登録され、メッセージを受信できることを確認する。
func TestKafkaPubSub_Subscribe(t *testing.T) {
	consumer := newMockKafkaConsumer()
	ps := pubsub.NewKafkaPubSub("kafka", &mockKafkaProducer{}, consumer)
	ctx := context.Background()
	require.NoError(t, ps.Init(ctx, pubsub.Metadata{}))

	ch, err := ps.Subscribe(ctx, "events")
	require.NoError(t, err)
	assert.NotNil(t, ch)

	// ハンドラーを経由してメッセージを受信できることを確認する。
	env := pubsub.KafkaEventEnvelope{Topic: "events", Key: "k1", Payload: []byte("hello")}
	require.NoError(t, consumer.deliver(ctx, "events", env))

	select {
	case received := <-ch:
		assert.Equal(t, "k1", received.ID)
		assert.Equal(t, []byte("hello"), received.Data)
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for message")
	}
}
