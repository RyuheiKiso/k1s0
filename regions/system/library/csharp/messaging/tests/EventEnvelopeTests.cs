using K1s0.System.Messaging;
using Xunit;

namespace K1s0.System.Messaging.Tests;

public class EventEnvelopeTests
{
    [Fact]
    public void Constructor_SetsAllProperties()
    {
        var metadata = EventMetadata.New("test.event", "test-source", "trace-1", "corr-1");
        var payload = new byte[] { 0x7B, 0x7D };
        var headers = new Dictionary<string, string> { ["x-custom"] = "header-value" };

        var envelope = new EventEnvelope(
            Topic: "test-topic",
            Key: "test-key",
            Payload: payload,
            Metadata: metadata,
            Headers: headers);

        Assert.Equal("test-topic", envelope.Topic);
        Assert.Equal("test-key", envelope.Key);
        Assert.Equal(payload, envelope.Payload);
        Assert.Equal(metadata, envelope.Metadata);
        Assert.Equal("header-value", envelope.Headers!["x-custom"]);
    }

    [Fact]
    public void Constructor_WithNullKey_Allowed()
    {
        var metadata = EventMetadata.New("test.event", "test-source", "trace-1", "corr-1");
        var payload = new byte[] { 0x64, 0x61, 0x74, 0x61 };

        var envelope = new EventEnvelope(
            Topic: "topic",
            Key: null,
            Payload: payload,
            Metadata: metadata);

        Assert.Null(envelope.Key);
    }

    [Fact]
    public void Constructor_WithNullHeaders_DefaultsToNull()
    {
        var metadata = EventMetadata.New("test.event", "test-source", "trace-1", "corr-1");
        var payload = Array.Empty<byte>();

        var envelope = new EventEnvelope(
            Topic: "topic",
            Key: "key",
            Payload: payload,
            Metadata: metadata);

        Assert.Null(envelope.Headers);
    }
}
