namespace K1s0.System.Messaging;

public sealed record ConsumedMessage(
    string Topic,
    int Partition,
    long Offset,
    string? Key,
    byte[] Payload,
    IDictionary<string, string> Headers);
