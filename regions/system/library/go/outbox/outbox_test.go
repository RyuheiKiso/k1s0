package outbox_test

import (
	"context"
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
	statusUpdates []statusUpdate
	saveErr       error
	getErr        error
	updateErr     error
}

type statusUpdate struct {
	id          string
	status      outbox.OutboxStatus
	retryCount  int
	scheduledAt time.Time
}

func (m *mockStore) SaveMessage(ctx context.Context, msg outbox.OutboxMessage) error {
	if m.saveErr != nil {
		return m.saveErr
	}
	m.savedMessages = append(m.savedMessages, msg)
	return nil
}

func (m *mockStore) GetPendingMessages(ctx context.Context, limit int) ([]outbox.OutboxMessage, error) {
	if m.getErr != nil {
		return nil, m.getErr
	}
	if len(m.messages) > limit {
		return m.messages[:limit], nil
	}
	return m.messages, nil
}

func (m *mockStore) UpdateStatus(ctx context.Context, id string, status outbox.OutboxStatus) error {
	if m.updateErr != nil {
		return m.updateErr
	}
	m.statusUpdates = append(m.statusUpdates, statusUpdate{id: id, status: status})
	return nil
}

func (m *mockStore) UpdateStatusWithRetry(ctx context.Context, id string, status outbox.OutboxStatus, retryCount int, scheduledAt time.Time) error {
	if m.updateErr != nil {
		return m.updateErr
	}
	m.statusUpdates = append(m.statusUpdates, statusUpdate{
		id: id, status: status, retryCount: retryCount, scheduledAt: scheduledAt,
	})
	return nil
}

type mockPublisher struct {
	published []outbox.OutboxMessage
	err       error
}

func (m *mockPublisher) Publish(ctx context.Context, msg outbox.OutboxMessage) error {
	if m.err != nil {
		return m.err
	}
	m.published = append(m.published, msg)
	return nil
}

// --- テスト ---

func TestNewOutboxMessage(t *testing.T) {
	msg := outbox.NewOutboxMessage("k1s0.system.user.created.v1", "user.created.v1", `{"id":"1"}`, "corr-123")
	assert.NotEmpty(t, msg.ID)
	assert.Equal(t, "k1s0.system.user.created.v1", msg.Topic)
	assert.Equal(t, "user.created.v1", msg.EventType)
	assert.Equal(t, outbox.OutboxStatusPending, msg.Status)
	assert.Equal(t, 0, msg.RetryCount)
	assert.Equal(t, "corr-123", msg.CorrelationId)
	assert.False(t, msg.CreatedAt.IsZero())
}

func TestNewOutboxMessage_UniqueIds(t *testing.T) {
	msg1 := outbox.NewOutboxMessage("topic", "event.v1", `{}`, "corr-1")
	msg2 := outbox.NewOutboxMessage("topic", "event.v1", `{}`, "corr-1")
	assert.NotEqual(t, msg1.ID, msg2.ID)
}

func TestNextScheduledAt(t *testing.T) {
	tests := []struct {
		retryCount int
		minDelay   time.Duration
		maxDelay   time.Duration
	}{
		{0, 1 * time.Minute, 2 * time.Minute},   // 2^0 = 1 分
		{1, 2 * time.Minute, 3 * time.Minute},   // 2^1 = 2 分
		{2, 4 * time.Minute, 5 * time.Minute},   // 2^2 = 4 分
		{3, 8 * time.Minute, 9 * time.Minute},   // 2^3 = 8 分
		{6, 60 * time.Minute, 61 * time.Minute}, // 64 → 60 分上限
		{7, 60 * time.Minute, 61 * time.Minute}, // 128 → 60 分上限
	}
	for _, tt := range tests {
		scheduled := outbox.NextScheduledAt(tt.retryCount)
		delay := time.Until(scheduled)
		assert.GreaterOrEqual(t, delay, tt.minDelay-time.Second, "retry %d: delay too short", tt.retryCount)
		assert.LessOrEqual(t, delay, tt.maxDelay, "retry %d: delay too long", tt.retryCount)
	}
}

func TestOutboxStatus_CanTransitionTo(t *testing.T) {
	// 有効な遷移
	assert.True(t, outbox.OutboxStatusPending.CanTransitionTo(outbox.OutboxStatusProcessing))
	assert.True(t, outbox.OutboxStatusProcessing.CanTransitionTo(outbox.OutboxStatusDelivered))
	assert.True(t, outbox.OutboxStatusProcessing.CanTransitionTo(outbox.OutboxStatusFailed))
	assert.True(t, outbox.OutboxStatusFailed.CanTransitionTo(outbox.OutboxStatusPending))

	// 無効な遷移
	assert.False(t, outbox.OutboxStatusDelivered.CanTransitionTo(outbox.OutboxStatusPending))
	assert.False(t, outbox.OutboxStatusDelivered.CanTransitionTo(outbox.OutboxStatusFailed))
	assert.False(t, outbox.OutboxStatusPending.CanTransitionTo(outbox.OutboxStatusDelivered))
	assert.False(t, outbox.OutboxStatusPending.CanTransitionTo(outbox.OutboxStatusFailed))
}

