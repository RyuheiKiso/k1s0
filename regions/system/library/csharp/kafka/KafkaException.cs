namespace K1s0.System.Kafka;

public class KafkaException : Exception
{
    public string Code { get; }

    public KafkaException(string code, string message)
        : base(message)
    {
        Code = code;
    }

    public KafkaException(string code, string message, Exception innerException)
        : base(message, innerException)
    {
        Code = code;
    }

    public static class ErrorCodes
    {
        public const string ConnectionFailed = "CONNECTION_FAILED";
        public const string TopicNotFound = "TOPIC_NOT_FOUND";
        public const string Partition = "PARTITION_ERROR";
        public const string Config = "CONFIG_ERROR";
        public const string Timeout = "TIMEOUT";
    }
}
