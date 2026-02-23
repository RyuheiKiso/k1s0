package websocket

import (
	"context"
	"errors"
	"sync"
)

// MessageType はメッセージ種別。
type MessageType int

const (
	MessageText   MessageType = iota
	MessageBinary
	MessagePing
	MessagePong
	MessageClose
)

// Message はWebSocketメッセージ。
type Message struct {
	Type    MessageType
	Payload []byte
}

// ConnectionState は接続状態。
type ConnectionState int

const (
	StateDisconnected  ConnectionState = iota
	StateConnecting
	StateConnected
	StateReconnecting
	StateClosing
)

// Config はWebSocket設定。
type Config struct {
	URL                  string
	Reconnect            bool
	MaxReconnectAttempts int
	ReconnectDelayMs     int64
	PingIntervalMs       *int64
}

// DefaultConfig はデフォルト設定を返す。
func DefaultConfig() Config {
	return Config{
		URL:                  "ws://localhost",
		Reconnect:            true,
		MaxReconnectAttempts: 5,
		ReconnectDelayMs:     1000,
	}
}

// WsClient はWebSocketクライアントのインターフェース。
type WsClient interface {
	Connect(ctx context.Context) error
	Disconnect(ctx context.Context) error
	Send(ctx context.Context, msg Message) error
	Receive(ctx context.Context) (Message, error)
	State() ConnectionState
}

// InMemoryWsClient はメモリ内のWebSocketクライアント。
type InMemoryWsClient struct {
	state   ConnectionState
	sendBuf chan Message
	recvBuf chan Message
	mu      sync.Mutex
}

// NewInMemoryWsClient は新しい InMemoryWsClient を生成する。
func NewInMemoryWsClient() *InMemoryWsClient {
	return &InMemoryWsClient{
		state:   StateDisconnected,
		sendBuf: make(chan Message, 100),
		recvBuf: make(chan Message, 100),
	}
}

// Connect は接続する。
func (c *InMemoryWsClient) Connect(_ context.Context) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.state == StateConnected {
		return errors.New("already connected")
	}
	c.state = StateConnected
	return nil
}

// Disconnect は切断する。
func (c *InMemoryWsClient) Disconnect(_ context.Context) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.state == StateDisconnected {
		return errors.New("already disconnected")
	}
	c.state = StateDisconnected
	return nil
}

// Send はメッセージを送信する。
func (c *InMemoryWsClient) Send(_ context.Context, msg Message) error {
	c.mu.Lock()
	state := c.state
	c.mu.Unlock()
	if state != StateConnected {
		return errors.New("not connected")
	}
	c.sendBuf <- msg
	return nil
}

// Receive はメッセージを受信する。
func (c *InMemoryWsClient) Receive(ctx context.Context) (Message, error) {
	c.mu.Lock()
	state := c.state
	c.mu.Unlock()
	if state != StateConnected {
		return Message{}, errors.New("not connected")
	}
	select {
	case msg := <-c.recvBuf:
		return msg, nil
	case <-ctx.Done():
		return Message{}, ctx.Err()
	}
}

// State は現在の接続状態を返す。
func (c *InMemoryWsClient) State() ConnectionState {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.state
}

// InjectMessage はテスト用に受信バッファにメッセージを入れる。
func (c *InMemoryWsClient) InjectMessage(msg Message) {
	c.recvBuf <- msg
}

// SentMessages は送信バッファからメッセージを取り出す（テスト用）。
func (c *InMemoryWsClient) SentMessages() []Message {
	var msgs []Message
	for {
		select {
		case msg := <-c.sendBuf:
			msgs = append(msgs, msg)
		default:
			return msgs
		}
	}
}
