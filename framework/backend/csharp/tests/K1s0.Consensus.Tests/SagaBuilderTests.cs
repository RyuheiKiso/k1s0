using FluentAssertions;
using Xunit;

namespace K1s0.Consensus.Tests;

public class SagaBuilderTests
{
    private sealed class TestContext
    {
        public List<string> ExecutedSteps { get; } = [];
        public List<string> CompensatedSteps { get; } = [];
    }

    [Fact]
    public void Build_WithNoSteps_ThrowsInvalidOperationException()
    {
        var builder = new SagaBuilder<TestContext>("empty-saga");

        var act = () => builder.Build();
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*at least one step*");
    }

    [Fact]
    public void Build_WithDelegateSteps_CreatesDefinition()
    {
        var definition = new SagaBuilder<TestContext>("test-saga")
            .AddStep(
                "step-1",
                async (ctx, ct) =>
                {
                    ctx.ExecutedSteps.Add("step-1");
                    return ctx;
                },
                async (ctx, ct) =>
                {
                    ctx.CompensatedSteps.Add("step-1");
                    return ctx;
                })
            .AddStep(
                "step-2",
                async (ctx, ct) =>
                {
                    ctx.ExecutedSteps.Add("step-2");
                    return ctx;
                },
                async (ctx, ct) =>
                {
                    ctx.CompensatedSteps.Add("step-2");
                    return ctx;
                })
            .Build();

        definition.Name.Should().Be("test-saga");
        definition.Steps.Should().HaveCount(2);
        definition.Steps[0].Name.Should().Be("step-1");
        definition.Steps[1].Name.Should().Be("step-2");
    }

    [Fact]
    public void Build_WithCustomRetryPolicies_AppliesCorrectly()
    {
        var customPolicy = new RetryPolicy { MaxRetries = 5, Strategy = BackoffStrategy.Linear };

        var definition = new SagaBuilder<TestContext>("retry-saga")
            .AddStep("step-1",
                async (ctx, ct) => ctx,
                async (ctx, ct) => ctx)
            .WithRetryPolicy("step-1", customPolicy)
            .WithDefaultRetryPolicy(new RetryPolicy { MaxRetries = 1 })
            .Build();

        definition.GetRetryPolicy("step-1").Should().BeSameAs(customPolicy);
        definition.GetRetryPolicy("step-1").MaxRetries.Should().Be(5);
        definition.GetRetryPolicy("step-unknown").MaxRetries.Should().Be(1);
    }

    [Fact]
    public async Task DelegateStep_Execute_InvokesDelegate()
    {
        var context = new TestContext();

        var definition = new SagaBuilder<TestContext>("exec-saga")
            .AddStep("step-1",
                async (ctx, ct) =>
                {
                    ctx.ExecutedSteps.Add("step-1");
                    return ctx;
                },
                async (ctx, ct) =>
                {
                    ctx.CompensatedSteps.Add("step-1");
                    return ctx;
                })
            .Build();

        var result = await definition.Steps[0].ExecuteAsync(context);
        result.ExecutedSteps.Should().Contain("step-1");
    }

    [Fact]
    public async Task DelegateStep_Compensate_InvokesDelegate()
    {
        var context = new TestContext();

        var definition = new SagaBuilder<TestContext>("comp-saga")
            .AddStep("step-1",
                async (ctx, ct) => ctx,
                async (ctx, ct) =>
                {
                    ctx.CompensatedSteps.Add("step-1");
                    return ctx;
                })
            .Build();

        var result = await definition.Steps[0].CompensateAsync(context);
        result.CompensatedSteps.Should().Contain("step-1");
    }

    [Fact]
    public void Build_WithISagaStep_CreatesDefinition()
    {
        var step = new TestSagaStep("custom-step");

        var definition = new SagaBuilder<TestContext>("custom-saga")
            .AddStep(step)
            .Build();

        definition.Steps.Should().HaveCount(1);
        definition.Steps[0].Should().BeSameAs(step);
    }

    [Theory]
    [InlineData(BackoffStrategy.Fixed, 0, 1000)]
    [InlineData(BackoffStrategy.Fixed, 3, 1000)]
    [InlineData(BackoffStrategy.Linear, 0, 1000)]
    [InlineData(BackoffStrategy.Linear, 2, 3000)]
    [InlineData(BackoffStrategy.Exponential, 0, 1000)]
    [InlineData(BackoffStrategy.Exponential, 3, 8000)]
    public void RetryPolicy_GetDelay_CalculatesCorrectly(BackoffStrategy strategy, int attempt, double expectedMs)
    {
        var policy = new RetryPolicy
        {
            BaseDelay = TimeSpan.FromSeconds(1),
            MaxDelay = TimeSpan.FromSeconds(60),
            Strategy = strategy
        };

        policy.GetDelay(attempt).TotalMilliseconds.Should().Be(expectedMs);
    }

    [Fact]
    public void RetryPolicy_GetDelay_RespectsMaxDelay()
    {
        var policy = new RetryPolicy
        {
            BaseDelay = TimeSpan.FromSeconds(1),
            MaxDelay = TimeSpan.FromSeconds(5),
            Strategy = BackoffStrategy.Exponential
        };

        // 2^10 * 1000 = 1024000ms, but capped at 5000ms
        policy.GetDelay(10).TotalMilliseconds.Should().Be(5000);
    }

    [Fact]
    public void RetryPolicy_None_HasZeroRetries()
    {
        RetryPolicy.None.MaxRetries.Should().Be(0);
    }

    private sealed class TestSagaStep : ISagaStep<TestContext>
    {
        public string Name { get; }

        public TestSagaStep(string name) => Name = name;

        public Task<TestContext> ExecuteAsync(TestContext context, CancellationToken cancellationToken = default)
        {
            context.ExecutedSteps.Add(Name);
            return Task.FromResult(context);
        }

        public Task<TestContext> CompensateAsync(TestContext context, CancellationToken cancellationToken = default)
        {
            context.CompensatedSteps.Add(Name);
            return Task.FromResult(context);
        }
    }
}
