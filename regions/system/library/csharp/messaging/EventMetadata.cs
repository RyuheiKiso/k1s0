namespace K1s0.System.Messaging;

public sealed record EventMetadata(
    string Id,
    string EventType,
    string Source,
    DateTimeOffset Timestamp,
    string TraceId,
    string CorrelationId,
    string SchemaVersion)
{
    public static EventMetadata New(
        string eventType,
        string source,
        string traceId,
        string correlationId,
        string schemaVersion = "1.0")
    {
        return new EventMetadata(
            Id: Guid.NewGuid().ToString(),
            EventType: eventType,
            Source: source,
            Timestamp: DateTimeOffset.UtcNow,
            TraceId: traceId,
            CorrelationId: correlationId,
            SchemaVersion: schemaVersion);
    }
}
