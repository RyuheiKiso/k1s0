namespace K1s0.System.FeatureFlag;

public record EvaluationContext(
    string? UserId = null,
    string? TenantId = null,
    Dictionary<string, string>? Attributes = null);
