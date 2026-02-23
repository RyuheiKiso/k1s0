namespace K1s0.System.FeatureFlag;

public record FlagVariant(string Name, string Value, int Weight);

public record FeatureFlag(
    string Id,
    string FlagKey,
    string Description,
    bool Enabled,
    IReadOnlyList<FlagVariant> Variants);

public record EvaluationResult(string FlagKey, bool Enabled, string? Variant, string Reason);
