using Xunit;

namespace K1s0.System.Kafka.Tests;

public class KafkaConfigBuilderTests
{
    [Fact]
    public void Build_WithBrokers_SetsBootstrapServers()
    {
        var config = new KafkaConfigBuilder()
            .WithBrokers("kafka1:9092", "kafka2:9092")
            .Build();

        Assert.Equal(["kafka1:9092", "kafka2:9092"], config.Brokers);
        Assert.Equal("kafka1:9092,kafka2:9092", config.BootstrapServers);
    }

    [Fact]
    public void Build_WithSecurityProtocol_SetsProtocol()
    {
        var config = new KafkaConfigBuilder()
            .WithBrokers("kafka:9092")
            .WithSecurityProtocol("SASL_SSL")
            .Build();

        Assert.Equal("SASL_SSL", config.SecurityProtocol);
        Assert.True(config.UsesTls);
    }

    [Fact]
    public void Build_WithSasl_SetsSaslConfig()
    {
        var config = new KafkaConfigBuilder()
            .WithBrokers("kafka:9092")
            .WithSasl("PLAIN", "user", "pass")
            .Build();

        Assert.NotNull(config.Sasl);
        Assert.Equal("PLAIN", config.Sasl.Mechanism);
        Assert.Equal("user", config.Sasl.Username);
        Assert.Equal("pass", config.Sasl.Password);
    }

    [Fact]
    public void Build_WithConsumerGroup_SetsGroup()
    {
        var config = new KafkaConfigBuilder()
            .WithBrokers("kafka:9092")
            .WithConsumerGroup("my-group")
            .Build();

        Assert.Equal("my-group", config.ConsumerGroup);
    }

    [Fact]
    public void Build_WithTls_SetsTlsConfig()
    {
        var config = new KafkaConfigBuilder()
            .WithBrokers("kafka:9092")
            .WithSecurityProtocol("SSL")
            .WithTls("/path/to/ca.pem")
            .Build();

        Assert.NotNull(config.Tls);
        Assert.Equal("/path/to/ca.pem", config.Tls.CaCertPath);
        Assert.True(config.UsesTls);
    }

    [Fact]
    public void Build_WithMessageMaxBytes_SetsValue()
    {
        var config = new KafkaConfigBuilder()
            .WithBrokers("kafka:9092")
            .WithMessageMaxBytes(2_000_000)
            .Build();

        Assert.Equal(2_000_000, config.MessageMaxBytes);
    }

    [Fact]
    public void Build_WithTimeouts_SetsValues()
    {
        var config = new KafkaConfigBuilder()
            .WithBrokers("kafka:9092")
            .WithRequestTimeoutMs(60000)
            .WithSessionTimeoutMs(30000)
            .Build();

        Assert.Equal(60000, config.RequestTimeoutMs);
        Assert.Equal(30000, config.SessionTimeoutMs);
    }

    [Fact]
    public void Build_NoBrokers_ThrowsKafkaException()
    {
        var builder = new KafkaConfigBuilder();

        var ex = Assert.Throws<KafkaException>(() => builder.Build());
        Assert.Equal(KafkaException.ErrorCodes.Config, ex.Code);
    }

    [Fact]
    public void Build_PlaintextProtocol_DoesNotUseTls()
    {
        var config = new KafkaConfigBuilder()
            .WithBrokers("kafka:9092")
            .WithSecurityProtocol("PLAINTEXT")
            .Build();

        Assert.False(config.UsesTls);
    }

    [Fact]
    public void Build_DefaultValues_AreSet()
    {
        var config = new KafkaConfigBuilder()
            .WithBrokers("kafka:9092")
            .Build();

        Assert.Equal("PLAINTEXT", config.SecurityProtocol);
        Assert.Null(config.ConsumerGroup);
        Assert.Null(config.Sasl);
        Assert.Null(config.Tls);
        Assert.Null(config.MessageMaxBytes);
        Assert.Null(config.RequestTimeoutMs);
        Assert.Null(config.SessionTimeoutMs);
    }
}
