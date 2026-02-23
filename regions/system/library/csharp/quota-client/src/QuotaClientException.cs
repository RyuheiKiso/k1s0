namespace K1s0.System.QuotaClient;

public class QuotaClientException : Exception
{
    public QuotaClientException(string message)
        : base(message)
    {
    }

    public QuotaClientException(string message, Exception inner)
        : base(message, inner)
    {
    }
}

public class QuotaExceededException : QuotaClientException
{
    public QuotaExceededException(string quotaId, ulong remaining)
        : base($"Quota exceeded: {quotaId}, remaining={remaining}")
    {
        QuotaId = quotaId;
        Remaining = remaining;
    }

    public string QuotaId { get; }

    public ulong Remaining { get; }
}

public class QuotaNotFoundException : QuotaClientException
{
    public QuotaNotFoundException(string quotaId)
        : base($"Quota not found: {quotaId}")
    {
        QuotaId = quotaId;
    }

    public string QuotaId { get; }
}
