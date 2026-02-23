namespace K1s0.System.TenantClient;

public enum TenantErrorCode
{
    NotFound,
    Suspended,
    ServerError,
    Timeout,
}

public class TenantException : Exception
{
    public TenantErrorCode Code { get; }

    public TenantException(string message, TenantErrorCode code)
        : base(message)
    {
        Code = code;
    }
}
