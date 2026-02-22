using Xunit;

namespace K1s0.System.Kafka.Tests;

public class TopicConfigTests
{
    [Theory]
    [InlineData("k1s0.system.auth.user-created.v1", true)]
    [InlineData("k1s0.business.orders.order-placed.v1", true)]
    [InlineData("k1s0.service.payments.payment-processed.v2", true)]
    [InlineData("invalid-topic-name", false)]
    [InlineData("k1s0.wrong.auth.user-created.v1", false)]
    [InlineData("k1s0.system.Auth.user-created.v1", false)]
    [InlineData("k1s0.system.auth.user-created", false)]
    [InlineData("", false)]
    public void ValidateName_ReturnsExpected(string topicName, bool expected)
    {
        var config = new TopicConfig { TopicName = topicName };
        Assert.Equal(expected, config.ValidateName());
    }

    [Fact]
    public void TopicConfig_DefaultValues_AreCorrect()
    {
        var config = new TopicConfig { TopicName = "k1s0.system.auth.user-created.v1" };

        Assert.Equal(3, config.NumPartitions);
        Assert.Equal((short)3, config.ReplicationFactor);
        Assert.Null(config.RetentionMs);
    }

    [Fact]
    public void TopicConfig_CustomValues_AreSet()
    {
        var config = new TopicConfig
        {
            TopicName = "k1s0.system.auth.user-created.v1",
            NumPartitions = 6,
            ReplicationFactor = 2,
            RetentionMs = 604_800_000,
        };

        Assert.Equal(6, config.NumPartitions);
        Assert.Equal((short)2, config.ReplicationFactor);
        Assert.Equal(604_800_000L, config.RetentionMs);
    }
}
