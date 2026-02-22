using System.Diagnostics;
using System.Diagnostics.Metrics;

namespace K1s0.System.Telemetry;

public sealed class K1s0Metrics
{
    public const string MeterName = "K1s0";

    private readonly Counter<long> _requestTotal;
    private readonly Histogram<double> _requestDuration;
    private readonly Counter<long> _requestErrors;
    private readonly UpDownCounter<long> _requestInFlight;

    public K1s0Metrics(IMeterFactory meterFactory)
    {
        var meter = meterFactory.Create(MeterName);
        _requestTotal = meter.CreateCounter<long>(
            "k1s0.request.total",
            description: "Total number of requests");
        _requestDuration = meter.CreateHistogram<double>(
            "k1s0.request.duration",
            unit: "ms",
            description: "Request duration in milliseconds");
        _requestErrors = meter.CreateCounter<long>(
            "k1s0.request.errors",
            description: "Total number of request errors");
        _requestInFlight = meter.CreateUpDownCounter<long>(
            "k1s0.request.in_flight",
            description: "Number of requests currently in flight");
    }

    public void RecordRequest(string method, string path, int statusCode, double durationMs)
    {
        var tags = new TagList
        {
            { "method", method },
            { "path", path },
            { "status_code", statusCode.ToString() },
        };

        _requestTotal.Add(1, tags);
        _requestDuration.Record(durationMs, tags);

        if (statusCode >= 400)
        {
            _requestErrors.Add(1, tags);
        }
    }

    public void IncrementInFlight() => _requestInFlight.Add(1);

    public void DecrementInFlight() => _requestInFlight.Add(-1);
}
