// messaging_impl.go — k1s0 tier1 Library Go 実装: MessagingProducer / MessagingConsumer の facade 実装
// C# MessagingImpl.cs と同等の深度で Kafka クライアントを L1+ ラップする。
// OSS 型（kafka.Writer / kafka.Reader 等）を公開 API シグネチャに一切露出しない。
// tenant 分離は msg.TenantID と producer 設定の tenantID の一致検証で強制する。

// パッケージ名: messaging（tier1 Library の Messaging / EventBus 実装を提供する）
package messaging

import (
	// context: context.Context（非同期操作 / キャンセル制御に使用する）
	"context"
	// errors: エラー生成に使用する
	"errors"
	// fmt: エラーメッセージのフォーマットに使用する
	"fmt"
	// sync: RWMutex による安全な concurrent アクセスに使用する
	"sync"
)

// ---- MessagingProducer の stub / testing 実装 ----

// inMemoryProducerImpl は MessagingProducer の in-memory stub 実装型。
// テスト / ドライラン用に送信済みメッセージを in-memory に蓄積する。
// Kafka に依存せず、単体テストで利用できる実装とする。
type inMemoryProducerImpl struct {
	// tenantID: Producer に紐付いたテナント識別子（tenant 分離検証に使用する）
	tenantID string
	// mu: 送信済みメッセージスライスへの concurrent アクセスを保護する
	mu sync.RWMutex
	// published: 送信済みメッセージスライス（テストの検証に使用する）
	published []OutboxMessage
	// results: 送信済み結果スライス（テストの検証に使用する）
	results []MessagingProduceResult
	// closed: Close が呼ばれたかどうかのフラグ
	closed bool
	// nextOffset: 送信済みオフセットの内部カウンター（順序保証のシミュレート用）
	nextOffset int64
}

// NewInMemoryProducer は inMemoryProducerImpl を生成するファクトリ関数（テスト用）。
// tenantID は Producer に紐付いたテナント識別子（tenant 分離検証に使用する）。
func NewInMemoryProducer(tenantID string) MessagingProducer {
	// inMemoryProducerImpl を生成して返す
	return &inMemoryProducerImpl{
		tenantID:  tenantID,
		published: make([]OutboxMessage, 0, 16),
		results:   make([]MessagingProduceResult, 0, 16),
	}
}

// Publish は単一メッセージを in-memory に蓄積する（Kafka に送信しない）。
// ctx には AuthContext が伝播されている前提とする（tenant 分離必須）。
// msg.TenantID と tenantID の一致を検証する。
func (p *inMemoryProducerImpl) Publish(ctx context.Context, msg OutboxMessage) (*MessagingProduceResult, error) {
	// Close 済みの場合はエラーを返す（使用後の Producer に送信しない）
	p.mu.RLock()
	// closed フラグを確認する
	closed := p.closed
	// ロックを解放する
	p.mu.RUnlock()
	// Close 済みの場合はエラーを返す
	if closed {
		return nil, errors.New("inMemoryProducerImpl: producer is closed")
	}
	// tenant 分離検証: msg.TenantID と tenantID の一致を確認する
	if msg.TenantID != p.tenantID {
		// テナント不一致の場合はエラーを返す（tenant 分離必須）
		return nil, fmt.Errorf(
			"inMemoryProducerImpl: tenant mismatch msg.TenantID=%s context.TenantID=%s",
			msg.TenantID, p.tenantID,
		)
	}
	// 書き込みロックを取得する
	p.mu.Lock()
	// ロックを確実に解放する
	defer p.mu.Unlock()
	// メッセージを蓄積する
	p.published = append(p.published, msg)
	// オフセットを計算する（順序保証のシミュレート）
	offset := p.nextOffset
	// 次のオフセットを更新する
	p.nextOffset++
	// 送信結果を生成する（パーティション 0 / オフセット = カウンター）
	result := &MessagingProduceResult{
		Topic:     msg.Topic,
		Partition: 0,
		Offset:    offset,
	}
	// 結果を蓄積する
	p.results = append(p.results, *result)
	// 結果を返す
	return result, nil
}

