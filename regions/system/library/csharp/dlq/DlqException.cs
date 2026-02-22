namespace K1s0.System.Dlq;

public class DlqException : Exception
{
    public string Code { get; }

    public DlqException(string code, string message, Exception? inner = null)
        : base(message, inner)
    {
        Code = code;
    }
}

public static class DlqErrorCodes
{
    public const string NotFound = "NOT_FOUND";
    public const string ServerError = "SERVER_ERROR";
    public const string Network = "NETWORK_ERROR";
}
