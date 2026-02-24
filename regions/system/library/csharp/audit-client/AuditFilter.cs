namespace K1s0.System.AuditClient;

public record AuditFilter(
    string? TenantId = null,
    string? ActorId = null,
    string? Action = null,
    string? ResourceType = null,
    DateTimeOffset? From = null,
    DateTimeOffset? To = null);