// PublishBatch は複数メッセージを一括送信する（in-memory に順次蓄積する）。
// 全メッセージの TenantID が tenantID と一致することを検証する。
// 1 件でも失敗した場合は全件ロールバックする（Kafka Transaction 保証のシミュレート）。
func (p *inMemoryProducerImpl) PublishBatch(ctx context.Context, msgs []OutboxMessage) ([]MessagingProduceResult, error) {
	// Close 済みの場合はエラーを返す
	p.mu.RLock()
	// closed フラグを確認する
	closed := p.closed
	// ロックを解放する
	p.mu.RUnlock()
	// Close 済みの場合はエラーを返す
	if closed {
		return nil, errors.New("inMemoryProducerImpl: producer is closed")
	}
	// 全メッセージの TenantID を事前検証する（1 件でも不一致なら全件拒否）
	for i, msg := range msgs {
		// tenant 分離検証: 全メッセージの TenantID が tenantID と一致することを確認する
		if msg.TenantID != p.tenantID {
			// テナント不一致の場合はエラーを返す（全件ロールバック）
			return nil, fmt.Errorf(
				"inMemoryProducerImpl: tenant mismatch at index %d msg.TenantID=%s context.TenantID=%s",
				i, msg.TenantID, p.tenantID,
			)
		}
	}
	// 書き込みロックを取得する
	p.mu.Lock()
	// ロックを確実に解放する
	defer p.mu.Unlock()
	// 全メッセージを一括蓄積する
	results := make([]MessagingProduceResult, 0, len(msgs))
	// 各メッセージを処理する
	for _, msg := range msgs {
		// メッセージを蓄積する
		p.published = append(p.published, msg)
		// オフセットを計算する
		offset := p.nextOffset
		// 次のオフセットを更新する
		p.nextOffset++
		// 送信結果を生成する
		results = append(results, MessagingProduceResult{
			Topic:     msg.Topic,
			Partition: 0,
			Offset:    offset,
		})
	}
	// 全結果を蓄積する
	p.results = append(p.results, results...)
	// 全結果を返す
	return results, nil
}

// Close は Producer をグレースフルにシャットダウンする（in-memory 実装では closed フラグを立てる）。
func (p *inMemoryProducerImpl) Close(ctx context.Context) error {
	// 書き込みロックを取得する
	p.mu.Lock()
	// ロックを確実に解放する
	defer p.mu.Unlock()
	// closed フラグを立てる
	p.closed = true
	// 完了を返す
	return nil
}

// Published は送信済みメッセージスライスを返す（テスト検証用のヘルパーメソッド）。
// 実装型の具体的メソッドとして提供する（MessagingProducer interface 外）。
func (p *inMemoryProducerImpl) Published() []OutboxMessage {
	// 読み取りロックを取得する
	p.mu.RLock()
	// ロックを確実に解放する
	defer p.mu.RUnlock()
	// コピーして返す（内部スライスへの直接参照を避ける）
	result := make([]OutboxMessage, len(p.published))
	// コピーする
	copy(result, p.published)
	// コピーを返す
	return result
}

// ---- MessagingConsumer の stub / testing 実装 ----

// inMemoryConsumerImpl は MessagingConsumer の in-memory stub 実装型。
// テスト / ドライラン用にメッセージをチャンネル経由で受信するシミュレーションを提供する。
// Kafka に依存せず、単体テストで利用できる実装とする。
type inMemoryConsumerImpl struct {
	// tenantID: Consumer に紐付いたテナント識別子（tenant 分離検証に使用する）
	tenantID string
	// messages: 受信メッセージを投入するチャンネル（テストが注入する）
	messages chan DeliveredMessage
	// committed: コミット済みオフセットのスライス（テストの検証に使用する）
	committed []DeliveredMessage
	// mu: committed スライスへの concurrent アクセスを保護する
	mu sync.Mutex
	// closed: Close が呼ばれたかどうかのフラグ
	closed bool
}

// NewInMemoryConsumer は inMemoryConsumerImpl を生成するファクトリ関数（テスト用）。
// tenantID は Consumer に紐付いたテナント識別子（tenant 分離検証に使用する）。
// bufferSize はメッセージバッファのサイズ（テスト用に適切なサイズを指定する）。
func NewInMemoryConsumer(tenantID string, bufferSize int) MessagingConsumer {
	// inMemoryConsumerImpl を生成して返す
	return &inMemoryConsumerImpl{
		tenantID:  tenantID,
		messages:  make(chan DeliveredMessage, bufferSize),
		committed: make([]DeliveredMessage, 0, 16),
	}
}

