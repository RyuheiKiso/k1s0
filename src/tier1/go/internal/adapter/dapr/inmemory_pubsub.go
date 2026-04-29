// 本ファイルは in-memory backend 用 PubSubAdapter 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/03_PubSub_API.md
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//
// 役割:
//   inmemory.go の `inMemoryDapr` は SDK の `*daprclient.Subscription` を構築できないため、
//   pubsub の Subscribe では ErrNotWired を返している。dev / CI でも Publish → Subscribe の
//   round-trip を検証可能にするため、本ファイルでは PubSubAdapter interface を直接実装する
//   in-memory bus を提供する。
//
// 配信モデル（dev / CI 限定）:
//   - 同一プロセス内 publish → subscribe round-trip
//   - tenant_id × topic × consumer_group で channel を分離
//   - 同 (tenant, topic, group) の subscriber 群は 1 channel を共有（消費は競合）
//   - subscribe 開始前の publish は破棄（永続化なし、Kafka とは異なる）
//   - bound channel 容量を超える publish は backpressure（caller block）
//
// production との関係:
//   実 Dapr Pub/Sub（Kafka）が結線されている場合は本 in-memory bus を経由しない。
//   `NewPubSubAdapter` が Client.pubsubBus を見て分岐する。

package dapr

import (
	// channel ベースの subscriber キューに使う。
	"context"
	// subscriber 切断時のエラー型に使う。
	"errors"
	// 並行制御。
	"sync"
)

// pubsubBufferSize は in-memory bus の subscriber 1 件あたりの保留キュー容量。
// 容量を超えると Publish が block して publisher を遅延させる。
const pubsubBufferSize = 64

// pubsubBus は in-memory pub/sub の broker 本体。
// (tenant_id, topic, consumer_group) で 1 channel を構築し、複数 subscriber が同 channel を
// 競合消費する Kafka 風の挙動を最小限に再現する。
type pubsubBus struct {
	// 全状態を保護する RWMutex。
	mu sync.RWMutex
	// (tenant_id, topic, consumer_group) → 1 channel
	channels map[busKey]chan *SubscribedEvent
}

// busKey は in-memory bus の channel 識別子。
type busKey struct {
	tenantID      string
	topic         string
	consumerGroup string
}

// newPubSubBus は空の bus を生成する。
func newPubSubBus() *pubsubBus {
	// channels map を空で初期化する。
	return &pubsubBus{
		channels: map[busKey]chan *SubscribedEvent{},
	}
}

// channelFor は (tenant, topic, group) に対応する channel を取得 / 遅延生成する。
func (b *pubsubBus) channelFor(tenant, topic, group string) chan *SubscribedEvent {
	// 書込ロックを取る（map 変更可能性のため）。
	b.mu.Lock()
	defer b.mu.Unlock()
	// 既存 channel があれば返す。
	k := busKey{tenantID: tenant, topic: topic, consumerGroup: group}
	if ch, ok := b.channels[k]; ok {
		return ch
	}
	// 新 channel を bound 容量で生成する。
	ch := make(chan *SubscribedEvent, pubsubBufferSize)
	// map に登録する。
	b.channels[k] = ch
	// 返却する。
	return ch
}

// publish は (tenant, topic) に対応する **全 consumer group** の channel に event を fan-out する。
// consumer group が異なる subscriber には全員配信、同一 consumer group の subscriber は競合消費。
func (b *pubsubBus) publish(ctx context.Context, tenant, topic string, ev *SubscribedEvent) error {
	// 読出ロックで対象 channel 群を集める。
	b.mu.RLock()
	// 対象 channel 群を copy する（送信中は lock を握り続けない）。
	targets := make([]chan *SubscribedEvent, 0, 4)
	// 全エントリを走査して (tenant, topic) 一致するものを集める。
	for k, ch := range b.channels {
		if k.tenantID == tenant && k.topic == topic {
			// 一致したら slice に追加する。
			targets = append(targets, ch)
		}
	}
	// 解放する。
	b.mu.RUnlock()
	// 各 channel へ送信する（容量超過時は ctx 解約まで block）。
	for _, ch := range targets {
		// context 解約と送信のどちらかで unblock する。
		select {
		case ch <- ev:
			// 送信成功 → 次へ。
		case <-ctx.Done():
			// caller がキャンセルした → エラー返却。
			return ctx.Err()
		}
	}
	// subscribe する subscriber がいない場合は何もせず成功扱い（Kafka 似の retention は無し）。
	return nil
}

