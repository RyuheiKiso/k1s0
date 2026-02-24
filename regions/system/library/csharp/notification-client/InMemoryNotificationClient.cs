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

    public Task<IReadOnlyList<NotificationResponse>> SendBatchAsync(
        IReadOnlyList<NotificationRequest> requests, CancellationToken ct = default)
    {
        var responses = new List<NotificationResponse>(requests.Count);
        foreach (var request in requests)
        {
            _sent.Add(request);
            responses.Add(new NotificationResponse(request.Id, "sent", Guid.NewGuid().ToString()));
        }

        return Task.FromResult<IReadOnlyList<NotificationResponse>>(responses.AsReadOnly());
    }
}
