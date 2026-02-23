namespace K1s0.SessionClient;

public record Session(
    string Id,
    string UserId,
    string Token,
    DateTimeOffset ExpiresAt,
    DateTimeOffset CreatedAt,
    bool Revoked,
    IReadOnlyDictionary<string, string> Metadata);

public record CreateSessionRequest(
    string UserId,
    long TtlSeconds,
    IReadOnlyDictionary<string, string>? Metadata = null);

public record RefreshSessionRequest(string Id, long TtlSeconds);
