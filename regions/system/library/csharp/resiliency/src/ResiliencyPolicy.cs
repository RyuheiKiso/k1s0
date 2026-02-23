namespace K1s0.System.Resiliency;

public record RetryConfig
{
    public int MaxAttempts { get; init; } = 3;

    public TimeSpan BaseDelay { get; init; } = TimeSpan.FromMilliseconds(100);

    public TimeSpan MaxDelay { get; init; } = TimeSpan.FromSeconds(5);

    public bool Jitter { get; init; } = true;
}

public record CircuitBreakerConfig
{
    public int FailureThreshold { get; init; } = 5;

    public TimeSpan RecoveryTimeout { get; init; } = TimeSpan.FromSeconds(30);

    public int HalfOpenMaxCalls { get; init; } = 2;
}

public record BulkheadConfig
{
    public int MaxConcurrentCalls { get; init; } = 20;

    public TimeSpan MaxWaitDuration { get; init; } = TimeSpan.FromMilliseconds(500);
}

public record ResiliencyPolicy
{
    public RetryConfig? Retry { get; init; }

    public CircuitBreakerConfig? CircuitBreaker { get; init; }

    public BulkheadConfig? Bulkhead { get; init; }

    public TimeSpan? Timeout { get; init; }
}
