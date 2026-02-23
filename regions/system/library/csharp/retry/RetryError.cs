namespace K1s0.System.Retry;

public class RetryExhaustedException(int attempts, Exception lastError)
    : Exception($"Exhausted {attempts} retry attempts: {lastError.Message}", lastError)
{
    public int Attempts { get; } = attempts;

    public Exception LastError { get; } = lastError;
}
