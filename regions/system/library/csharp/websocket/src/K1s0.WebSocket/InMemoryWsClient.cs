using System.Threading.Channels;

namespace K1s0.WebSocket;

public class InMemoryWsClient : IWsClient
{
    private readonly Channel<WsMessage> _receiveChannel = Channel.CreateUnbounded<WsMessage>();
    private readonly List<WsMessage> _sentMessages = new();

    public ConnectionState State { get; private set; } = ConnectionState.Disconnected;

    public IReadOnlyList<WsMessage> SentMessages => _sentMessages.AsReadOnly();

    public void InjectMessage(WsMessage message) =>
        _receiveChannel.Writer.TryWrite(message);

    public Task ConnectAsync(CancellationToken ct = default)
    {
        State = ConnectionState.Connecting;
        State = ConnectionState.Connected;
        return Task.CompletedTask;
    }

    public Task DisconnectAsync(CancellationToken ct = default)
    {
        State = ConnectionState.Closing;
        State = ConnectionState.Disconnected;
        return Task.CompletedTask;
    }

    public Task SendAsync(WsMessage message, CancellationToken ct = default)
    {
        if (State != ConnectionState.Connected)
        {
            throw new InvalidOperationException($"Cannot send message while {State}");
        }

        _sentMessages.Add(message);
        return Task.CompletedTask;
    }

    public async Task<WsMessage> ReceiveAsync(CancellationToken ct = default)
    {
        if (_receiveChannel.Reader.TryRead(out var message))
        {
            return message;
        }

        return await _receiveChannel.Reader.ReadAsync(ct);
    }
}
