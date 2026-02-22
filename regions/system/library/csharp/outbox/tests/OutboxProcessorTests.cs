using NSubstitute;
using NSubstitute.ExceptionExtensions;

namespace K1s0.System.Outbox.Tests;

public class OutboxProcessorTests
{
    private readonly IOutboxStore _store = Substitute.For<IOutboxStore>();
    private readonly IEventProducer _producer = Substitute.For<IEventProducer>();
    private readonly OutboxConfig _config = new(
        ConnectionString: "Host=localhost",
        PollingInterval: TimeSpan.FromMilliseconds(50),
        MaxRetries: 3,
        BackoffBase: TimeSpan.FromSeconds(1));

    [Fact]
    public async Task ExecuteAsync_PublishesPendingMessages()
    {
        var message = CreateMessage();
        _store.FetchPendingAsync(Arg.Any<int>(), Arg.Any<CancellationToken>())
            .Returns(new List<OutboxMessage> { message });

        using var cts = new CancellationTokenSource(TimeSpan.FromMilliseconds(200));
        var processor = new OutboxProcessor(_config, _store, _producer);

        await RunProcessorAsync(processor, cts.Token);

        await _producer.Received().PublishAsync(message, Arg.Any<CancellationToken>());
        await _store.Received().MarkPublishedAsync(message.Id, Arg.Any<CancellationToken>());
    }

    [Fact]
    public async Task ExecuteAsync_MarksFailedOnPublishError()
    {
        var message = CreateMessage();
        _store.FetchPendingAsync(Arg.Any<int>(), Arg.Any<CancellationToken>())
            .Returns(new List<OutboxMessage> { message });
        _producer.PublishAsync(Arg.Any<OutboxMessage>(), Arg.Any<CancellationToken>())
            .ThrowsAsync(new InvalidOperationException("Publish failed"));

        using var cts = new CancellationTokenSource(TimeSpan.FromMilliseconds(200));
        var processor = new OutboxProcessor(_config, _store, _producer);

        await RunProcessorAsync(processor, cts.Token);

        await _store.Received().MarkFailedAsync(message.Id, "Publish failed", Arg.Any<CancellationToken>());
    }

    [Fact]
    public async Task ExecuteAsync_SkipsMessagesExceedingMaxRetries()
    {
        var message = CreateMessage(retryCount: 3);
        _store.FetchPendingAsync(Arg.Any<int>(), Arg.Any<CancellationToken>())
            .Returns(new List<OutboxMessage> { message });

        using var cts = new CancellationTokenSource(TimeSpan.FromMilliseconds(200));
        var processor = new OutboxProcessor(_config, _store, _producer);

        await RunProcessorAsync(processor, cts.Token);

        await _producer.DidNotReceive().PublishAsync(Arg.Any<OutboxMessage>(), Arg.Any<CancellationToken>());
    }

    [Theory]
    [InlineData(0, 1.0)]
    [InlineData(1, 2.0)]
    [InlineData(2, 4.0)]
    [InlineData(3, 8.0)]
    public void CalculateBackoff_ReturnsExponentialDelay(int retryCount, double expectedSeconds)
    {
        var backoff = OutboxProcessor.CalculateBackoff(retryCount, TimeSpan.FromSeconds(1));
        Assert.Equal(TimeSpan.FromSeconds(expectedSeconds), backoff);
    }

    [Fact]
    public async Task ExecuteAsync_ContinuesOnStoreFailure()
    {
        _store.FetchPendingAsync(Arg.Any<int>(), Arg.Any<CancellationToken>())
            .ThrowsAsync(new OutboxException(OutboxErrorCodes.Fetch, "DB error"));

        using var cts = new CancellationTokenSource(TimeSpan.FromMilliseconds(200));
        var processor = new OutboxProcessor(_config, _store, _producer);

        await RunProcessorAsync(processor, cts.Token);

        // Should not throw; processor continues polling
        await _producer.DidNotReceive().PublishAsync(Arg.Any<OutboxMessage>(), Arg.Any<CancellationToken>());
    }

    private static OutboxMessage CreateMessage(int retryCount = 0) =>
        new(
            Id: Guid.NewGuid(),
            Topic: "test.topic",
            Payload: [0x01, 0x02],
            Status: OutboxStatus.Pending,
            RetryCount: retryCount,
            CreatedAt: DateTimeOffset.UtcNow,
            UpdatedAt: DateTimeOffset.UtcNow,
            LastError: null);

    private static async Task RunProcessorAsync(OutboxProcessor processor, CancellationToken ct)
    {
        await processor.StartAsync(ct);
        try
        {
            await Task.Delay(TimeSpan.FromMilliseconds(300), ct);
        }
        catch (OperationCanceledException)
        {
            // Expected
        }

        await processor.StopAsync(CancellationToken.None);
    }
}
