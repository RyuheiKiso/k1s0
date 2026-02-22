namespace K1s0.System.Kafka;

public sealed record TopicPartitionInfo
{
    public required string Topic { get; init; }

    public required int Partition { get; init; }

    public required int Leader { get; init; }

    public required int[] Replicas { get; init; }

    public required int[] Isr { get; init; }
}
