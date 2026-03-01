package outbox_test

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	outbox "github.com/k1s0-platform/system-library-go-outbox"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// --- インラインモック ---

type mockStore struct {
	messages      []outbox.OutboxMessage
	savedMessages []outbox.OutboxMessage
	updatedMsgs   []outbox.OutboxMessage
	saveErr       error
	getErr        error
	updateErr     error
	deleteCount   int64
	deleteErr     error
}

func (m *mockStore) Save(ctx context.Context, msg *outbox.OutboxMessage) error {
	if m.saveErr != nil {
		return m.saveErr
	}
	m.savedMessages = append(m.savedMessages, *msg)
	return nil
}

func (m *mockStore) FetchPending(ctx context.Context, limit int) ([]outbox.OutboxMessage, error) {
	if m.getErr != nil {
		return nil, m.getErr
	}
	if len(m.messages) > limit {
		return m.messages[:limit], nil
	}
	return m.messages, nil
}

func (m *mockStore) Update(ctx context.Context, msg *outbox.OutboxMessage) error {
	if m.updateErr != nil {
		return m.updateErr
	}
	m.updatedMsgs = append(m.updatedMsgs, *msg)
	return nil
}

func (m *mockStore) DeleteDelivered(ctx context.Context, olderThanDays int) (int64, error) {
	if m.deleteErr != nil {
		return 0, m.deleteErr
	}
	return m.deleteCount, nil
}

type mockPublisher struct {
	published []outbox.OutboxMessage
	err       error
}

func (m *mockPublisher) Publish(ctx context.Context, msg *outbox.OutboxMessage) error {
	if m.err != nil {
		return m.err
	}
	m.published = append(m.published, *msg)
	return nil
}

// --- OutboxMessage テスト ---

func TestNewOutboxMessage(t *testing.T) {
	payload := json.RawMessage(`{"order_id":"ord-001"}`)
	msg := outbox.NewOutboxMessage("k1s0.service.order.created.v1", "ord-001", payload)
	assert.NotEmpty(t, msg.ID)
	assert.Equal(t, "k1s0.service.order.created.v1", msg.Topic)
	assert.Equal(t, "ord-001", msg.PartitionKey)
	assert.Equal(t, outbox.OutboxStatusPending, msg.Status)
	assert.Equal(t, 0, msg.RetryCount)
	assert.Equal(t, 3, msg.MaxRetries)
	assert.Empty(t, msg.LastError)
	assert.False(t, msg.CreatedAt.IsZero())
	assert.True(t, msg.IsProcessable())
}

func TestNewOutboxMessage_UniqueIds(t *testing.T) {
	payload := json.RawMessage(`{}`)
	msg1 := outbox.NewOutboxMessage("topic", "key", payload)
	msg2 := outbox.NewOutboxMessage("topic", "key", payload)
	assert.NotEqual(t, msg1.ID, msg2.ID)
}

func TestNewOutboxMessage_TimestampIsUTC(t *testing.T) {
	payload := json.RawMessage(`{}`)
	msg := outbox.NewOutboxMessage("topic", "key", payload)
	assert.Equal(t, time.UTC, msg.CreatedAt.Location())
	assert.Equal(t, time.UTC, msg.ProcessAfter.Location())
}

func TestOutboxMessage_MarkDelivered(t *testing.T) {
	payload := json.RawMessage(`{}`)
	msg := outbox.NewOutboxMessage("test.topic", "key", payload)
	msg.MarkDelivered()
	assert.Equal(t, outbox.OutboxStatusDelivered, msg.Status)
	assert.False(t, msg.IsProcessable())
}

func TestOutboxMessage_MarkFailed_IncrementsRetry(t *testing.T) {
	payload := json.RawMessage(`{}`)
	msg := outbox.NewOutboxMessage("test.topic", "key", payload)
	msg.MarkFailed("kafka error")
	assert.Equal(t, 1, msg.RetryCount)
	assert.Equal(t, outbox.OutboxStatusFailed, msg.Status)
	assert.Equal(t, "kafka error", msg.LastError)
}

func TestOutboxMessage_MarkFailed_DeadLetterOnMaxRetries(t *testing.T) {
	payload := json.RawMessage(`{}`)
	msg := outbox.NewOutboxMessage("test.topic", "key", payload)
	msg.MaxRetries = 3
	msg.MarkFailed("error 1")
	msg.MarkFailed("error 2")
	msg.MarkFailed("error 3")
	assert.Equal(t, outbox.OutboxStatusDeadLetter, msg.Status)
	assert.Equal(t, 3, msg.RetryCount)
}

func TestOutboxMessage_MarkFailed_BackoffInSeconds(t *testing.T) {
	payload := json.RawMessage(`{}`)
	msg := outbox.NewOutboxMessage("test.topic", "key", payload)
	msg.MaxRetries = 10 // 高めに設定して DeadLetter にならないようにする
	before := time.Now().UTC()
	msg.MarkFailed("error")
	// retry_count=1 なので 2^1 = 2秒のバックオフ
	assert.True(t, msg.ProcessAfter.After(before), "ProcessAfter should be in the future")
	// 最大でも 2秒 + 少しの余裕
	assert.True(t, msg.ProcessAfter.Before(before.Add(5*time.Second)), "backoff should be in seconds, not minutes")
}

