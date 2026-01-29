using FluentAssertions;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;

namespace K1s0.Observability.Tests;

public class ObservabilityExtensionsTests
{
    [Fact]
    public void AddK1s0Observability_RegistersK1s0Metrics()
    {
        var config = new ConfigurationBuilder()
            .AddInMemoryCollection(new Dictionary<string, string?>
            {
                ["app:name"] = "test-service",
            })
            .Build();

        var services = new ServiceCollection();
        services.AddK1s0Observability(config);

        var descriptors = services.Where(s => s.ServiceType == typeof(K1s0Metrics)).ToList();

        descriptors.Should().ContainSingle();
        descriptors[0].Lifetime.Should().Be(ServiceLifetime.Singleton);
    }

    [Fact]
    public void AddK1s0Observability_NullServices_Throws()
    {
        var config = new ConfigurationBuilder().Build();
        IServiceCollection services = null!;

        var act = () => services.AddK1s0Observability(config);
        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void AddK1s0Observability_NullConfig_Throws()
    {
        var services = new ServiceCollection();

        var act = () => services.AddK1s0Observability(null!);
        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void K1s0ActivitySource_HasCorrectName()
    {
        K1s0ActivitySource.Name.Should().Be("k1s0");
        K1s0ActivitySource.Instance.Should().NotBeNull();
        K1s0ActivitySource.Instance.Name.Should().Be("k1s0");
    }
}
