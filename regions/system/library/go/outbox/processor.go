package outbox

import (
	"context"
	"log"
	"time"
)

// OutboxProcessor はアウトボックスメッセージの定期処理を担う。
// FetchPending → Publish → Update のサイクルを実行する。
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

// ProcessBatch は1回分のアウトボックス処理を実行する。
// 処理したメッセージ数を返す。
func (p *OutboxProcessor) ProcessBatch(ctx context.Context) (int, error) {
	messages, err := p.store.FetchPending(ctx, p.batchSize)
	if err != nil {
		return 0, NewStoreError("FetchPending", err)
	}

	processed := 0
	for i := range messages {
		msg := &messages[i]
		msg.MarkProcessing()
		if err := p.store.Update(ctx, msg); err != nil {
			log.Printf("outbox: failed to update message %s to Processing: %v", msg.ID, err)
			continue
		}

		if err := p.publisher.Publish(ctx, msg); err != nil {
			msg.MarkFailed(err.Error())
			if updateErr := p.store.Update(ctx, msg); updateErr != nil {
				log.Printf("outbox: failed to update message %s to Failed: %v", msg.ID, updateErr)
			}
			continue
		}

		msg.MarkDelivered()
		if err := p.store.Update(ctx, msg); err != nil {
			log.Printf("outbox: failed to update message %s to Delivered: %v", msg.ID, err)
			continue
		}
		processed++
	}

	return processed, nil
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
