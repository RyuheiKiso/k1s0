namespace K1s0.System.Saga;

public class SagaException : Exception
{
    public string Code { get; }

    public SagaException(string code, string message, Exception? inner = null)
        : base(message, inner)
    {
        Code = code;
    }
}

public static class SagaErrorCodes
{
    public const string NotFound = "NOT_FOUND";
    public const string ServerError = "SERVER_ERROR";
    public const string Network = "NETWORK_ERROR";
    public const string InvalidStatus = "INVALID_STATUS";
}
