package outbox

import (
	"context"
	"log/slog"
	"time"
)

// OutboxProcessor はアウトボックスメッセージの定期処理を担う。
// FetchPending → Publish → Update のサイクルを実行する。
type OutboxProcessor struct {
	store     OutboxStore
	publisher OutboxPublisher
	batchSize int
	logger    *slog.Logger
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
		logger:    slog.Default(),
	}
}

// NewOutboxProcessorWithLogger はカスタムロガー付きの OutboxProcessor を生成する。
func NewOutboxProcessorWithLogger(store OutboxStore, publisher OutboxPublisher, batchSize int, logger *slog.Logger) *OutboxProcessor {
	p := NewOutboxProcessor(store, publisher, batchSize)
	if logger != nil {
		p.logger = logger
	}
	return p
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
			// Processing ステータスへの更新失敗をログに記録し、次のメッセージへ進む
			p.logger.Error("アウトボックスメッセージの Processing 更新に失敗",
				slog.String("message_id", msg.ID),
				slog.String("error", err.Error()),
			)
			continue
		}

		if err := p.publisher.Publish(ctx, msg); err != nil {
			msg.MarkFailed(err.Error())
			if updateErr := p.store.Update(ctx, msg); updateErr != nil {
				// Failed ステータスへの更新失敗をログに記録する
				p.logger.Error("アウトボックスメッセージの Failed 更新に失敗",
					slog.String("message_id", msg.ID),
					slog.String("error", updateErr.Error()),
				)
			}
			continue
		}

		msg.MarkDelivered()
		if err := p.store.Update(ctx, msg); err != nil {
			// Delivered ステータスへの更新失敗をログに記録する
			p.logger.Error("アウトボックスメッセージの Delivered 更新に失敗",
				slog.String("message_id", msg.ID),
				slog.String("error", err.Error()),
			)
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
				// バッチ処理全体のエラーをログに記録する
				p.logger.Error("アウトボックスバッチ処理でエラーが発生",
					slog.String("error", err.Error()),
				)
			}
		}
	}
}
