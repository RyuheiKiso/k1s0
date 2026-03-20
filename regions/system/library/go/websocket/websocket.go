package websocket

import (
	"context"
	"errors"
	"sync"
)

// ErrAlreadyConnected は既に接続済みの場合のエラー。
var ErrAlreadyConnected = errors.New("already connected")

// ErrAlreadyDisconnected は既に切断済みの場合のエラー。
var ErrAlreadyDisconnected = errors.New("already disconnected")

// ErrNotConnected は未接続状態の場合のエラー。
var ErrNotConnected = errors.New("not connected")

// MessageType はメッセージ種別。
type MessageType int

const (
	MessageText MessageType = iota
	MessageBinary
	MessagePing
	MessagePong
	MessageClose
)

// WsMessage は WebSocket メッセージ。
type WsMessage struct {
	Type    MessageType
	Payload []byte
}

// ConnectionState は接続状態。
type ConnectionState int

const (
	StateDisconnected ConnectionState = iota
	StateConnecting
	StateConnected
	StateReconnecting
	StateClosing
)

// WsConfig は WebSocket 設定。
type WsConfig struct {
	URL                  string
	Reconnect            bool
	MaxReconnectAttempts int
	ReconnectDelayMs     int64
	PingIntervalMs       *int64
}

// DefaultConfig はデフォルト設定を返す。URL は呼び出し側が必ず指定すること。
func DefaultConfig() WsConfig {
	return WsConfig{
		Reconnect:            true,
		MaxReconnectAttempts: 5,
		ReconnectDelayMs:     1000,
	}
}

// WsClient はWebSocketクライアントのインターフェース。
type WsClient interface {
	Connect(ctx context.Context) error
	Disconnect(ctx context.Context) error
	Send(ctx context.Context, msg WsMessage) error
	Receive(ctx context.Context) (WsMessage, error)
	State() ConnectionState
}

// InMemoryWsClient はメモリ内のWebSocketクライアント。
type InMemoryWsClient struct {
	state   ConnectionState
	sendBuf chan WsMessage
	recvBuf chan WsMessage
	mu      sync.Mutex
}

// NewInMemoryWsClient は新しい InMemoryWsClient を生成する。
func NewInMemoryWsClient() *InMemoryWsClient {
	return &InMemoryWsClient{
		state:   StateDisconnected,
		sendBuf: make(chan WsMessage, 100),
		recvBuf: make(chan WsMessage, 100),
	}
}

// Connect は接続する。
func (c *InMemoryWsClient) Connect(_ context.Context) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.state == StateConnected {
		return ErrAlreadyConnected
	}
	c.state = StateConnected
	return nil
}

// Disconnect は切断する。
func (c *InMemoryWsClient) Disconnect(_ context.Context) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.state == StateDisconnected {
		return ErrAlreadyDisconnected
	}
	c.state = StateDisconnected
	return nil
}

// Send はメッセージを送信する。
// バッファ満杯時にブロックせず、context がキャンセルされたらエラーを返す。
func (c *InMemoryWsClient) Send(ctx context.Context, msg WsMessage) error {
	c.mu.Lock()
	state := c.state
	c.mu.Unlock()
	if state != StateConnected {
		return ErrNotConnected
	}
	// バッファ満杯時にブロックせず、context がキャンセルされたらエラーを返す
	select {
	case c.sendBuf <- msg:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

// Receive はメッセージを受信する。
func (c *InMemoryWsClient) Receive(ctx context.Context) (WsMessage, error) {
	c.mu.Lock()
	state := c.state
	c.mu.Unlock()
	if state != StateConnected {
		return WsMessage{}, ErrNotConnected
	}
	select {
	case msg := <-c.recvBuf:
		return msg, nil
	case <-ctx.Done():
		return WsMessage{}, ctx.Err()
	}
}

// State は現在の接続状態を返す。
func (c *InMemoryWsClient) State() ConnectionState {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.state
}

// InjectMessage はテスト用に受信バッファにメッセージを入れる。
func (c *InMemoryWsClient) InjectMessage(msg WsMessage) {
	c.recvBuf <- msg
}

// SentMessages は送信バッファからメッセージを取り出す（テスト用）。
func (c *InMemoryWsClient) SentMessages() []WsMessage {
	var msgs []WsMessage
	for {
		select {
		case msg := <-c.sendBuf:
			msgs = append(msgs, msg)
		default:
			return msgs
		}
	}
}