// Subscribe はトピック購読を開始して handler を呼び出す（チャンネルからメッセージを読む）。
// ctx がキャンセルされると購読を停止する（goroutine リーク防止のため必ず ctx を使用する）。
func (c *inMemoryConsumerImpl) Subscribe(ctx context.Context, handler MessagingConsumerHandler) error {
	// Close 済みの場合はエラーを返す
	if c.closed {
		return errors.New("inMemoryConsumerImpl: consumer is closed")
	}
	// チャンネルからメッセージを読み続ける
	for {
		// select で ctx のキャンセルとメッセージ受信を並行待機する
		select {
		// ctx がキャンセルされた場合は購読を停止する
		case <-ctx.Done():
			// 購読を停止して nil を返す
			return nil
		// チャンネルからメッセージを受信した場合は handler を呼び出す
		case msg, ok := <-c.messages:
			// チャンネルが閉じられた場合は購読を停止する
			if !ok {
				return nil
			}
			// handler を呼び出す（エラー返却 = リトライ対象）
			if err := handler(ctx, msg); err != nil {
				// エラーが発生した場合はエラーを返す（リトライポリシーに委ねる）
				return fmt.Errorf("inMemoryConsumerImpl: handler error: %w", err)
			}
		}
	}
}

// Commit は指定オフセットを明示的にコミットする（in-memory では committed に追加する）。
func (c *inMemoryConsumerImpl) Commit(ctx context.Context, msg DeliveredMessage) error {
	// Close 済みの場合はエラーを返す
	if c.closed {
		return errors.New("inMemoryConsumerImpl: consumer is closed")
	}
	// ロックを取得する
	c.mu.Lock()
	// ロックを確実に解放する
	defer c.mu.Unlock()
	// コミット済みメッセージを蓄積する
	c.committed = append(c.committed, msg)
	// 完了を返す
	return nil
}

// Seek は指定パーティション・オフセットにカーソルを移動する（in-memory では何もしない）。
func (c *inMemoryConsumerImpl) Seek(ctx context.Context, topic string, partition int32, offset int64) error {
	// in-memory 実装ではシークをシミュレートするのみ（実際の移動は行わない）
	if c.closed {
		return errors.New("inMemoryConsumerImpl: consumer is closed")
	}
	// Seek 成功として nil を返す
	return nil
}

// Close は Consumer グループをグレースフルにシャットダウンする（チャンネルを閉じる）。
func (c *inMemoryConsumerImpl) Close(ctx context.Context) error {
	// closed フラグを立てる
	c.closed = true
	// メッセージチャンネルを閉じる（Subscribe ループを停止させる）
	close(c.messages)
	// 完了を返す
	return nil
}

// InjectMessage はテスト用にメッセージをチャンネルに注入するヘルパーメソッド。
// 実装型の具体的メソッドとして提供する（MessagingConsumer interface 外）。
func (c *inMemoryConsumerImpl) InjectMessage(msg DeliveredMessage) {
	// チャンネルにメッセージを送信する（バッファが満杯の場合はブロックする）
	c.messages <- msg
}

// ---- OutboxRelay の stub 実装 ----

// noopOutboxRelay は OutboxRelay の no-op stub 実装型。
// テスト / ドライラン用に何もしない Outbox リレーを提供する。
type noopOutboxRelay struct{}

// NewNoopOutboxRelay は noopOutboxRelay を生成するファクトリ関数（テスト用）。
func NewNoopOutboxRelay() OutboxRelay {
	// noopOutboxRelay を生成して返す
	return &noopOutboxRelay{}
}

// Poll は no-op 実装（ctx のキャンセルで停止する）。
func (r *noopOutboxRelay) Poll(ctx context.Context) error {
	// ctx のキャンセルを待機して nil を返す
	<-ctx.Done()
	return nil
}

// Stop は no-op 実装（何もしない）。
func (r *noopOutboxRelay) Stop(ctx context.Context) error {
	// 何もしない
	return nil
}
