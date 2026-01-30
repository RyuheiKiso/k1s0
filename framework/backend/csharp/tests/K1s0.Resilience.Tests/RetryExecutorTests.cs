using FluentAssertions;
using Xunit;

namespace K1s0.Resilience.Tests;

public class RetryExecutorTests
{
    [Fact]
    public async Task ExecuteAsync_SucceedsOnFirstAttempt_ReturnsResult()
    {
        var executor = new RetryExecutor(new RetryConfig(MaxAttempts: 3, InitialIntervalMs: 10));

        var result = await executor.ExecuteAsync(() => Task.FromResult(42));

        result.Should().Be(42);
    }

    [Fact]
    public async Task ExecuteAsync_SucceedsAfterRetries_ReturnsResult()
    {
        var executor = new RetryExecutor(new RetryConfig(MaxAttempts: 3, InitialIntervalMs: 10));
        int attempts = 0;

        var result = await executor.ExecuteAsync(() =>
        {
            attempts++;
            if (attempts < 3)
            {
                throw new InvalidOperationException("transient");
            }

            return Task.FromResult(99);
        });

        result.Should().Be(99);
        attempts.Should().Be(3);
    }

    [Fact]
    public async Task ExecuteAsync_ExceedsMaxAttempts_ThrowsLastException()
    {
        var executor = new RetryExecutor(new RetryConfig(MaxAttempts: 2, InitialIntervalMs: 10));
        int attempts = 0;

        var act = () => executor.ExecuteAsync<int>(() =>
        {
            attempts++;
            throw new InvalidOperationException($"attempt {attempts}");
        });

        await act.Should().ThrowAsync<InvalidOperationException>()
            .WithMessage("attempt 2");
        attempts.Should().Be(2);
    }

    [Fact]
    public async Task ExecuteAsync_NonRetryableException_ThrowsImmediately()
    {
        var executor = new RetryExecutor(new RetryConfig(
            MaxAttempts: 3,
            InitialIntervalMs: 10,
            RetryableChecker: ex => ex is not ArgumentException));

        int attempts = 0;

        var act = () => executor.ExecuteAsync<int>(() =>
        {
            attempts++;
            throw new ArgumentException("not retryable");
        });

        await act.Should().ThrowAsync<ArgumentException>();
        attempts.Should().Be(1);
    }

    [Theory]
    [InlineData(0, 1000)]
    [InlineData(1, 2000)]
    [InlineData(2, 4000)]
    public void CalculateDelay_ExponentialBackoff_WithoutJitter(int attempt, double expectedBaseDelay)
    {
        var executor = new RetryExecutor(new RetryConfig(
            InitialIntervalMs: 1000,
            Multiplier: 2.0,
            JitterFactor: 0.0,
            MaxIntervalMs: 60000));

        var delay = executor.CalculateDelay(attempt);

        delay.Should().Be(expectedBaseDelay);
    }

    [Fact]
    public void CalculateDelay_RespectsMaxInterval()
    {
        var executor = new RetryExecutor(new RetryConfig(
            InitialIntervalMs: 1000,
            Multiplier: 2.0,
            JitterFactor: 0.0,
            MaxIntervalMs: 3000));

        var delay = executor.CalculateDelay(10);

        delay.Should().Be(3000);
    }

    [Fact]
    public void CalculateDelay_WithJitter_IsWithinRange()
    {
        var executor = new RetryExecutor(new RetryConfig(
            InitialIntervalMs: 1000,
            Multiplier: 1.0,
            JitterFactor: 0.5,
            MaxIntervalMs: 60000));

        // Run multiple times to verify jitter is applied.
        var delays = Enumerable.Range(0, 100)
            .Select(_ => executor.CalculateDelay(0))
            .ToList();

        delays.Should().AllSatisfy(d => d.Should().BeInRange(500, 1500));
    }
}
