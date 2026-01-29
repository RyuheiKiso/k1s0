using System.Diagnostics.Metrics;

namespace K1s0.Observability;

/// <summary>
/// Common metrics for k1s0 services: request count, latency, and error count.
/// </summary>
public sealed class K1s0Metrics
{
    /// <summary>
    /// The meter name used for all k1s0 metrics.
    /// </summary>
    public const string MeterName = "k1s0";

    private readonly Counter<long> _requestCount;
    private readonly Histogram<double> _requestLatency;
    private readonly Counter<long> _errorCount;

    /// <summary>
    /// Creates a new <see cref="K1s0Metrics"/> instance using the provided <see cref="IMeterFactory"/>.
    /// </summary>
    /// <param name="meterFactory">The meter factory for creating instruments.</param>
    public K1s0Metrics(IMeterFactory meterFactory)
    {
        var meter = meterFactory.Create(MeterName);
        _requestCount = meter.CreateCounter<long>("k1s0.requests.total", description: "Total number of requests");
        _requestLatency = meter.CreateHistogram<double>("k1s0.requests.duration", "ms", "Request latency in milliseconds");
        _errorCount = meter.CreateCounter<long>("k1s0.errors.total", description: "Total number of errors");
    }

    /// <summary>
    /// Records a completed request.
    /// </summary>
    /// <param name="method">The method or operation name.</param>
    /// <param name="statusCode">The response status code.</param>
    public void RecordRequest(string method, int statusCode)
    {
        _requestCount.Add(1,
            new KeyValuePair<string, object?>("method", method),
            new KeyValuePair<string, object?>("status_code", statusCode));
    }

    /// <summary>
    /// Records request latency.
    /// </summary>
    /// <param name="method">The method or operation name.</param>
    /// <param name="durationMs">Duration in milliseconds.</param>
    public void RecordLatency(string method, double durationMs)
    {
        _requestLatency.Record(durationMs,
            new KeyValuePair<string, object?>("method", method));
    }

    /// <summary>
    /// Records an error occurrence.
    /// </summary>
    /// <param name="method">The method or operation name.</param>
    /// <param name="errorCode">The k1s0 error code.</param>
    public void RecordError(string method, string errorCode)
    {
        _errorCount.Add(1,
            new KeyValuePair<string, object?>("method", method),
            new KeyValuePair<string, object?>("error_code", errorCode));
    }
}
