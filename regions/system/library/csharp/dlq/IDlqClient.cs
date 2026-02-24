namespace K1s0.System.Dlq;

public interface IDlqClient : IAsyncDisposable
{
    Task<DlqMessage> SendAsync(DlqSendRequest request, CancellationToken ct = default);

    Task<ListDlqMessagesResponse> ListMessagesAsync(
        string topic, int page = 1, int pageSize = 20, CancellationToken ct = default);

    Task<DlqMessage> GetMessageAsync(Guid messageId, CancellationToken ct = default);

    Task<RetryDlqMessageResponse> RetryMessageAsync(Guid messageId, CancellationToken ct = default);

    Task DeleteMessageAsync(Guid messageId, CancellationToken ct = default);

    Task RetryAllAsync(string topic, CancellationToken ct = default);
}