// inMemoryPubSubAdapter は PubSubAdapter interface を直接実装する in-memory bus 経由の adapter。
// Client.pubsubBus が non-nil の時のみ NewPubSubAdapter から返される。
type inMemoryPubSubAdapter struct {
	// bus への参照。
	bus *pubsubBus
}

// Publish は in-memory bus に event を fan-out する。
func (a *inMemoryPubSubAdapter) Publish(ctx context.Context, req PublishRequest) (PublishResponse, error) {
	// SubscribedEvent に詰め替える。Ack / Retry は no-op（in-memory には DLQ なし）。
	ev := &SubscribedEvent{
		Topic:       req.Topic,
		Data:        req.Data,
		ContentType: req.ContentType,
		Metadata:    req.Metadata,
		Offset:      0,
		Ack:         func() error { return nil },
		Retry:       func() error { return nil },
	}
	// bus に publish する。
	if err := a.bus.publish(ctx, req.TenantID, req.Topic, ev); err != nil {
		// publish 失敗を返却する。
		return PublishResponse{}, err
	}
	// 成功時 Offset=0 で返す（in-memory は exposing しない）。
	return PublishResponse{Offset: 0}, nil
}

// Subscribe は in-memory bus の channel を 1 件取得し、PubSubSubscription として包む。
func (a *inMemoryPubSubAdapter) Subscribe(_ context.Context, req SubscribeAdapterRequest) (PubSubSubscription, error) {
	// (tenant, topic, group) で channel を確保する（無ければ生成）。
	ch := a.bus.channelFor(req.TenantID, req.Topic, req.ConsumerGroup)
	// subscription を返却する。
	return &inMemoryPubSubSubscription{ch: ch, topic: req.Topic}, nil
}

// inMemoryPubSubSubscription は PubSubSubscription interface 実装。
type inMemoryPubSubSubscription struct {
	// 受信 channel。
	ch chan *SubscribedEvent
	// 論理トピック名（Receive 応答に詰める）。
	topic string
	// Close 後の二重 close 防止フラグ。
	closed bool
	// Close と Receive の同期制御。
	mu sync.Mutex
}

// errSubscriptionClosed は Receive / Close 後の Receive で返される sentinel。
var errSubscriptionClosed = errors.New("tier1: in-memory pubsub subscription closed")

// Receive は次の event を待ち受ける。ctx キャンセルでエラー返却。
func (s *inMemoryPubSubSubscription) Receive(ctx context.Context) (*SubscribedEvent, error) {
	// closed 判定（mutex 保護）。
	s.mu.Lock()
	// closed 後は即時 sentinel エラー。
	if s.closed {
		// unlock してから return する。
		s.mu.Unlock()
		return nil, errSubscriptionClosed
	}
	// unlock してから select に入る（recv は別 goroutine で進行）。
	s.mu.Unlock()
	// channel 受信 / context 解約のどちらかで戻る。
	select {
	case ev, ok := <-s.ch:
		// channel 閉鎖は subscription 終了。
		if !ok {
			// sentinel エラーを返す。
			return nil, errSubscriptionClosed
		}
		// イベントの Topic を Receive 開始時の論理 topic に揃える。
		if ev != nil && ev.Topic == "" {
			// 論理トピック名で補完する。
			ev.Topic = s.topic
		}
		// 受信した event を返却する。
		return ev, nil
	case <-ctx.Done():
		// caller がキャンセル → ctx エラーをそのまま返却する。
		return nil, ctx.Err()
	}
}

// Close は subscription を解放する。bus の channel は他 subscriber と共有のため close しない。
func (s *inMemoryPubSubSubscription) Close() error {
	// 二重 Close を防ぐ。
	s.mu.Lock()
	// 既に closed なら no-op。
	if s.closed {
		// unlock して終わる。
		s.mu.Unlock()
		return nil
	}
	// flag を立てる。
	s.closed = true
	// unlock する。
	s.mu.Unlock()
	// 共有 channel は close しない（他 subscriber を破壊するため）。Receive はフラグで遮断する。
	return nil
}
