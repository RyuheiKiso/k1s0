using System.Text;
using Confluent.Kafka;
using K1s0.System.Messaging;
using NSubstitute;

namespace K1s0.System.Messaging.Tests;

public class KafkaEventProducerTests
{
    private readonly IProducer<string, byte[]> _mockProducer;
    private readonly KafkaEventProducer _producer;

    public KafkaEventProducerTests()
    {
        _mockProducer = Substitute.For<IProducer<string, byte[]>>();
        _producer = new KafkaEventProducer(_mockProducer);
    }

    [Fact]
    public async Task PublishAsync_CallsProduceAsync()
    {
        var metadata = EventMetadata.New("user.created", "auth-service", "trace-1", "corr-1");
        var payload = Encoding.UTF8.GetBytes("{\"user_id\":\"123\"}");
        var envelope = new EventEnvelope("user-events", "user-123", payload, metadata);

        _mockProducer.ProduceAsync(
                Arg.Any<string>(),
                Arg.Any<Message<string, byte[]>>(),
                Arg.Any<CancellationToken>())
            .Returns(Task.FromResult(new DeliveryResult<string, byte[]>()));

        await _producer.PublishAsync(envelope);

        await _mockProducer.Received(1).ProduceAsync(
            "user-events",
            Arg.Is<Message<string, byte[]>>(m => m.Key == "user-123"),
            Arg.Any<CancellationToken>());
    }

    [Fact]
    public async Task PublishAsync_WithHeaders_IncludesHeaders()
    {
        var metadata = EventMetadata.New("order.placed", "order-service", "trace-2", "corr-2");
        var payload = Encoding.UTF8.GetBytes("{}");
        var headers = new Dictionary<string, string> { ["x-tenant"] = "tenant-a" };
        var envelope = new EventEnvelope("orders", "order-1", payload, metadata, headers);

        _mockProducer.ProduceAsync(
                Arg.Any<string>(),
                Arg.Any<Message<string, byte[]>>(),
                Arg.Any<CancellationToken>())
            .Returns(Task.FromResult(new DeliveryResult<string, byte[]>()));

        await _producer.PublishAsync(envelope);

        await _mockProducer.Received(1).ProduceAsync(
            "orders",
            Arg.Is<Message<string, byte[]>>(m => m.Headers != null && m.Headers.Count == 1),
            Arg.Any<CancellationToken>());
    }

    [Fact]
    public async Task PublishBatchAsync_PublishesAllEnvelopes()
    {
        var metadata = EventMetadata.New("batch.event", "batch-source", "trace-3", "corr-3");
        var payload = Encoding.UTF8.GetBytes("data");

        var envelopes = new List<EventEnvelope>
        {
            new("topic-a", "key-1", payload, metadata),
            new("topic-a", "key-2", payload, metadata),
            new("topic-a", "key-3", payload, metadata),
        };

        _mockProducer.ProduceAsync(
                Arg.Any<string>(),
                Arg.Any<Message<string, byte[]>>(),
                Arg.Any<CancellationToken>())
            .Returns(Task.FromResult(new DeliveryResult<string, byte[]>()));

        await _producer.PublishBatchAsync(envelopes);

        await _mockProducer.Received(3).ProduceAsync(
            Arg.Any<string>(),
            Arg.Any<Message<string, byte[]>>(),
            Arg.Any<CancellationToken>());
    }

    [Fact]
    public async Task PublishAsync_ProduceException_ThrowsMessagingException()
    {
        var metadata = EventMetadata.New("fail.event", "fail-source", "trace-4", "corr-4");
        var envelope = new EventEnvelope("topic", "key", Array.Empty<byte>(), metadata);

        _mockProducer.ProduceAsync(
                Arg.Any<string>(),
                Arg.Any<Message<string, byte[]>>(),
                Arg.Any<CancellationToken>())
            .Returns<DeliveryResult<string, byte[]>>(x =>
                throw new ProduceException<string, byte[]>(
                    new Error(ErrorCode.Local_MsgTimedOut, "Timed out"),
                    new DeliveryResult<string, byte[]>()));

        var ex = await Assert.ThrowsAsync<MessagingException>(() => _producer.PublishAsync(envelope));
        Assert.Equal("Publish", ex.Code);
    }

    [Fact]
    public async Task DisposeAsync_FlushesAndDisposesProducer()
    {
        await _producer.DisposeAsync();

        _mockProducer.Received(1).Flush(Arg.Any<TimeSpan>());
        _mockProducer.Received(1).Dispose();
    }
}
