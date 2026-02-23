namespace K1s0.System.VaultClient;

public enum VaultErrorCode
{
    NotFound,
    PermissionDenied,
    ServerError,
    Timeout,
    LeaseExpired,
}

public class VaultException : Exception
{
    public VaultErrorCode Code { get; }

    public VaultException(VaultErrorCode code, string message) : base(message)
    {
        Code = code;
    }
}
