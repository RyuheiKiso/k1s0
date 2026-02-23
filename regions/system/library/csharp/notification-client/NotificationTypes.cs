namespace K1s0.System.NotificationClient;

public enum NotificationChannel
{
    Email,
    Sms,
    Push,
    Webhook,
}

public record NotificationRequest(
    string Id,
    NotificationChannel Channel,
    string Recipient,
    string? Subject,
    string Body);

public record NotificationResponse(
    string Id,
    string Status,
    string? MessageId);
