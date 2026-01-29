using FluentAssertions;

namespace K1s0.Grpc.Client.Tests;

public class GrpcClientFactoryTests
{
    [Fact]
    public void CreateChannel_ValidAddress_ReturnsChannel()
    {
        using var channel = GrpcClientFactory.CreateChannel("https://localhost:5001");

        channel.Should().NotBeNull();
        channel.Target.Should().Be("localhost:5001");
    }

    [Fact]
    public void CreateChannel_EmptyAddress_Throws()
    {
        var act = () => GrpcClientFactory.CreateChannel("");
        act.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void CreateChannel_WithCustomOptions_AppliesOptions()
    {
        bool configured = false;
        using var channel = GrpcClientFactory.CreateChannel("https://localhost:5001", options =>
        {
            configured = true;
            options.MaxReceiveMessageSize = 8 * 1024 * 1024;
        });

        configured.Should().BeTrue();
    }

    [Fact]
    public void DefaultDeadline_Is30Seconds()
    {
        GrpcClientFactory.DefaultDeadline.Should().Be(TimeSpan.FromSeconds(30));
    }

    [Fact]
    public void CreateDefaultCallOptions_SetsDeadline()
    {
        using var channel = GrpcClientFactory.CreateChannel("https://localhost:5001");

        var before = DateTime.UtcNow;
        var options = channel.CreateDefaultCallOptions();
        var after = DateTime.UtcNow;

        options.Deadline.Should().NotBeNull();
        options.Deadline!.Value.Should().BeOnOrAfter(before.Add(GrpcClientFactory.DefaultDeadline).AddSeconds(-1));
    }
}
