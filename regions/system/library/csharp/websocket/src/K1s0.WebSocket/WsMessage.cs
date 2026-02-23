namespace K1s0.WebSocket;

public enum MessageType
{
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

public record WsMessage(MessageType Type, ReadOnlyMemory<byte> Payload);

public enum ConnectionState
{
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Closing,
}
