namespace K1s0.WebSocket;

public record WsConfig(
    string Url = "ws://localhost",
    bool Reconnect = true,
    int MaxReconnectAttempts = 5,
    TimeSpan? ReconnectDelay = null,
    TimeSpan? PingInterval = null)
{
    public static WsConfig Default => new();
}
