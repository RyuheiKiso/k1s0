using FluentAssertions;
using Xunit;

namespace K1s0.Resilience.Tests;

public class CircuitBreakerTests
{
    [Fact]
    public async Task InitialState_IsClosed()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig());

        cb.State.Should().Be(CircuitState.Closed);
    }

    [Fact]
    public async Task ClosedToOpen_AfterFailureThreshold()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(FailureThreshold: 3));

        for (int i = 0; i < 3; i++)
        {
            try
            {
                await cb.ExecuteAsync<int>(() => throw new InvalidOperationException("fail"));
            }
            catch (InvalidOperationException)
            {
                // Expected.
            }
        }

        cb.State.Should().Be(CircuitState.Open);
    }

    [Fact]
    public async Task Open_RejectsRequests()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(FailureThreshold: 1, ResetTimeoutSeconds: 60.0));

        try
        {
            await cb.ExecuteAsync<int>(() => throw new InvalidOperationException("fail"));
        }
        catch (InvalidOperationException)
        {
            // Expected.
        }

        cb.State.Should().Be(CircuitState.Open);

        var act = () => cb.ExecuteAsync(() => Task.FromResult(1));
        await act.Should().ThrowAsync<CircuitOpenException>();
        cb.RejectedCount.Should().Be(1);
    }

    [Fact]
    public async Task OpenToHalfOpen_AfterResetTimeout()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(FailureThreshold: 1, ResetTimeoutSeconds: 0.1, SuccessThreshold: 1));

        try
        {
            await cb.ExecuteAsync<int>(() => throw new InvalidOperationException("fail"));
        }
        catch (InvalidOperationException)
        {
            // Expected.
        }

        cb.State.Should().Be(CircuitState.Open);

        await Task.Delay(TimeSpan.FromSeconds(0.2));

        var result = await cb.ExecuteAsync(() => Task.FromResult(42));

        result.Should().Be(42);
        cb.State.Should().Be(CircuitState.Closed);
    }

    [Fact]
    public async Task HalfOpenToOpen_OnFailure()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(FailureThreshold: 1, ResetTimeoutSeconds: 0.1, SuccessThreshold: 2));

        try
        {
            await cb.ExecuteAsync<int>(() => throw new InvalidOperationException("fail"));
        }
        catch (InvalidOperationException)
        {
            // Expected.
        }

        await Task.Delay(TimeSpan.FromSeconds(0.2));

        try
        {
            await cb.ExecuteAsync<int>(() => throw new InvalidOperationException("fail again"));
        }
        catch (InvalidOperationException)
        {
            // Expected.
        }

        cb.State.Should().Be(CircuitState.Open);
    }

    [Fact]
    public async Task HalfOpenToClosed_AfterSuccessThreshold()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(FailureThreshold: 1, ResetTimeoutSeconds: 0.1, SuccessThreshold: 2));

        try
        {
            await cb.ExecuteAsync<int>(() => throw new InvalidOperationException("fail"));
        }
        catch (InvalidOperationException)
        {
            // Expected.
        }

        await Task.Delay(TimeSpan.FromSeconds(0.2));

        // First success in half-open.
        await cb.ExecuteAsync(() => Task.FromResult(1));
        cb.State.Should().Be(CircuitState.HalfOpen);

        // Second success closes the circuit.
        await cb.ExecuteAsync(() => Task.FromResult(2));
        cb.State.Should().Be(CircuitState.Closed);
    }

    [Fact]
    public async Task StateTransitionCount_Increments()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(FailureThreshold: 1, ResetTimeoutSeconds: 0.1, SuccessThreshold: 1));

        try
        {
            await cb.ExecuteAsync<int>(() => throw new InvalidOperationException("fail"));
        }
        catch (InvalidOperationException)
        {
            // Expected.
        }

        // Closed -> Open = 1 transition.
        cb.StateTransitionCount.Should().Be(1);
    }

    [Fact]
    public async Task FailurePredicate_IgnoresNonMatchingExceptions()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(
            FailureThreshold: 1,
            FailurePredicate: ex => ex is InvalidOperationException));

        try
        {
            await cb.ExecuteAsync<int>(() => throw new ArgumentException("not counted"));
        }
        catch (ArgumentException)
        {
            // Expected.
        }

        cb.State.Should().Be(CircuitState.Closed);
    }
}