func TestOutboxMessage_IsProcessable(t *testing.T) {
	payload := json.RawMessage(`{}`)

	// Pending + ProcessAfter in the past → processable
	msg := outbox.NewOutboxMessage("test.topic", "key", payload)
	assert.True(t, msg.IsProcessable())

	// Processing → not processable
	msg2 := outbox.NewOutboxMessage("test.topic", "key", payload)
	msg2.MarkProcessing()
	assert.False(t, msg2.IsProcessable())

	// Delivered → not processable
	msg3 := outbox.NewOutboxMessage("test.topic", "key", payload)
	msg3.MarkDelivered()
	assert.False(t, msg3.IsProcessable())

	// DeadLetter → not processable
	msg4 := outbox.NewOutboxMessage("test.topic", "key", payload)
	msg4.MaxRetries = 1
	msg4.MarkFailed("error")
	assert.Equal(t, outbox.OutboxStatusDeadLetter, msg4.Status)
	assert.False(t, msg4.IsProcessable())
}

// --- OutboxStatus テスト ---

func TestOutboxStatus_Constants(t *testing.T) {
	assert.Equal(t, outbox.OutboxStatus("PENDING"), outbox.OutboxStatusPending)
	assert.Equal(t, outbox.OutboxStatus("PROCESSING"), outbox.OutboxStatusProcessing)
	assert.Equal(t, outbox.OutboxStatus("DELIVERED"), outbox.OutboxStatusDelivered)
	assert.Equal(t, outbox.OutboxStatus("FAILED"), outbox.OutboxStatusFailed)
	assert.Equal(t, outbox.OutboxStatus("DEAD_LETTER"), outbox.OutboxStatusDeadLetter)
}

// --- OutboxError テスト ---

func TestOutboxError_StoreError(t *testing.T) {
	cause := errors.New("connection refused")
	err := outbox.NewStoreError("save failed", cause)
	assert.Contains(t, err.Error(), "store error")
	assert.Contains(t, err.Error(), "save failed")
	assert.Contains(t, err.Error(), "connection refused")
	assert.ErrorIs(t, err, cause)
}

func TestOutboxError_PublishError(t *testing.T) {
	cause := errors.New("kafka unavailable")
	err := outbox.NewPublishError("publish failed", cause)
	assert.Contains(t, err.Error(), "publish error")
	assert.Contains(t, err.Error(), "publish failed")
	assert.ErrorIs(t, err, cause)
}

func TestOutboxError_SerializationError(t *testing.T) {
	cause := errors.New("invalid json")
	err := outbox.NewSerializationError("marshal failed", cause)
	assert.Contains(t, err.Error(), "serialization error")
	assert.Contains(t, err.Error(), "marshal failed")
	assert.ErrorIs(t, err, cause)
}

func TestOutboxError_NotFound(t *testing.T) {
	err := outbox.NewNotFoundError("msg-123")
	assert.Contains(t, err.Error(), "message not found")
	assert.Contains(t, err.Error(), "msg-123")
}

// --- OutboxStore テスト ---

func TestMockStore_Save(t *testing.T) {
	store := &mockStore{}
	payload := json.RawMessage(`{}`)
	msg := outbox.NewOutboxMessage("test.topic", "key", payload)
	err := store.Save(context.Background(), &msg)
	require.NoError(t, err)
	require.Len(t, store.savedMessages, 1)
}

func TestMockStore_FetchPending(t *testing.T) {
	payload := json.RawMessage(`{}`)
	msg := outbox.NewOutboxMessage("test.topic", "key", payload)
	store := &mockStore{messages: []outbox.OutboxMessage{msg}}

	result, err := store.FetchPending(context.Background(), 10)
	require.NoError(t, err)
	require.Len(t, result, 1)
	assert.Equal(t, outbox.OutboxStatusPending, result[0].Status)
}

func TestMockStore_Update(t *testing.T) {
	store := &mockStore{}
	payload := json.RawMessage(`{}`)
	msg := outbox.NewOutboxMessage("test.topic", "key", payload)
	msg.MarkDelivered()
	err := store.Update(context.Background(), &msg)
	require.NoError(t, err)
	require.Len(t, store.updatedMsgs, 1)
	assert.Equal(t, outbox.OutboxStatusDelivered, store.updatedMsgs[0].Status)
}

func TestMockStore_DeleteDelivered(t *testing.T) {
	store := &mockStore{deleteCount: 5}
	count, err := store.DeleteDelivered(context.Background(), 30)
	require.NoError(t, err)
	assert.Equal(t, int64(5), count)
}

// --- OutboxProcessor テスト ---