func TestProcessBatch_Success(t *testing.T) {
	msg := outbox.NewOutboxMessage("topic", "event.v1", `{"key":"value"}`, "corr-1")
	store := &mockStore{messages: []outbox.OutboxMessage{msg}}
	publisher := &mockPublisher{}

	processor := outbox.NewOutboxProcessor(store, publisher, 10)
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 1, count)

	// Publish が呼ばれていることを確認
	require.Len(t, publisher.published, 1)
	assert.Equal(t, msg.Topic, publisher.published[0].Topic)

	// ステータスが Processing → Delivered に更新されていることを確認
	require.Len(t, store.statusUpdates, 2)
	assert.Equal(t, outbox.OutboxStatusProcessing, store.statusUpdates[0].status)
	assert.Equal(t, outbox.OutboxStatusDelivered, store.statusUpdates[1].status)
}

func TestProcessBatch_PublishFails(t *testing.T) {
	msg := outbox.NewOutboxMessage("topic", "event.v1", `{}`, "corr-1")
	store := &mockStore{messages: []outbox.OutboxMessage{msg}}
	publisher := &mockPublisher{err: errors.New("kafka unavailable")}

	processor := outbox.NewOutboxProcessor(store, publisher, 10)
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err) // バッチ全体はエラーにならない
	assert.Equal(t, 0, count)

	// Processing → Failed に更新されていることを確認
	require.Len(t, store.statusUpdates, 2)
	assert.Equal(t, outbox.OutboxStatusProcessing, store.statusUpdates[0].status)
	assert.Equal(t, outbox.OutboxStatusFailed, store.statusUpdates[1].status)
	assert.Equal(t, 1, store.statusUpdates[1].retryCount)
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
	messages := make([]outbox.OutboxMessage, 5)
	for i := range messages {
		messages[i] = outbox.NewOutboxMessage("topic", "event.v1", `{}`, "corr-1")
	}
	store := &mockStore{messages: messages}
	publisher := &mockPublisher{}

	processor := outbox.NewOutboxProcessor(store, publisher, 3) // batch size = 3
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 3, count) // 3 件のみ処理
}

func TestOutboxStoreError(t *testing.T) {
	cause := errors.New("connection failed")
	err := &outbox.OutboxStoreError{Op: "SaveMessage", Err: cause}
	assert.Contains(t, err.Error(), "SaveMessage")
	assert.ErrorIs(t, err, cause)
}

func TestNewOutboxMessage_TimestampIsUTC(t *testing.T) {
	msg := outbox.NewOutboxMessage("topic", "event.v1", `{}`, "corr-1")
	assert.Equal(t, time.UTC, msg.CreatedAt.Location())
	assert.Equal(t, time.UTC, msg.UpdatedAt.Location())
	assert.Equal(t, time.UTC, msg.ScheduledAt.Location())
}

func TestOutboxStatus_Constants(t *testing.T) {
	assert.Equal(t, outbox.OutboxStatus("PENDING"), outbox.OutboxStatusPending)
	assert.Equal(t, outbox.OutboxStatus("PROCESSING"), outbox.OutboxStatusProcessing)
	assert.Equal(t, outbox.OutboxStatus("DELIVERED"), outbox.OutboxStatusDelivered)
	assert.Equal(t, outbox.OutboxStatus("FAILED"), outbox.OutboxStatusFailed)
}

func TestProcessBatch_MultipleMessages(t *testing.T) {
	msgs := []outbox.OutboxMessage{
		outbox.NewOutboxMessage("topic", "event.v1", `{"n":1}`, "corr-1"),
		outbox.NewOutboxMessage("topic", "event.v1", `{"n":2}`, "corr-2"),
		outbox.NewOutboxMessage("topic", "event.v1", `{"n":3}`, "corr-3"),
	}
	store := &mockStore{messages: msgs}
	publisher := &mockPublisher{}

	processor := outbox.NewOutboxProcessor(store, publisher, 10)
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 3, count)
	assert.Len(t, publisher.published, 3)
	// 各メッセージが Processing → Delivered の 2 回ずつ更新される
	assert.Len(t, store.statusUpdates, 6)
}

func TestProcessBatch_PartialSuccess(t *testing.T) {
	// 2 件のメッセージのうち 2 件目が失敗するケース
	msgs := []outbox.OutboxMessage{
		outbox.NewOutboxMessage("topic", "event.v1", `{"n":1}`, "corr-1"),
		outbox.NewOutboxMessage("topic", "event.v1", `{"n":2}`, "corr-2"),
	}
	store := &mockStore{messages: msgs}
	var published []outbox.OutboxMessage
	callCount := 0
	pub := &mockPublisherFunc{fn: func(ctx context.Context, msg outbox.OutboxMessage) error {
		callCount++
		if callCount == 2 {
			return errors.New("second message failed")
		}
		published = append(published, msg)
		return nil
	}}

	processor := outbox.NewOutboxProcessor(store, pub, 10)
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 1, count) // 1 件のみ成功
	assert.Len(t, published, 1)
}

type mockPublisherFunc struct {
	fn func(ctx context.Context, msg outbox.OutboxMessage) error
}

func (m *mockPublisherFunc) Publish(ctx context.Context, msg outbox.OutboxMessage) error {
	return m.fn(ctx, msg)
}

func TestOutboxProcessor_DefaultBatchSize(t *testing.T) {
	// batchSize = 0 の場合はデフォルト 100 が使われる
	msgs := make([]outbox.OutboxMessage, 5)
	for i := range msgs {
		msgs[i] = outbox.NewOutboxMessage("topic", "event.v1", `{}`, "corr")
	}
	store := &mockStore{messages: msgs}
	publisher := &mockPublisher{}

	processor := outbox.NewOutboxProcessor(store, publisher, 0) // 0 → default 100
	count, err := processor.ProcessBatch(context.Background())
	require.NoError(t, err)
	assert.Equal(t, 5, count) // 全件処理される（100 > 5）
}
