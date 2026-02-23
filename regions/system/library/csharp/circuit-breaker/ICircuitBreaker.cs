namespace K1s0.System.CircuitBreaker;

public enum CircuitState
{
    Closed,
    Open,
    HalfOpen,
}

public class CircuitBreakerOpenException : Exception
{
    public CircuitBreakerOpenException()
        : base("Circuit breaker is open")
    {
    }
}

public record CircuitBreakerConfig(int FailureThreshold, int SuccessThreshold, TimeSpan Timeout);
