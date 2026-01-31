namespace K1s0.Resilience;

/// <summary>
/// Represents the state of a circuit breaker.
/// </summary>
public enum CircuitState
{
    /// <summary>
    /// The circuit is closed and requests flow through normally.
    /// </summary>
    Closed,

    /// <summary>
    /// The circuit is open and requests are rejected immediately.
    /// </summary>
    Open,

    /// <summary>
    /// The circuit is half-open and a limited number of requests are allowed through to test recovery.
    /// </summary>
    HalfOpen,
}
