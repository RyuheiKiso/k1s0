using System.Text.Json;
using FluentAssertions;

namespace K1s0.DomainEvent.Tests;

public sealed class EventEnvelopeTests
{
    [Fact]
    public void Wrap_SetsEventType()
    {
        var domainEvent = new TestEvent("order-123", "Order");

        var envelope = EventEnvelope.Wrap(domainEvent, "order-service");

        envelope.EventType.Should().Be("test.created");
    }

    [Fact]
    public void Wrap_GeneratesMetadataWithEventIdAndTimestamp()
    {
        var before = DateTimeOffset.UtcNow;
        var domainEvent = new TestEvent("agg-1", "TestAggregate");

        var envelope = EventEnvelope.Wrap(domainEvent, "my-service");

        envelope.Metadata.EventId.Should().NotBeEmpty();
        envelope.Metadata.OccurredAt.Should().BeOnOrAfter(before);
        envelope.Metadata.Source.Should().Be("my-service");
        envelope.Metadata.CorrelationId.Should().BeNull();
        envelope.Metadata.CausationId.Should().BeNull();
    }

    [Fact]
    public void Wrap_SetsCorrelationAndCausationIds()
    {
        var domainEvent = new TestEvent("agg-1", "Agg");

        var envelope = EventEnvelope.Wrap(domainEvent, "svc", "corr-1", "cause-1");

        envelope.Metadata.CorrelationId.Should().Be("corr-1");
        envelope.Metadata.CausationId.Should().Be("cause-1");
    }

    [Fact]
    public void Wrap_SerializesPayloadAsJson()
    {
        var domainEvent = new TestEvent("order-42", "Order");

        var envelope = EventEnvelope.Wrap(domainEvent, "svc");

        var deserialized = JsonSerializer.Deserialize<TestEvent>(envelope.Payload);
        deserialized.Should().NotBeNull();
        deserialized!.AggregateId.Should().Be("order-42");
    }

    [Fact]
    public void Wrap_ThrowsOnNullEvent()
    {
        var act = () => EventEnvelope.Wrap(null!, "svc");

        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void Wrap_ThrowsOnEmptySource()
    {
        var act = () => EventEnvelope.Wrap(new TestEvent("1", "T"), "");

        act.Should().Throw<ArgumentException>();
    }

    private sealed record TestEvent(string AggregateId, string AggregateType) : IDomainEvent
    {
        public string EventType => "test.created";
    }
}
