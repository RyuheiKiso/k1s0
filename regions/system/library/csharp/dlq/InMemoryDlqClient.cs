namespace K1s0.System.Dlq;

public sealed class InMemoryDlqClient : IDlqClient
{
    private readonly Dictionary<Guid, DlqMessage> _messages = new();

    public IReadOnlyDictionary<Guid, DlqMessage> Messages => _messages;

    public Task<DlqMessage> SendAsync(DlqSendRequest request, CancellationToken ct = default)
    {
        var id = Guid.NewGuid();
        var now = DateTimeOffset.UtcNow;
        var message = new DlqMessage(
            id,
            request.OriginalTopic,
            request.ErrorMessage,
            0,
            request.MaxRetries,
            request.Payload,
            DlqStatus.Pending,
            now,
            now);
        _messages[id] = message;
        return Task.FromResult(message);
    }

    public Task<ListDlqMessagesResponse> ListMessagesAsync(
        string topic, int page = 1, int pageSize = 20, CancellationToken ct = default)
    {
        var filtered = _messages.Values
            .Where(m => m.OriginalTopic == topic)
            .OrderByDescending(m => m.CreatedAt)
            .ToList();

        var total = filtered.Count;
        var items = filtered.Skip((page - 1) * pageSize).Take(pageSize).ToList();

        return Task.FromResult(new ListDlqMessagesResponse(items, total, page, pageSize));
    }

    public Task<DlqMessage> GetMessageAsync(Guid messageId, CancellationToken ct = default)
    {
        if (!_messages.TryGetValue(messageId, out var message))
        {
            throw new DlqException(DlqErrorCodes.NotFound, $"Message not found: {messageId}");
        }

        return Task.FromResult(message);
    }

    public Task<RetryDlqMessageResponse> RetryMessageAsync(Guid messageId, CancellationToken ct = default)
    {
        if (!_messages.TryGetValue(messageId, out var message))
        {
            throw new DlqException(DlqErrorCodes.NotFound, $"Message not found: {messageId}");
        }

        _messages[messageId] = message with
        {
            Status = DlqStatus.Retrying,
            RetryCount = message.RetryCount + 1,
            UpdatedAt = DateTimeOffset.UtcNow,
        };

        return Task.FromResult(new RetryDlqMessageResponse(messageId, DlqStatus.Retrying, "Retry scheduled"));
    }

    public Task DeleteMessageAsync(Guid messageId, CancellationToken ct = default)
    {
        if (!_messages.Remove(messageId))
        {
            throw new DlqException(DlqErrorCodes.NotFound, $"Message not found: {messageId}");
        }

        return Task.CompletedTask;
    }

    public Task RetryAllAsync(string topic, CancellationToken ct = default)
    {
        var toRetry = _messages
            .Where(kv => kv.Value.OriginalTopic == topic && kv.Value.Status == DlqStatus.Pending)
            .Select(kv => kv.Key)
            .ToList();

        foreach (var id in toRetry)
        {
            var msg = _messages[id];
            _messages[id] = msg with
            {
                Status = DlqStatus.Retrying,
                RetryCount = msg.RetryCount + 1,
                UpdatedAt = DateTimeOffset.UtcNow,
            };
        }

        return Task.CompletedTask;
    }

    public ValueTask DisposeAsync() => ValueTask.CompletedTask;
}
