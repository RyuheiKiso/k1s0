namespace K1s0.System.Dlq;

public sealed record DlqSendRequest(
    string OriginalTopic,
    string ErrorMessage,
    string Payload,
    int MaxRetries = 3);
