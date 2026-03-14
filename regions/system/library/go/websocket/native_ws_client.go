package websocket

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/gorilla/websocket"
)

// NativeWsClient は gorilla/websocket を使用した本番用 WebSocket クライアント実装。
// 自動再接続、Ping/Pong ハートビート、スレッドセーフなメッセージ送受信をサポートする。
type NativeWsClient struct {
	config  WsConfig
	conn    *websocket.Conn
	mu      sync.RWMutex // state と conn を保護する
	writeMu sync.Mutex   // gorilla/websocket の書き込みはgoroutine-safeではないため
	state   ConnectionState
	recvCh  chan WsMessage
	quit    chan struct{} // Disconnect() でcloseしてバックグラウンドgoroutineを停止する
	done    chan struct{} // readLoop が終了したことを通知する
	once    sync.Once    // quit チャネルを一度だけcloseするため
}

// NewNativeWsClient は指定した設定で NativeWsClient を生成する。
func NewNativeWsClient(config WsConfig) *NativeWsClient {
	return &NativeWsClient{
		config: config,
		state:  StateDisconnected,
		recvCh: make(chan WsMessage, 100),
	}
}

// Connect は WebSocket 接続を確立する。接続済みの場合はエラーを返す。
func (c *NativeWsClient) Connect(ctx context.Context) error {
	c.mu.Lock()
	if c.state != StateDisconnected {
		c.mu.Unlock()
		return fmt.Errorf("already connected")
	}
	c.state = StateConnecting
	c.quit = make(chan struct{})
	c.done = make(chan struct{})
	c.once = sync.Once{}
	c.mu.Unlock()

	conn, _, err := websocket.DefaultDialer.DialContext(ctx, c.config.URL, nil)
	if err != nil {
		c.mu.Lock()
		c.state = StateDisconnected
		close(c.done)
		c.mu.Unlock()
		return fmt.Errorf("websocket connect: %w", err)
	}

	c.mu.Lock()
	c.conn = conn
	c.state = StateConnected
	c.mu.Unlock()

	go c.readLoop()
	if c.config.PingIntervalMs != nil && *c.config.PingIntervalMs > 0 {
		go c.pingLoop()
	}
	return nil
}

// Disconnect は接続を閉じてリソースを解放する。
func (c *NativeWsClient) Disconnect(ctx context.Context) error {
	c.mu.Lock()
	if c.state == StateDisconnected {
		c.mu.Unlock()
		return fmt.Errorf("not connected")
	}
	c.state = StateClosing
	conn := c.conn
	done := c.done
	c.mu.Unlock()

	// バックグラウンド goroutine を停止させる
	c.once.Do(func() { close(c.quit) })

	// クローズメッセージを送信してから接続を閉じる
	c.writeMu.Lock()
	_ = conn.WriteMessage(
		websocket.CloseMessage,
		websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""),
	)
	_ = conn.Close()
	c.writeMu.Unlock()

	// readLoop が終了するまで待機する
	select {
	case <-done:
	case <-ctx.Done():
	}

	c.mu.Lock()
	c.state = StateDisconnected
	c.mu.Unlock()
	return nil
}

// Send は WebSocket メッセージを送信する。
func (c *NativeWsClient) Send(_ context.Context, msg WsMessage) error {
	c.mu.RLock()
	state := c.state
	conn := c.conn
	c.mu.RUnlock()

	if state != StateConnected {
		return fmt.Errorf("not connected")
	}

	msgType, data, err := toGorillaMessage(msg)
	if err != nil {
		return err
	}

	c.writeMu.Lock()
	defer c.writeMu.Unlock()
	return conn.WriteMessage(msgType, data)
}

// Receive は次の WebSocket メッセージを受信する。コンテキストがキャンセルされるまでブロックする。
func (c *NativeWsClient) Receive(ctx context.Context) (WsMessage, error) {
	c.mu.RLock()
	state := c.state
	c.mu.RUnlock()

	if state != StateConnected {
		return WsMessage{}, fmt.Errorf("not connected")
	}

	select {
	case msg, ok := <-c.recvCh:
		if !ok {
			return WsMessage{}, fmt.Errorf("connection closed")
		}
		return msg, nil
	case <-ctx.Done():
		return WsMessage{}, ctx.Err()
	}
}

