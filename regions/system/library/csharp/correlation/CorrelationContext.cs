namespace K1s0.System.Correlation;

public sealed record CorrelationContext(string CorrelationId, string TraceId)
{
    public static CorrelationContext New() => new(
        CorrelationIdGenerator.NewCorrelationId(),
        CorrelationIdGenerator.NewTraceId());
}
