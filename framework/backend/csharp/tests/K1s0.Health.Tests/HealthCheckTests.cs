using FluentAssertions;
using Microsoft.Extensions.Diagnostics.HealthChecks;

namespace K1s0.Health.Tests;

public class HealthCheckTests
{
    [Fact]
    public async Task LivenessCheck_AlwaysReturnsHealthy()
    {
        var check = new LivenessCheck();

        var result = await check.CheckHealthAsync(new HealthCheckContext());

        result.Status.Should().Be(HealthStatus.Healthy);
    }

    [Fact]
    public async Task ReadinessCheck_NoDependencies_ReturnsHealthy()
    {
        var check = new ReadinessCheck();

        var result = await check.CheckHealthAsync(new HealthCheckContext());

        result.Status.Should().Be(HealthStatus.Healthy);
    }

    [Fact]
    public async Task ReadinessCheck_AllReady_ReturnsHealthy()
    {
        var check = new ReadinessCheck();
        check.AddDependencyCheck(_ => Task.FromResult(true));
        check.AddDependencyCheck(_ => Task.FromResult(true));

        var result = await check.CheckHealthAsync(new HealthCheckContext());

        result.Status.Should().Be(HealthStatus.Healthy);
    }

    [Fact]
    public async Task ReadinessCheck_OneNotReady_ReturnsUnhealthy()
    {
        var check = new ReadinessCheck();
        check.AddDependencyCheck(_ => Task.FromResult(true));
        check.AddDependencyCheck(_ => Task.FromResult(false));

        var result = await check.CheckHealthAsync(new HealthCheckContext());

        result.Status.Should().Be(HealthStatus.Unhealthy);
    }

    [Fact]
    public async Task ReadinessCheck_CheckThrows_ReturnsUnhealthy()
    {
        var check = new ReadinessCheck();
        check.AddDependencyCheck(_ => throw new InvalidOperationException("db down"));

        var result = await check.CheckHealthAsync(new HealthCheckContext());

        result.Status.Should().Be(HealthStatus.Unhealthy);
        result.Exception.Should().BeOfType<InvalidOperationException>();
    }

    [Fact]
    public void ReadinessCheck_NullCheck_Throws()
    {
        var check = new ReadinessCheck();
        var act = () => check.AddDependencyCheck(null!);
        act.Should().Throw<ArgumentNullException>();
    }
}
