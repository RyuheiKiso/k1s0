namespace K1s0.System.RateLimitClient;

public class RateLimitException : Exception
{
    public string Code { get; }

    public ulong? RetryAfterSecs { get; }

    public RateLimitException(string message, string code, ulong? retryAfterSecs = null)
        : base(message)
    {
        Code = code;
        RetryAfterSecs = retryAfterSecs;
    }
}
