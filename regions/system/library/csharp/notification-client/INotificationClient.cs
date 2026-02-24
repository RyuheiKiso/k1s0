namespace K1s0.System.NotificationClient;

public interface INotificationClient
{
    Task<NotificationResponse> SendAsync(NotificationRequest request, CancellationToken ct = default);

    Task<IReadOnlyList<NotificationResponse>> SendBatchAsync(
        IReadOnlyList<NotificationRequest> requests, CancellationToken ct = default);
}
