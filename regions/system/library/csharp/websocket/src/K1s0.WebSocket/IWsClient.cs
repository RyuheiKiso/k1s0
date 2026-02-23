namespace K1s0.WebSocket;

public interface IWsClient
{
    Task ConnectAsync(CancellationToken ct = default);

    Task DisconnectAsync(CancellationToken ct = default);

    Task SendAsync(WsMessage message, CancellationToken ct = default);

    Task<WsMessage> ReceiveAsync(CancellationToken ct = default);

    ConnectionState State { get; }
}
