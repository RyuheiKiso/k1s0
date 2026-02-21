package kafka

import "context"

// KafkaHealthStatus は Kafka ヘルスチェックの結果。
type KafkaHealthStatus struct {
	// Healthy はブローカーに接続できるかどうか。
	Healthy bool
	// Message はヘルスチェックのメッセージ。
	Message string
	// BrokerCount は接続できたブローカー数。
	BrokerCount int
}

// KafkaHealthChecker は Kafka の疎通確認インターフェース。
type KafkaHealthChecker interface {
	HealthCheck(ctx context.Context) (*KafkaHealthStatus, error)
}

// NoOpKafkaHealthChecker はテスト用の no-op 実装。
type NoOpKafkaHealthChecker struct {
	Status *KafkaHealthStatus
	Err    error
}

// HealthCheck は設定された Status と Err を返す。
func (n *NoOpKafkaHealthChecker) HealthCheck(ctx context.Context) (*KafkaHealthStatus, error) {
	return n.Status, n.Err
}
