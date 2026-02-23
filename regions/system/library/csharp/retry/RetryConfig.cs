namespace K1s0.System.Retry;

public record RetryConfig(
    int MaxAttempts = 3,
    TimeSpan? InitialDelay = null,
    TimeSpan? MaxDelay = null,
    double Multiplier = 2.0,
    bool Jitter = true)
{
    private static readonly Random Rng = new();

    public TimeSpan ComputeDelay(int attempt)
    {
        var initial = InitialDelay ?? TimeSpan.FromMilliseconds(100);
        var max = MaxDelay ?? TimeSpan.FromSeconds(30);

        var delayMs = initial.TotalMilliseconds * Math.Pow(Multiplier, attempt);

        if (Jitter)
        {
            delayMs *= 0.5 + Rng.NextDouble();
        }

        delayMs = Math.Min(delayMs, max.TotalMilliseconds);
        return TimeSpan.FromMilliseconds(Math.Max(0, delayMs));
    }
}
