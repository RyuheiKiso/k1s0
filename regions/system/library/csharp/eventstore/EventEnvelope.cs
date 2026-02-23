using System.Text.Json;

namespace K1s0.System.EventStore;

public record EventEnvelope(
    string EventId,
    string StreamId,
    long Version,
    string EventType,
    JsonElement Payload,
    JsonElement? Metadata = null,
    DateTimeOffset? RecordedAt = null);
