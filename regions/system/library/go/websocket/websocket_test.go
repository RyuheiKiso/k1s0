package websocket_test

import (
	"context"
	"testing"

	websocket "github.com/k1s0-platform/system-library-go-websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestConnect(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	assert.Equal(t, websocket.StateDisconnected, c.State())

	err := c.Connect(ctx)
	require.NoError(t, err)
	assert.Equal(t, websocket.StateConnected, c.State())
}

func TestConnect_AlreadyConnected(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	_ = c.Connect(ctx)
	err := c.Connect(ctx)
	require.Error(t, err)
}

func TestDisconnect(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	_ = c.Connect(ctx)
	err := c.Disconnect(ctx)
	require.NoError(t, err)
	assert.Equal(t, websocket.StateDisconnected, c.State())
}

func TestDisconnect_AlreadyDisconnected(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	err := c.Disconnect(ctx)
	require.Error(t, err)
}

func TestSendReceive(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	_ = c.Connect(ctx)

	// Send a message
	sendMsg := websocket.Message{Type: websocket.MessageText, Payload: []byte("hello")}
	err := c.Send(ctx, sendMsg)
	require.NoError(t, err)

	sent := c.SentMessages()
	require.Len(t, sent, 1)
	assert.Equal(t, []byte("hello"), sent[0].Payload)

	// Receive via injection
	injected := websocket.Message{Type: websocket.MessageText, Payload: []byte("world")}
	c.InjectMessage(injected)

	received, err := c.Receive(ctx)
	require.NoError(t, err)
	assert.Equal(t, []byte("world"), received.Payload)
}

func TestSend_NotConnected(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	err := c.Send(ctx, websocket.Message{Type: websocket.MessageText, Payload: []byte("hello")})
	require.Error(t, err)
}

func TestReceive_NotConnected(t *testing.T) {
	c := websocket.NewInMemoryWsClient()
	ctx := context.Background()

	_, err := c.Receive(ctx)
	require.Error(t, err)
}

func TestDefaultConfig(t *testing.T) {
	cfg := websocket.DefaultConfig()
	assert.Equal(t, "ws://localhost", cfg.URL)
	assert.True(t, cfg.Reconnect)
	assert.Equal(t, 5, cfg.MaxReconnectAttempts)
	assert.Equal(t, int64(1000), cfg.ReconnectDelayMs)
	assert.Nil(t, cfg.PingIntervalMs)
}
