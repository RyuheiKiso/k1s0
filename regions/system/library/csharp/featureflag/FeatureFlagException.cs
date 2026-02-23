namespace K1s0.System.FeatureFlag;

public class FeatureFlagNotFoundException(string flagKey)
    : Exception($"Flag not found: {flagKey}")
{
    public string FlagKey { get; } = flagKey;
}
