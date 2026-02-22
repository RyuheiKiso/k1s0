namespace K1s0.System.Auth;

public class AuthException : Exception
{
    public string Code { get; }

    public AuthException(string code, string message)
        : base(message)
    {
        Code = code;
    }

    public AuthException(string code, string message, Exception innerException)
        : base(message, innerException)
    {
        Code = code;
    }
}
