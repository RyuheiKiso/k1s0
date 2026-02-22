namespace K1s0.System.Messaging;

public sealed record EventEnvelope(
    string Topic,
    string? Key,
    byte[] Payload,
    EventMetadata Metadata,
    IDictionary<string, string>? Headers = null);
