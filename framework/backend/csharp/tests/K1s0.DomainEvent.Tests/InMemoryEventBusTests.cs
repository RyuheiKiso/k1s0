using FluentAssertions;

namespace K1s0.DomainEvent.Tests;

public sealed class InMemoryEventBusTests
{
    private readonly InMemoryEventBus _bus = new();

    [Fact]
    public async Task PublishAsync_DeliversToSubscribedHandler()
    {
        var handler = new RecordingHandler("order.created");
        await _bus.SubscribeAsync(handler);

        var envelope = MakeEnvelope("order.created");
        await _bus.PublishAsync(envelope);

        handler.Received.Should().ContainSingle().Which.Should().Be(envelope);
    }

    [Fact]
    public async Task PublishAsync_DoesNotDeliverToUnrelatedHandler()
    {
        var handler = new RecordingHandler("user.created");
        await _bus.SubscribeAsync(handler);

        await _bus.PublishAsync(MakeEnvelope("order.created"));

        handler.Received.Should().BeEmpty();
    }

    [Fact]
    public async Task PublishAsync_DeliversToMultipleHandlers()
    {
        var handler1 = new RecordingHandler("order.created");
        var handler2 = new RecordingHandler("order.created");
        await _bus.SubscribeAsync(handler1);
        await _bus.SubscribeAsync(handler2);

        await _bus.PublishAsync(MakeEnvelope("order.created"));

        handler1.Received.Should().HaveCount(1);
        handler2.Received.Should().HaveCount(1);
    }

    [Fact]
    public async Task SubscribeAsync_DisposingRemovesHandler()
    {
        var handler = new RecordingHandler("order.created");
        var subscription = await _bus.SubscribeAsync(handler);

        subscription.Dispose();
        await _bus.PublishAsync(MakeEnvelope("order.created"));

        handler.Received.Should().BeEmpty();
    }

    [Fact]
    public async Task SubscribeAsync_DoubleDisposeIsHarmless()
    {
        var handler = new RecordingHandler("order.created");
        var subscription = await _bus.SubscribeAsync(handler);

        subscription.Dispose();
        subscription.Dispose(); // should not throw
    }

    [Fact]
    public async Task PublishBatchAsync_DeliversAllEnvelopes()
    {
        var handler = new RecordingHandler("order.created");
        await _bus.SubscribeAsync(handler);

        var envelopes = new[] { MakeEnvelope("order.created"), MakeEnvelope("order.created") };
        await _bus.PublishBatchAsync(envelopes);

        handler.Received.Should().HaveCount(2);
    }

    [Fact]
    public async Task PublishAsync_NoSubscribers_DoesNotThrow()
    {
        var act = async () => await _bus.PublishAsync(MakeEnvelope("unknown.event"));

        await act.Should().NotThrowAsync();
    }

    private static EventEnvelope MakeEnvelope(string eventType)
    {
        var metadata = new EventMetadata(Guid.NewGuid(), DateTimeOffset.UtcNow, "test-source");
        return new EventEnvelope(eventType, metadata, "{}");
    }

    private sealed class RecordingHandler(string eventType) : IEventHandler
    {
        public string EventType => eventType;
        public List<EventEnvelope> Received { get; } = [];

        public Task HandleAsync(EventEnvelope envelope, CancellationToken cancellationToken = default)
        {
            Received.Add(envelope);
            return Task.CompletedTask;
        }
    }
}
