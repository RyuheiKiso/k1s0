namespace K1s0.System.FeatureFlag;

public interface IFeatureFlagClient
{
    Task<EvaluationResult> EvaluateAsync(string flagKey, EvaluationContext context);

    Task<FeatureFlag> GetFlagAsync(string flagKey);

    Task<bool> IsEnabledAsync(string flagKey, EvaluationContext context);

    Task<string?> GetVariationAsync(string flagKey, EvaluationContext context);
}
