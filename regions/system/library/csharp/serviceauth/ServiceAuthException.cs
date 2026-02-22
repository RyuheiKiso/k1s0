namespace K1s0.System.ServiceAuth;

public class ServiceAuthException : Exception
{
    public string Code { get; }

    public ServiceAuthException(string code, string message)
        : base(message)
    {
        Code = code;
    }

    public ServiceAuthException(string code, string message, Exception innerException)
        : base(message, innerException)
    {
        Code = code;
    }
}
