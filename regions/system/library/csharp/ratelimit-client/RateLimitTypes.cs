namespace K1s0.System.RateLimitClient;

public record RateLimitStatus(
    bool Allowed,
    uint Remaining,
    DateTimeOffset ResetAt,
    ulong? RetryAfterSecs = null);

public record RateLimitResult(
    uint Remaining,
    DateTimeOffset ResetAt);

public record RateLimitPolicy(
    string Key,
    uint Limit,
    ulong WindowSecs,
    string Algorithm);
