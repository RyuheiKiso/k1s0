namespace K1s0.System.Telemetry;

public class TelemetryException : Exception
{
    public string Code { get; }

    public TelemetryException(string code, string message, Exception? inner = null)
        : base(message, inner)
    {
        Code = code;
    }
}
