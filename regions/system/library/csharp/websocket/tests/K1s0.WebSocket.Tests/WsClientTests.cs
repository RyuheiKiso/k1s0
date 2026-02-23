using System.Text;
using K1s0.WebSocket;

namespace K1s0.WebSocket.Tests;

public class WsClientTests
{
    [Fact]
    public void WsConfig_Default_HasExpectedValues()
    {
        var config = WsConfig.Default;
        Assert.Equal("ws://localhost", config.Url);
        Assert.True(config.Reconnect);
        Assert.Equal(5, config.MaxReconnectAttempts);
        Assert.Null(config.ReconnectDelay);
        Assert.Null(config.PingInterval);
    }

    [Fact]
    public void WsConfig_CustomValues()
    {
        var config = new WsConfig(
            Url: "ws://example.com",
            Reconnect: false,
            MaxReconnectAttempts: 3,
            ReconnectDelay: TimeSpan.FromSeconds(2),
            PingInterval: TimeSpan.FromSeconds(30));

        Assert.Equal("ws://example.com", config.Url);
        Assert.False(config.Reconnect);
        Assert.Equal(3, config.MaxReconnectAttempts);
    }

    [Fact]
    public void WsMessage_CreatesWithTextPayload()
    {
        var payload = Encoding.UTF8.GetBytes("hello");
        var msg = new WsMessage(MessageType.Text, payload);
        Assert.Equal(MessageType.Text, msg.Type);
        Assert.Equal("hello", Encoding.UTF8.GetString(msg.Payload.Span));
    }

    [Fact]
    public void WsMessage_CreatesWithBinaryPayload()
    {
        var payload = new byte[] { 1, 2, 3 };
        var msg = new WsMessage(MessageType.Binary, payload);
        Assert.Equal(MessageType.Binary, msg.Type);
        Assert.Equal(3, msg.Payload.Length);
    }

    [Fact]
    public void ConnectionState_HasAllValues()
    {
        var values = Enum.GetValues<ConnectionState>();
        Assert.Equal(5, values.Length);
    }

    [Fact]
    public void InMemoryWsClient_StartsDisconnected()
    {
        var client = new InMemoryWsClient();
        Assert.Equal(ConnectionState.Disconnected, client.State);
    }

    [Fact]
    public async Task Connect_TransitionsToConnected()
    {
        var client = new InMemoryWsClient();
        await client.ConnectAsync();
        Assert.Equal(ConnectionState.Connected, client.State);
    }

    [Fact]
    public async Task Disconnect_TransitionsToDisconnected()
    {
        var client = new InMemoryWsClient();
        await client.ConnectAsync();
        await client.DisconnectAsync();
        Assert.Equal(ConnectionState.Disconnected, client.State);
    }

    [Fact]
    public async Task Send_StoresMessages()
    {
        var client = new InMemoryWsClient();
        await client.ConnectAsync();
        var msg = new WsMessage(MessageType.Text, Encoding.UTF8.GetBytes("test"));
        await client.SendAsync(msg);
        Assert.Single(client.SentMessages);
    }

    [Fact]
    public async Task Send_ThrowsWhenNotConnected()
    {
        var client = new InMemoryWsClient();
        var msg = new WsMessage(MessageType.Text, Encoding.UTF8.GetBytes("test"));
        await Assert.ThrowsAsync<InvalidOperationException>(() => client.SendAsync(msg));
    }

    [Fact]
    public async Task Receive_ReturnsInjectedMessage()
    {
        var client = new InMemoryWsClient();
        var msg = new WsMessage(MessageType.Text, Encoding.UTF8.GetBytes("incoming"));
        client.InjectMessage(msg);
        var received = await client.ReceiveAsync();
        Assert.Equal("incoming", Encoding.UTF8.GetString(received.Payload.Span));
    }

    [Fact]
    public async Task Receive_ReturnsMessagesInOrder()
    {
        var client = new InMemoryWsClient();
        client.InjectMessage(new WsMessage(MessageType.Text, Encoding.UTF8.GetBytes("first")));
        client.InjectMessage(new WsMessage(MessageType.Text, Encoding.UTF8.GetBytes("second")));
        var first = await client.ReceiveAsync();
        var second = await client.ReceiveAsync();
        Assert.Equal("first", Encoding.UTF8.GetString(first.Payload.Span));
        Assert.Equal("second", Encoding.UTF8.GetString(second.Payload.Span));
    }

    [Fact]
    public async Task Send_PingMessage()
    {
        var client = new InMemoryWsClient();
        await client.ConnectAsync();
        var msg = new WsMessage(MessageType.Ping, ReadOnlyMemory<byte>.Empty);
        await client.SendAsync(msg);
        Assert.Equal(MessageType.Ping, client.SentMessages[0].Type);
    }
}
