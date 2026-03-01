package eventbus

import (
	"context"
	"fmt"
	"sync"
	"sync/atomic"
	"time"
)

// --- DDD ドメインイベント ---

// DomainEvent はドメインイベントのインターフェース。
type DomainEvent interface {
	EventType() string
	AggregateID() string
	OccurredAt() time.Time
}

// --- EventBusError ---

// ErrorKind はイベントバスエラーの種類。
type ErrorKind int

const (
	// PublishFailed はイベント発行に失敗した場合のエラー。
	PublishFailed ErrorKind = iota
	// HandlerFailed はハンドラー処理に失敗した場合のエラー。
	HandlerFailed
	// ChannelClosed はチャネルが閉じられた場合のエラー。
	ChannelClosed
)

// EventBusError はイベントバスのエラー型。
type EventBusError struct {
	Kind    ErrorKind
	Message string
	Err     error
}

func (e *EventBusError) Error() string {
	switch e.Kind {
	case PublishFailed:
		return fmt.Sprintf("publish failed: %s", e.Message)
	case HandlerFailed:
		return fmt.Sprintf("handler failed: %s", e.Message)
	case ChannelClosed:
		return "channel closed"
	default:
		return e.Message
	}
}

func (e *EventBusError) Unwrap() error {
	return e.Err
}

// --- EventBusConfig ---

// EventBusConfig はイベントバスの設定。
type EventBusConfig struct {
	BufferSize     int
	HandlerTimeout time.Duration
}

// DefaultEventBusConfig はデフォルトの設定を返す。
func DefaultEventBusConfig() EventBusConfig {
	return EventBusConfig{
		BufferSize:     1024,
		HandlerTimeout: 30 * time.Second,
	}
}

// --- DDD EventHandler ---

// EventHandler はドメインイベントを処理するジェネリックインターフェース。
type EventHandler[T DomainEvent] interface {
	Handle(ctx context.Context, event T) error
}

// EventHandlerFunc はハンドラー関数をEventHandlerインターフェースに変換するアダプター。
type EventHandlerFunc[T DomainEvent] func(ctx context.Context, event T) error

func (f EventHandlerFunc[T]) Handle(ctx context.Context, event T) error {
	return f(ctx, event)
}

// --- EventSubscription ---

// EventSubscription はイベント購読を表す。Unsubscribe() で購読を解除できる。
type EventSubscription struct {
	id        uint64
	eventType string
	bus       *EventBus
}

// Unsubscribe はこの購読を解除する。
func (s *EventSubscription) Unsubscribe() {
	s.bus.removeSubscription(s.eventType, s.id)
}

// --- EventBus (DDD パターン) ---

type handlerEntry struct {
	id      uint64
	handler func(ctx context.Context, event any) error
}

// EventBus は DDD パターンに対応したイベントバス。
type EventBus struct {
	config   EventBusConfig
	mu       sync.RWMutex
	handlers map[string][]handlerEntry
	nextID   atomic.Uint64
}

// NewEventBus は設定を指定して新しい EventBus を生成する。
func NewEventBus(config EventBusConfig) *EventBus {
	return &EventBus{
		config:   config,
		handlers: make(map[string][]handlerEntry),
	}
}

func (b *EventBus) removeSubscription(eventType string, id uint64) {
	b.mu.Lock()
	defer b.mu.Unlock()
	entries := b.handlers[eventType]
	for i, e := range entries {
		if e.id == id {
			b.handlers[eventType] = append(entries[:i], entries[i+1:]...)
			break
		}
	}
	if len(b.handlers[eventType]) == 0 {
		delete(b.handlers, eventType)
	}
}

// Subscribe はジェネリックなドメインイベントハンドラーを購読する。
func Subscribe[T DomainEvent](bus *EventBus, handler EventHandler[T]) *EventSubscription {
	// T のゼロ値から event type を取得するために、最初のイベント到着時に判定する
	// ただしイベントタイプはイベント自体が持つため、購読時にはワイルドカード登録し、
	// publish 側でマッチングする方式を採用
	//
	// より実用的なアプローチ: 購読時にイベントタイプを指定する必要があるため
	// SubscribeWithType を使う設計とする
	//
	// 代替: イベントをany経由で受け取り型アサーションで処理する
	id := bus.nextID.Add(1)

	wrappedHandler := func(ctx context.Context, event any) error {
		typedEvent, ok := event.(T)
		if !ok {
			return nil // 型が合わないイベントはスキップ
		}
		return handler.Handle(ctx, typedEvent)
	}

	// 購読登録 - ワイルドカード "*" に登録し、publish 時に全ハンドラーに配信
	bus.mu.Lock()
	bus.handlers["*"] = append(bus.handlers["*"], handlerEntry{id: id, handler: wrappedHandler})
	bus.mu.Unlock()

	return &EventSubscription{id: id, eventType: "*", bus: bus}
}

