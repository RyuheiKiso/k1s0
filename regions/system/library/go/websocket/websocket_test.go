package websocket_test

import (
	"context"
	"testing"

	websocket "github.com/k1s0-platform/system-library-go-websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Connectが切断状態のクライアントを接続状態に遷移させることを確認する。
func TestConnect(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	assert.Equal(t, websocket.StateDisconnected, c.State())

	err := c.Connect(ctx)
	require.NoError(t, err)
	assert.Equal(t, websocket.StateConnected, c.State())
}

// Connectが既に接続済みのクライアントに対してエラーを返すことを確認する。
func TestConnect_AlreadyConnected(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	_ = c.Connect(ctx)
	err := c.Connect(ctx)
	require.Error(t, err)
}

// Disconnectが接続中のクライアントを切断状態に遷移させることを確認する。
func TestDisconnect(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	_ = c.Connect(ctx)
	err := c.Disconnect(ctx)
	require.NoError(t, err)
	assert.Equal(t, websocket.StateDisconnected, c.State())
}

// Disconnectが既に切断済みのクライアントに対してエラーを返すことを確認する。
func TestDisconnect_AlreadyDisconnected(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	err := c.Disconnect(ctx)
	require.Error(t, err)
}

// Sendでメッセージを送信しInjectMessageで注入したメッセージをReceiveで受信できることを確認する。
func TestSendReceive(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	_ = c.Connect(ctx)

	// Send a message
	sendMsg := websocket.WsMessage{Type: websocket.MessageText, Payload: []byte("hello")}
	err := c.Send(ctx, sendMsg)
	require.NoError(t, err)

	sent := c.SentMessages()
	require.Len(t, sent, 1)
	assert.Equal(t, []byte("hello"), sent[0].Payload)

	// Receive via injection
	injected := websocket.WsMessage{Type: websocket.MessageText, Payload: []byte("world")}
	c.InjectMessage(injected)

	received, err := c.Receive(ctx)
	require.NoError(t, err)
	assert.Equal(t, []byte("world"), received.Payload)
}

// Sendが未接続状態のクライアントに対してエラーを返すことを確認する。
func TestSend_NotConnected(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	err := c.Send(ctx, websocket.WsMessage{Type: websocket.MessageText, Payload: []byte("hello")})
	require.Error(t, err)
}

// Receiveが未接続状態のクライアントに対してエラーを返すことを確認する。
func TestReceive_NotConnected(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	_, err := c.Receive(ctx)
	require.Error(t, err)
}

// DefaultConfigがデフォルトのWebSocket接続設定を正しく返すことを確認する。URLはデフォルトで空文字列。
func TestDefaultConfig(t *testing.T) {
	cfg := websocket.DefaultConfig()
	// URL はデフォルト値を持たず、呼び出し側が明示的に指定する前提
	assert.Equal(t, "", cfg.URL)
	assert.True(t, cfg.Reconnect)
	assert.Equal(t, 5, cfg.MaxReconnectAttempts)
	assert.Equal(t, int64(1000), cfg.ReconnectDelayMs)
	assert.Nil(t, cfg.PingIntervalMs)
}

// DefaultConfigにURLを明示的に設定して使用するパターンを確認する。
func TestDefaultConfig_WithExplicitURL(t *testing.T) {
	cfg := websocket.DefaultConfig()
	cfg.URL = "ws://test.example.com"
	assert.Equal(t, "ws://test.example.com", cfg.URL)
	assert.True(t, cfg.Reconnect)
}
