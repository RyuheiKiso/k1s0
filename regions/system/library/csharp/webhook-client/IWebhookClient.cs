namespace K1s0.System.WebhookClient;

public interface IWebhookClient
{
    Task<int> SendAsync(string url, WebhookPayload payload, CancellationToken ct = default);
}