// SubscribeType は指定したイベントタイプのハンドラーを購読する。
func SubscribeType[T DomainEvent](bus *EventBus, eventType string, handler EventHandler[T]) *EventSubscription {
	id := bus.nextID.Add(1)

	wrappedHandler := func(ctx context.Context, event any) error {
		typedEvent, ok := event.(T)
		if !ok {
			return nil
		}
		return handler.Handle(ctx, typedEvent)
	}

	bus.mu.Lock()
	bus.handlers[eventType] = append(bus.handlers[eventType], handlerEntry{id: id, handler: wrappedHandler})
	bus.mu.Unlock()

	return &EventSubscription{id: id, eventType: eventType, bus: bus}
}

// Publish はドメインイベントを発行する。
func Publish[T DomainEvent](ctx context.Context, bus *EventBus, event T) error {
	bus.mu.RLock()
	eventType := event.EventType()
	// イベントタイプに一致するハンドラーとワイルドカードハンドラーを取得
	var allHandlers []handlerEntry
	if handlers, ok := bus.handlers[eventType]; ok {
		allHandlers = append(allHandlers, handlers...)
	}
	if handlers, ok := bus.handlers["*"]; ok {
		allHandlers = append(allHandlers, handlers...)
	}
	bus.mu.RUnlock()

	for _, h := range allHandlers {
		if bus.config.HandlerTimeout > 0 {
			timeoutCtx, cancel := context.WithTimeout(ctx, bus.config.HandlerTimeout)
			errCh := make(chan error, 1)
			go func() {
				errCh <- h.handler(timeoutCtx, event)
			}()
			select {
			case err := <-errCh:
				cancel()
				if err != nil {
					return &EventBusError{Kind: HandlerFailed, Message: err.Error(), Err: err}
				}
			case <-timeoutCtx.Done():
				cancel()
				return &EventBusError{Kind: HandlerFailed, Message: "handler timed out"}
			}
		} else {
			if err := h.handler(ctx, event); err != nil {
				return &EventBusError{Kind: HandlerFailed, Message: err.Error(), Err: err}
			}
		}
	}
	return nil
}

// --- レガシー API (後方互換性のため維持) ---

// Event はイベント（レガシー）。
type Event struct {
	ID        string         `json:"id"`
	EventType string         `json:"event_type"`
	Payload   map[string]any `json:"payload"`
	Timestamp time.Time      `json:"timestamp"`
}

// Handler はイベントハンドラー（レガシー）。
type Handler func(ctx context.Context, event Event) error

// LegacyEventBus はイベントバスのインターフェース（レガシー）。
type LegacyEventBus interface {
	Subscribe(eventType string, handler Handler)
	Publish(ctx context.Context, event Event) error
	Unsubscribe(eventType string)
}

// InMemoryBus はメモリ内のイベントバス（レガシー）。
type InMemoryBus struct {
	mu       sync.RWMutex
	handlers map[string][]Handler
}

// New は新しい InMemoryBus を生成する（レガシー）。
func New() *InMemoryBus {
	return &InMemoryBus{
		handlers: make(map[string][]Handler),
	}
}

func (b *InMemoryBus) Subscribe(eventType string, handler Handler) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.handlers[eventType] = append(b.handlers[eventType], handler)
}

func (b *InMemoryBus) Publish(ctx context.Context, event Event) error {
	b.mu.RLock()
	handlers := b.handlers[event.EventType]
	b.mu.RUnlock()

	for _, h := range handlers {
		if err := h(ctx, event); err != nil {
			return err
		}
	}
	return nil
}

func (b *InMemoryBus) Unsubscribe(eventType string) {
	b.mu.Lock()
	defer b.mu.Unlock()
	delete(b.handlers, eventType)
}
