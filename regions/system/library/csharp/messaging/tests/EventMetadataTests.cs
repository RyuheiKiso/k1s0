using K1s0.System.Messaging;

namespace K1s0.System.Messaging.Tests;

public class EventMetadataTests
{
    [Fact]
    public void New_CreatesMetadataWithGeneratedId()
    {
        var metadata = EventMetadata.New(
            "user.created",
            "auth-service",
            "trace-123",
            "corr-456");

        Assert.NotNull(metadata.Id);
        Assert.NotEmpty(metadata.Id);
        Assert.Equal("user.created", metadata.EventType);
        Assert.Equal("auth-service", metadata.Source);
        Assert.Equal("trace-123", metadata.TraceId);
        Assert.Equal("corr-456", metadata.CorrelationId);
        Assert.Equal("1.0", metadata.SchemaVersion);
        Assert.True(metadata.Timestamp <= DateTimeOffset.UtcNow);
    }

    [Fact]
    public void New_WithCustomSchemaVersion_SetsVersion()
    {
        var metadata = EventMetadata.New(
            "order.placed",
            "order-service",
            "trace-001",
            "corr-001",
            schemaVersion: "2.0");

        Assert.Equal("2.0", metadata.SchemaVersion);
    }

    [Fact]
    public void New_GeneratesUniqueIds()
    {
        var metadata1 = EventMetadata.New("evt", "src", "t1", "c1");
        var metadata2 = EventMetadata.New("evt", "src", "t2", "c2");

        Assert.NotEqual(metadata1.Id, metadata2.Id);
    }
}
