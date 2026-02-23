namespace K1s0.System.NotificationClient;

public interface INotificationClient
{
    Task<NotificationResponse> SendAsync(NotificationRequest request, CancellationToken ct = default);
}
