namespace K1s0.System.Resiliency;

public enum ResiliencyErrorKind
{
    MaxRetriesExceeded,
    CircuitBreakerOpen,
    BulkheadFull,
    Timeout,
}

public class ResiliencyException : Exception
{
    public ResiliencyErrorKind Kind { get; }

    public ResiliencyException(string message, ResiliencyErrorKind kind)
        : base(message)
    {
        Kind = kind;
    }

    public ResiliencyException(string message, ResiliencyErrorKind kind, Exception innerException)
        : base(message, innerException)
    {
        Kind = kind;
    }
}
