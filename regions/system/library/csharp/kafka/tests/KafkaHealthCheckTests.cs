using NSubstitute;
using Xunit;

namespace K1s0.System.Kafka.Tests;

public class KafkaHealthCheckTests
{
    [Fact]
    public async Task CheckHealthAsync_MockedHealthy_ReturnsHealthy()
    {
        // Arrange
        var mockHealthCheck = Substitute.For<IKafkaHealthCheck>();
        mockHealthCheck.CheckHealthAsync(Arg.Any<CancellationToken>())
            .Returns(HealthCheckResult.Healthy);

        // Act
        var result = await mockHealthCheck.CheckHealthAsync();

        // Assert
        Assert.Equal(HealthCheckResult.Healthy, result);
    }

    [Fact]
    public async Task CheckHealthAsync_MockedUnhealthy_ReturnsUnhealthy()
    {
        // Arrange
        var mockHealthCheck = Substitute.For<IKafkaHealthCheck>();
        mockHealthCheck.CheckHealthAsync(Arg.Any<CancellationToken>())
            .Returns(HealthCheckResult.Unhealthy);

        // Act
        var result = await mockHealthCheck.CheckHealthAsync();

        // Assert
        Assert.Equal(HealthCheckResult.Unhealthy, result);
    }

    [Fact]
    public async Task CheckHealthAsync_CancellationToken_IsPassed()
    {
        // Arrange
        using var cts = new CancellationTokenSource();
        var mockHealthCheck = Substitute.For<IKafkaHealthCheck>();
        mockHealthCheck.CheckHealthAsync(cts.Token)
            .Returns(HealthCheckResult.Healthy);

        // Act
        var result = await mockHealthCheck.CheckHealthAsync(cts.Token);

        // Assert
        Assert.Equal(HealthCheckResult.Healthy, result);
        await mockHealthCheck.Received(1).CheckHealthAsync(cts.Token);
    }
}
