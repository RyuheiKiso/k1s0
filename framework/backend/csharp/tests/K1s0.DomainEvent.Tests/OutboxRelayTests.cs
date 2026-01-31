using System.Text.Json;
using FluentAssertions;
using K1s0.DomainEvent.Outbox;
using Moq;

namespace K1s0.DomainEvent.Tests;

public sealed class OutboxRelayTests
{
    private readonly Mock<IOutboxStore> _storeMock = new();
    private readonly Mock<IEventPublisher> _publisherMock = new();

    [Fact]
    public async Task RelayPendingAsync_PublishesAndMarksAsPublished()
    {
        var envelope = MakeEnvelope("order.created");
        var entry = MakeEntry(envelope);

        _storeMock.Setup(s => s.FetchPendingAsync(It.IsAny<int>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync(new List<OutboxEntry> { entry });

        var relay = new OutboxRelay(_storeMock.Object, _publisherMock.Object);

        await relay.RelayPendingAsync();

        _publisherMock.Verify(p => p.PublishAsync(
            It.Is<EventEnvelope>(e => e.EventType == "order.created"),
            It.IsAny<CancellationToken>()), Times.Once);
        _storeMock.Verify(s => s.MarkPublishedAsync(entry.Id, It.IsAny<CancellationToken>()), Times.Once);
    }

    [Fact]
    public async Task RelayPendingAsync_MarksAsFailedOnPublishError()
    {
        var envelope = MakeEnvelope("order.created");
        var entry = MakeEntry(envelope);

        _storeMock.Setup(s => s.FetchPendingAsync(It.IsAny<int>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync(new List<OutboxEntry> { entry });
        _publisherMock.Setup(p => p.PublishAsync(It.IsAny<EventEnvelope>(), It.IsAny<CancellationToken>()))
            .ThrowsAsync(new InvalidOperationException("publish failed"));

        var relay = new OutboxRelay(_storeMock.Object, _publisherMock.Object);

        await relay.RelayPendingAsync();

        _storeMock.Verify(s => s.MarkFailedAsync(entry.Id, It.IsAny<CancellationToken>()), Times.Once);
        _storeMock.Verify(s => s.MarkPublishedAsync(It.IsAny<Guid>(), It.IsAny<CancellationToken>()), Times.Never);
    }

    [Fact]
    public async Task RelayPendingAsync_NoPendingEntries_DoesNothing()
    {
        _storeMock.Setup(s => s.FetchPendingAsync(It.IsAny<int>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync(new List<OutboxEntry>());

        var relay = new OutboxRelay(_storeMock.Object, _publisherMock.Object);

        await relay.RelayPendingAsync();

        _publisherMock.Verify(p => p.PublishAsync(It.IsAny<EventEnvelope>(), It.IsAny<CancellationToken>()), Times.Never);
    }

    [Fact]
    public async Task RunAsync_StopsOnCancellation()
    {
        _storeMock.Setup(s => s.FetchPendingAsync(It.IsAny<int>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync(new List<OutboxEntry>());

        var relay = new OutboxRelay(_storeMock.Object, _publisherMock.Object)
        {
            PollInterval = TimeSpan.FromMilliseconds(10),
        };

        using var cts = new CancellationTokenSource(TimeSpan.FromMilliseconds(50));

        await relay.RunAsync(cts.Token);

        _storeMock.Verify(s => s.FetchPendingAsync(It.IsAny<int>(), It.IsAny<CancellationToken>()), Times.AtLeastOnce);
    }

    private static EventEnvelope MakeEnvelope(string eventType)
    {
        var metadata = new EventMetadata(Guid.NewGuid(), DateTimeOffset.UtcNow, "test-source");
        return new EventEnvelope(eventType, metadata, "{}");
    }

    private static OutboxEntry MakeEntry(EventEnvelope envelope)
    {
        var payload = JsonSerializer.Serialize(envelope);
        return new OutboxEntry(
            Guid.NewGuid(),
            envelope.EventType,
            payload,
            OutboxStatus.Pending,
            RetryCount: 0,
            DateTimeOffset.UtcNow,
            DateTimeOffset.UtcNow);
    }
}
