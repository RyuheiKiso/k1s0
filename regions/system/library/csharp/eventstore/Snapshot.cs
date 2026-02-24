using System.Text.Json;

namespace K1s0.System.EventStore;

public record Snapshot(
    string StreamId,
    long Version,
    JsonElement State,
    DateTimeOffset CreatedAt);