func TestProcessBatch_Success(t *testing.T) {
	payload := json.RawMessage(`{"key":"value"}`)
	msg := outbox.NewOutboxMessage("topic", "key-1", payload)
	store := &mockStore{messages: []outbox.OutboxMessage{msg}}
	publisher := &mockPublisher{}

	processor := outbox.NewOutboxProcessor(store, publisher, 10)
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 1, count)

	// Publish が呼ばれていることを確認
	require.Len(t, publisher.published, 1)
	assert.Equal(t, msg.Topic, publisher.published[0].Topic)

	// Processing → Delivered の 2 回更新されていることを確認
	require.Len(t, store.updatedMsgs, 2)
	assert.Equal(t, outbox.OutboxStatusProcessing, store.updatedMsgs[0].Status)
	assert.Equal(t, outbox.OutboxStatusDelivered, store.updatedMsgs[1].Status)
}

func TestProcessBatch_PublishFails(t *testing.T) {
	payload := json.RawMessage(`{}`)
	msg := outbox.NewOutboxMessage("topic", "key-1", payload)
	store := &mockStore{messages: []outbox.OutboxMessage{msg}}
	publisher := &mockPublisher{err: errors.New("kafka unavailable")}

	processor := outbox.NewOutboxProcessor(store, publisher, 10)
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err) // バッチ全体はエラーにならない
	assert.Equal(t, 0, count)

	// Processing → Failed に更新されていることを確認
	require.Len(t, store.updatedMsgs, 2)
	assert.Equal(t, outbox.OutboxStatusProcessing, store.updatedMsgs[0].Status)
	assert.Equal(t, outbox.OutboxStatusFailed, store.updatedMsgs[1].Status)
	assert.Equal(t, 1, store.updatedMsgs[1].RetryCount)
	assert.Equal(t, "kafka unavailable", store.updatedMsgs[1].LastError)
}

func TestProcessBatch_StoreError(t *testing.T) {
	store := &mockStore{getErr: errors.New("db error")}
	publisher := &mockPublisher{}

	processor := outbox.NewOutboxProcessor(store, publisher, 10)
	_, err := processor.ProcessBatch(context.Background())
	assert.Error(t, err)
}

func TestProcessBatch_Empty(t *testing.T) {
	store := &mockStore{messages: []outbox.OutboxMessage{}}
	publisher := &mockPublisher{}

	processor := outbox.NewOutboxProcessor(store, publisher, 10)
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 0, count)
}

func TestProcessBatch_RespectsBatchSize(t *testing.T) {
	payload := json.RawMessage(`{}`)
	messages := make([]outbox.OutboxMessage, 5)
	for i := range messages {
		messages[i] = outbox.NewOutboxMessage("topic", "key", payload)
	}
	store := &mockStore{messages: messages}
	publisher := &mockPublisher{}

	processor := outbox.NewOutboxProcessor(store, publisher, 3) // batch size = 3
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 3, count) // 3 件のみ処理
}

func TestProcessBatch_MultipleMessages(t *testing.T) {
	msgs := []outbox.OutboxMessage{
		outbox.NewOutboxMessage("topic", "key-1", json.RawMessage(`{"n":1}`)),
		outbox.NewOutboxMessage("topic", "key-2", json.RawMessage(`{"n":2}`)),
		outbox.NewOutboxMessage("topic", "key-3", json.RawMessage(`{"n":3}`)),
	}
	store := &mockStore{messages: msgs}
	publisher := &mockPublisher{}

	processor := outbox.NewOutboxProcessor(store, publisher, 10)
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 3, count)
	assert.Len(t, publisher.published, 3)
	// 各メッセージが Processing → Delivered の 2 回ずつ更新される
	assert.Len(t, store.updatedMsgs, 6)
}

func TestProcessBatch_PartialSuccess(t *testing.T) {
	// 2 件のメッセージのうち 2 件目が失敗するケース
	msgs := []outbox.OutboxMessage{
		outbox.NewOutboxMessage("topic", "key-1", json.RawMessage(`{"n":1}`)),
		outbox.NewOutboxMessage("topic", "key-2", json.RawMessage(`{"n":2}`)),
	}
	store := &mockStore{messages: msgs}
	callCount := 0
	pub := &mockPublisherFunc{fn: func(ctx context.Context, msg *outbox.OutboxMessage) error {
		callCount++
		if callCount == 2 {
			return errors.New("second message failed")
		}
		return nil
	}}

	processor := outbox.NewOutboxProcessor(store, pub, 10)
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 1, count) // 1 件のみ成功
}

type mockPublisherFunc struct {
	fn func(ctx context.Context, msg *outbox.OutboxMessage) error
}

func (m *mockPublisherFunc) Publish(ctx context.Context, msg *outbox.OutboxMessage) error {
	return m.fn(ctx, msg)
}

func TestOutboxProcessor_DefaultBatchSize(t *testing.T) {
	// batchSize = 0 の場合はデフォルト 100 が使われる
	payload := json.RawMessage(`{}`)
	msgs := make([]outbox.OutboxMessage, 5)
	for i := range msgs {
		msgs[i] = outbox.NewOutboxMessage("topic", "key", payload)
	}
	store := &mockStore{messages: msgs}
	publisher := &mockPublisher{}

	processor := outbox.NewOutboxProcessor(store, publisher, 0) // 0 → default 100
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 5, count) // 全件処理される（100 > 5）
}
