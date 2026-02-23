namespace K1s0.System.WebhookClient;

public record WebhookPayload(
    string EventType,
    string Timestamp,
    Dictionary<string, object> Data);
