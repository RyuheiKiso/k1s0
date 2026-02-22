namespace K1s0.System.Dlq;

public sealed record ListDlqMessagesResponse(
    IReadOnlyList<DlqMessage> Messages,
    int Total,
    int Page,
    int PageSize);
