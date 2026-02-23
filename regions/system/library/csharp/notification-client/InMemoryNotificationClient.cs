namespace K1s0.System.NotificationClient;

public class InMemoryNotificationClient : INotificationClient
{
    private readonly List<NotificationRequest> _sent = new();

    public IReadOnlyList<NotificationRequest> Sent => _sent.AsReadOnly();

    public Task<NotificationResponse> SendAsync(NotificationRequest request, CancellationToken ct = default)
    {
        _sent.Add(request);
        var response = new NotificationResponse(request.Id, "sent", Guid.NewGuid().ToString());
        return Task.FromResult(response);
    }
}
