using K1s0.System.FeatureFlag;

namespace K1s0.System.FeatureFlag.Tests;

public class FeatureFlagTests
{
    [Fact]
    public async Task Evaluate_EnabledFlag_ReturnsEnabled()
    {
        var client = new InMemoryFeatureFlagClient();
        client.SetFlag(new FeatureFlag("1", "feat-1", "test", true, []));

        var result = await client.EvaluateAsync("feat-1", new EvaluationContext());
        Assert.True(result.Enabled);
        Assert.Equal("Enabled", result.Reason);
    }

    [Fact]
    public async Task Evaluate_DisabledFlag_ReturnsFalse()
    {
        var client = new InMemoryFeatureFlagClient();
        client.SetFlag(new FeatureFlag("1", "feat-off", "off", false, []));

        var result = await client.EvaluateAsync("feat-off", new EvaluationContext());
        Assert.False(result.Enabled);
        Assert.Equal("Disabled", result.Reason);
    }

    [Fact]
    public async Task Evaluate_NonExistentFlag_ReturnsFlagNotFound()
    {
        var client = new InMemoryFeatureFlagClient();

        var result = await client.EvaluateAsync("missing", new EvaluationContext());
        Assert.False(result.Enabled);
        Assert.Equal("FlagNotFound", result.Reason);
    }

    [Fact]
    public async Task Evaluate_WithVariants_AssignsVariant()
    {
        var client = new InMemoryFeatureFlagClient();
        var variants = new List<FlagVariant>
        {
            new("control", "A", 50),
            new("experiment", "B", 50),
        };
        client.SetFlag(new FeatureFlag("1", "ab-test", "ab", true, variants));

        var result = await client.EvaluateAsync("ab-test", new EvaluationContext(UserId: "user-1"));
        Assert.True(result.Enabled);
        Assert.NotNull(result.Variant);
        Assert.Equal("VariantAssigned", result.Reason);
    }

    [Fact]
    public async Task IsEnabled_EnabledFlag_ReturnsTrue()
    {
        var client = new InMemoryFeatureFlagClient();
        client.SetFlag(new FeatureFlag("1", "feat-on", "on", true, []));

        Assert.True(await client.IsEnabledAsync("feat-on", new EvaluationContext()));
    }

    [Fact]
    public async Task IsEnabled_NonExistentFlag_ReturnsFalse()
    {
        var client = new InMemoryFeatureFlagClient();
        Assert.False(await client.IsEnabledAsync("nope", new EvaluationContext()));
    }

    [Fact]
    public async Task GetFlag_Existing_ReturnsFlag()
    {
        var client = new InMemoryFeatureFlagClient();
        client.SetFlag(new FeatureFlag("1", "f1", "desc", true, []));

        var flag = await client.GetFlagAsync("f1");
        Assert.Equal("f1", flag.FlagKey);
    }

    [Fact]
    public async Task GetFlag_NonExistent_ThrowsNotFoundException()
    {
        var client = new InMemoryFeatureFlagClient();
        await Assert.ThrowsAsync<FeatureFlagNotFoundException>(() => client.GetFlagAsync("missing"));
    }
}