// State は現在の接続状態を返す。
func (c *NativeWsClient) State() ConnectionState {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.state
}

// readLoop はバックグラウンドでメッセージを読み続け、接続が切れた場合に再接続を試みる。
func (c *NativeWsClient) readLoop() {
	defer close(c.done)

	for {
		c.mu.RLock()
		conn := c.conn
		c.mu.RUnlock()

		msgType, data, err := conn.ReadMessage()
		if err != nil {
			// quit が要求されている場合は終了する
			select {
			case <-c.quit:
				return
			default:
			}

			// 再接続が無効なら終了する
			if !c.config.Reconnect {
				c.mu.Lock()
				c.state = StateDisconnected
				c.mu.Unlock()
				return
			}

			// 再接続を試みる
			newConn := c.tryReconnect()
			if newConn == nil {
				c.mu.Lock()
				c.state = StateDisconnected
				c.mu.Unlock()
				return
			}

			c.mu.Lock()
			c.conn = newConn
			c.state = StateConnected
			c.mu.Unlock()
			continue
		}

		msg, err := fromGorillaMessage(msgType, data)
		if err != nil {
			continue
		}

		select {
		case c.recvCh <- msg:
		case <-c.quit:
			return
		}
	}
}

// tryReconnect は再接続を MaxReconnectAttempts 回試みる。成功した場合は新しい接続を返す。
func (c *NativeWsClient) tryReconnect() *websocket.Conn {
	c.mu.Lock()
	c.state = StateReconnecting
	c.mu.Unlock()

	for attempt := 0; attempt < c.config.MaxReconnectAttempts; attempt++ {
		select {
		case <-c.quit:
			return nil
		case <-time.After(time.Duration(c.config.ReconnectDelayMs) * time.Millisecond):
		}

		conn, _, err := websocket.DefaultDialer.DialContext(context.Background(), c.config.URL, nil)
		if err == nil {
			return conn
		}
	}

	return nil
}

// pingLoop はバックグラウンドで定期的に Ping メッセージを送信して接続を維持する。
func (c *NativeWsClient) pingLoop() {
	interval := time.Duration(*c.config.PingIntervalMs) * time.Millisecond
	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	for {
		select {
		case <-c.quit:
			return
		case <-ticker.C:
			c.mu.RLock()
			state := c.state
			conn := c.conn
			c.mu.RUnlock()

			if state != StateConnected {
				continue
			}

			c.writeMu.Lock()
			_ = conn.WriteMessage(websocket.PingMessage, nil)
			c.writeMu.Unlock()
		}
	}
}

// toGorillaMessage は WsMessage を gorilla/websocket のメッセージ型に変換する。
func toGorillaMessage(msg WsMessage) (int, []byte, error) {
	switch msg.Type {
	case MessageText:
		return websocket.TextMessage, msg.Payload, nil
	case MessageBinary:
		return websocket.BinaryMessage, msg.Payload, nil
	case MessagePing:
		return websocket.PingMessage, msg.Payload, nil
	case MessagePong:
		return websocket.PongMessage, msg.Payload, nil
	case MessageClose:
		return websocket.CloseMessage, msg.Payload, nil
	default:
		return 0, nil, fmt.Errorf("unknown message type: %d", msg.Type)
	}
}

// fromGorillaMessage は gorilla/websocket のメッセージ型を WsMessage に変換する。
func fromGorillaMessage(msgType int, data []byte) (WsMessage, error) {
	switch msgType {
	case websocket.TextMessage:
		return WsMessage{Type: MessageText, Payload: data}, nil
	case websocket.BinaryMessage:
		return WsMessage{Type: MessageBinary, Payload: data}, nil
	case websocket.PingMessage:
		return WsMessage{Type: MessagePing, Payload: data}, nil
	case websocket.PongMessage:
		return WsMessage{Type: MessagePong, Payload: data}, nil
	case websocket.CloseMessage:
		return WsMessage{Type: MessageClose, Payload: data}, nil
	default:
		return WsMessage{}, fmt.Errorf("unknown message type: %d", msgType)
	}
}
