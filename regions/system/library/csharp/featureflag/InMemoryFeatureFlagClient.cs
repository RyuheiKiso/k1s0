namespace K1s0.System.FeatureFlag;

public class InMemoryFeatureFlagClient : IFeatureFlagClient
{
    private readonly Dictionary<string, FeatureFlag> _flags = new();

    public void SetFlag(FeatureFlag flag)
    {
        _flags[flag.FlagKey] = flag;
    }

    public Task<EvaluationResult> EvaluateAsync(string flagKey, EvaluationContext context)
    {
        if (!_flags.TryGetValue(flagKey, out var flag))
        {
            return Task.FromResult(new EvaluationResult(flagKey, false, null, "FlagNotFound"));
        }

        if (!flag.Enabled)
        {
            return Task.FromResult(new EvaluationResult(flagKey, false, null, "Disabled"));
        }

        if (flag.Variants.Count == 0)
        {
            return Task.FromResult(new EvaluationResult(flagKey, true, null, "Enabled"));
        }

        // Simple deterministic variant selection based on UserId hash
        var totalWeight = flag.Variants.Sum(v => v.Weight);
        if (totalWeight <= 0)
        {
            return Task.FromResult(new EvaluationResult(flagKey, true, flag.Variants[0].Value, "DefaultVariant"));
        }

        var hash = context.UserId != null
            ? Math.Abs(context.UserId.GetHashCode(StringComparison.Ordinal))
            : 0;
        var bucket = hash % totalWeight;
        var cumulative = 0;

        foreach (var variant in flag.Variants)
        {
            cumulative += variant.Weight;
            if (bucket < cumulative)
            {
                return Task.FromResult(new EvaluationResult(flagKey, true, variant.Value, "VariantAssigned"));
            }
        }

        return Task.FromResult(new EvaluationResult(flagKey, true, flag.Variants[^1].Value, "VariantAssigned"));
    }

    public Task<FeatureFlag> GetFlagAsync(string flagKey)
    {
        if (!_flags.TryGetValue(flagKey, out var flag))
        {
            throw new FeatureFlagNotFoundException(flagKey);
        }

        return Task.FromResult(flag);
    }

    public async Task<bool> IsEnabledAsync(string flagKey, EvaluationContext context)
    {
        var result = await EvaluateAsync(flagKey, context).ConfigureAwait(false);
        return result.Enabled;
    }
}
