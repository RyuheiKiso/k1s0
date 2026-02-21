package outbox

import (
	"context"
	"log"
	"time"
)

// OutboxProcessor はアウトボックスメッセージを定期的に処理する。
type OutboxProcessor struct {
	store     OutboxStore
	publisher OutboxPublisher
	batchSize int
}

// NewOutboxProcessor は新しい OutboxProcessor を生成する。
func NewOutboxProcessor(store OutboxStore, publisher OutboxPublisher, batchSize int) *OutboxProcessor {
	if batchSize <= 0 {
		batchSize = 100
	}
	return &OutboxProcessor{
		store:     store,
		publisher: publisher,
		batchSize: batchSize,
	}
}

// ProcessBatch は保留中のメッセージを一括処理する。
// 各メッセージを Processing に更新してから Publish し、結果に応じて Delivered または Failed に更新する。
func (p *OutboxProcessor) ProcessBatch(ctx context.Context) (int, error) {
	messages, err := p.store.GetPendingMessages(ctx, p.batchSize)
	if err != nil {
		return 0, &OutboxStoreError{Op: "GetPendingMessages", Err: err}
	}

	processed := 0
	for _, msg := range messages {
		if err := p.processMessage(ctx, msg); err != nil {
			log.Printf("outbox: failed to process message %s: %v", msg.ID, err)
			continue
		}
		processed++
	}
	return processed, nil
}

// processMessage は単一メッセージを処理する。
func (p *OutboxProcessor) processMessage(ctx context.Context, msg OutboxMessage) error {
	// Processing に更新
	if err := p.store.UpdateStatus(ctx, msg.ID, OutboxStatusProcessing); err != nil {
		return &OutboxStoreError{Op: "UpdateStatus(Processing)", Err: err}
	}

	// Kafka に送信
	if err := p.publisher.Publish(ctx, msg); err != nil {
		// 失敗: リトライカウントをインクリメントして Failed に更新
		retryCount := msg.RetryCount + 1
		scheduledAt := NextScheduledAt(retryCount)
		if updateErr := p.store.UpdateStatusWithRetry(ctx, msg.ID, OutboxStatusFailed, retryCount, scheduledAt); updateErr != nil {
			log.Printf("outbox: failed to update status to Failed for %s: %v", msg.ID, updateErr)
		}
		return err
	}

	// 成功: Delivered に更新
	if err := p.store.UpdateStatus(ctx, msg.ID, OutboxStatusDelivered); err != nil {
		return &OutboxStoreError{Op: "UpdateStatus(Delivered)", Err: err}
	}
	return nil
}

// Run は指定間隔でバッチ処理を継続実行する。ctx がキャンセルされたら停止する。
func (p *OutboxProcessor) Run(ctx context.Context, interval time.Duration) {
	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if _, err := p.ProcessBatch(ctx); err != nil {
				log.Printf("outbox: batch processing error: %v", err)
			}
		}
	}
}
