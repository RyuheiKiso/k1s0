using System.Text.RegularExpressions;

namespace K1s0.System.Kafka;

public sealed partial record TopicConfig
{
    public required string TopicName { get; init; }

    public int NumPartitions { get; init; } = 3;

    public short ReplicationFactor { get; init; } = 3;

    public long? RetentionMs { get; init; }

    public bool ValidateName()
    {
        return TopicNameRegex().IsMatch(TopicName);
    }

    [GeneratedRegex(@"^k1s0\.(system|business|service)\.[a-z][a-z0-9-]*\.[a-z][a-z0-9-]*\.v\d+$")]
    private static partial Regex TopicNameRegex();
}
