namespace K1s0.System.Dlq;

public sealed record RetryDlqMessageResponse(
    Guid MessageId,
    DlqStatus Status,
    string Message);
