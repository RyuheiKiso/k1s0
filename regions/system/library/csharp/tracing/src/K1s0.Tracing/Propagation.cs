namespace K1s0.Tracing;

public static class Propagation
{
    public static void InjectContext(IDictionary<string, string> headers, TraceContext ctx, Baggage? baggage = null)
    {
        headers["traceparent"] = ctx.ToTraceparent();
        if (baggage != null)
        {
            var header = baggage.ToHeader();
            if (!string.IsNullOrEmpty(header))
            {
                headers["baggage"] = header;
            }
        }
    }

    public static (TraceContext? Context, Baggage Baggage) ExtractContext(IDictionary<string, string> headers)
    {
        TraceContext? ctx = null;
        if (headers.TryGetValue("traceparent", out var traceparent))
        {
            ctx = TraceContext.FromTraceparent(traceparent);
        }

        var baggage = headers.TryGetValue("baggage", out var baggageHeader)
            ? Baggage.FromHeader(baggageHeader)
            : new Baggage();

        return (ctx, baggage);
    }
}
