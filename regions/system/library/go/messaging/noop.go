package messaging

import "context"

// NoOpEventProducer はテスト用の no-op EventProducer 実装。
// 送信されたイベントを記録するが、実際の Kafka への送信は行わない。
type NoOpEventProducer struct {
	// Published は送信されたイベントのリスト（テスト検証用）。
	Published []EventEnvelope
	// Err は Publish 時に返すエラー（nil の場合はエラーなし）。
	Err error
	// closed はクローズ済みかどうか。
	closed bool
}

// Publish はイベントを記録する。
func (n *NoOpEventProducer) Publish(ctx context.Context, event EventEnvelope) error {
	if n.Err != nil {
		return n.Err
	}
	n.Published = append(n.Published, event)
	return nil
}

// PublishBatch は複数イベントを順次 Publish する。
func (n *NoOpEventProducer) PublishBatch(ctx context.Context, events []EventEnvelope) error {
	for _, event := range events {
		if err := n.Publish(ctx, event); err != nil {
			return err
		}
	}
	return nil
}

// Close はプロデューサーをクローズ済みにする。
func (n *NoOpEventProducer) Close() error {
	n.closed = true
	return nil
}

// IsClosed はプロデューサーがクローズ済みかどうかを返す。
func (n *NoOpEventProducer) IsClosed() bool {
	return n.closed
}

// PublishedCount は送信されたイベント数を返す。
func (n *NoOpEventProducer) PublishedCount() int {
	return len(n.Published)
}
