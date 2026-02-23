namespace K1s0.System.AuditClient;

public record AuditEvent(
    string Id,
    string TenantId,
    string ActorId,
    string Action,
    string ResourceType,
    string ResourceId,
    DateTimeOffset Timestamp);
