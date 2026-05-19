// messaging.go — k1s0 tier1 Library Go 実装: Messaging / EventBus の L1+ interface
// 11_メッセージング適合仕様.md §MessagingProducer / §MessagingConsumer（Kafka L1+ 深耕）に準拠する。
// Kafka の full API を Library 独自語彙で表現しつつ、AuthContext 伝播を強制する。
// OSS 型（sarama.Producer / confluent-kafka-go 等）を公開シグネチャに一切含まない。

// パッケージ名: messaging（tier1 Library の Messaging / EventBus API を提供する）
package messaging

import (
	// context: context.Context（AuthContext 伝播 + 非同期操作に使用する）
	"context"
)

// OutboxMessage は Messaging L1+ (Kafka) の Outbox メッセージを宣言する型。
// 11_メッセージング適合仕様.md §Outbox Pattern の必須フィールドに準拠する。
// tenant 分離を保証するために TenantID を必須フィールドとして持つ。
type OutboxMessage struct {
	// TenantID: メッセージの発行元テナント識別子（必須: partition routing に使用する）
	TenantID string
	// Topic: Kafka トピック名（tenant prefix を含む: "{TenantID}.{事業ドメイン}" 形式を推奨する）
	Topic string
	// Key: Kafka メッセージキー（同一エンティティのメッセージ順序保証に使用する）
	Key string
	// Payload: メッセージボディ（protobuf / JSON バイト列）
	Payload []byte
	// Headers: Kafka メッセージヘッダー（trace_id / auth_class 等の横断属性）
	Headers map[string][]byte
	// PartitionKey: Kafka パーティションキー（空文字列 = Key を使用する）
	PartitionKey string
	// SchemaID: Schema Registry で登録されたスキーマ ID（0 = スキーマ検証なし）
	SchemaID int64
}

// DeliveredMessage は Messaging L1+ (Kafka) で受信したメッセージを宣言する型。
// Consumer が Kafka から受け取ったメッセージを Library 独自語彙で表現する。
type DeliveredMessage struct {
	// TenantID: メッセージの発行元テナント識別子
	TenantID string
	// Topic: 受信したトピック名
	Topic string
	// Key: メッセージキー
	Key string
	// Payload: メッセージボディ
	Payload []byte
	// Headers: メッセージヘッダー
	Headers map[string][]byte
	// Partition: 受信したパーティション番号
	Partition int32
	// Offset: 受信したオフセット
	Offset int64
	// SchemaID: Schema Registry のスキーマ ID（0 = 未設定）
	SchemaID int64
}

// MessagingProduceResult は Publish の結果を宣言する型。
// 送信成功したメッセージのパーティションとオフセットを返す（監査ログに使用する）。
type MessagingProduceResult struct {
	// Topic: 送信先トピック名
	Topic string
	// Partition: 送信されたパーティション番号
	Partition int32
	// Offset: 送信されたオフセット
	Offset int64
}

// MessagingProducer は Messaging L1+ (Kafka) の Producer interface を宣言する。
// Kafka の full API を Library 独自語彙で表現する。
// ctx に AuthContext が含まれることを強制する（tenant 分離必須）。
// OSS 型（sarama.SyncProducer 等）を引数・戻り値に一切含まない。
type MessagingProducer interface {
	// Publish は単一メッセージを Kafka トピックに送信する。
	// ctx には AuthContext が伝播されている前提とする（tenant 分離必須）。
	// msg.TenantID と AuthContext.TenantID の一致を実装側で検証する。
	Publish(ctx context.Context, msg OutboxMessage) (*MessagingProduceResult, error)

	// PublishBatch は複数メッセージを一括送信する（Transaction Producer を使用する）。
	// ctx には AuthContext が伝播されている前提とする（tenant 分離必須）。
	// 全メッセージの TenantID が AuthContext.TenantID と一致することを検証する。
	// 1 件でも失敗した場合は全件ロールバックする（Kafka Transaction 保証）。
	PublishBatch(ctx context.Context, msgs []OutboxMessage) ([]MessagingProduceResult, error)

	// Close は Producer をグレースフルにシャットダウンする（未送信メッセージをフラッシュする）。
	Close(ctx context.Context) error
}

// ConsumerGroupOptions は ConsumerGroup の設定オプションを宣言する型。
type ConsumerGroupOptions struct {
	// GroupID: Kafka Consumer Group ID（tenant prefix を含む形式を推奨する）
	GroupID string
	// Topics: 購読するトピックスライス
	Topics []string
	// AutoCommit: オフセットを自動コミットするかどうか（false = 手動コミット推奨）
	AutoCommit bool
	// MaxPollRecords: 一度のポーリングで取得する最大レコード数
	MaxPollRecords int
	// IsolationLevel: Kafka Isolation Level（"read_committed" / "read_uncommitted"）
	IsolationLevel string
}

// MessagingConsumerHandler は Consumer が受信したメッセージを処理する関数型。
// 関数が error を返した場合は実装がリトライ / DLQ 送信を行う。
type MessagingConsumerHandler func(ctx context.Context, msg DeliveredMessage) error

// MessagingConsumer は Messaging L1+ (Kafka) の Consumer interface を宣言する。
// Kafka Consumer Group の full API を Library 独自語彙で表現する。
// OSS 型（sarama.ConsumerGroup 等）を引数・戻り値に一切含まない。
type MessagingConsumer interface {
	// Subscribe はトピック購読を開始して handler を呼び出す（ブロッキング）。
	// ctx がキャンセルされると購読を停止する（goroutine リーク防止のため必ず ctx を使用する）。
	// handler にはメッセージを処理するコールバックを渡す（エラー返却 = リトライ対象）。
	Subscribe(ctx context.Context, handler MessagingConsumerHandler) error

	// Commit は指定オフセットを明示的にコミットする（AutoCommit=false 時に使用する）。
	// msg は処理が完了したメッセージ（Partition / Offset を使用してコミットする）。
	Commit(ctx context.Context, msg DeliveredMessage) error

	// Seek は指定パーティション・オフセットにカーソルを移動する（リプレイ用途）。
	// partition はシーク対象のパーティション番号、offset はシーク先オフセット。
	Seek(ctx context.Context, topic string, partition int32, offset int64) error

	// Close は Consumer グループをグレースフルにシャットダウンする。
	Close(ctx context.Context) error
}

// DeadLetterQueueOptions は DLQ（Dead Letter Queue）の設定を宣言する型。
type DeadLetterQueueOptions struct {
	// Topic: DLQ トピック名（"dlq.{original_topic}" 形式を推奨する）
	Topic string
	// MaxRetries: DLQ 送信前のリトライ回数
	MaxRetries int
}

// OutboxRelay は Outbox Pattern の中継 interface を宣言する。
// DB Outbox テーブルから Kafka に at-least-once でメッセージを転送する。
type OutboxRelay interface {
	// Poll は Outbox テーブルから未送信メッセージを取得して Kafka に転送する。
	// ctx のキャンセルでポーリングを停止する（永続バックグラウンド処理を想定する）。
	Poll(ctx context.Context) error

	// Stop はポーリングを停止してグレースフルシャットダウンする。
	Stop(ctx context.Context) error
}
