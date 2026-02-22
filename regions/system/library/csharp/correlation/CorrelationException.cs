namespace K1s0.System.Correlation;

public class CorrelationException : Exception
{
    public string Code { get; }

    public CorrelationException(string code, string message, Exception? inner = null)
        : base(message, inner)
    {
        Code = code;
    }
}
