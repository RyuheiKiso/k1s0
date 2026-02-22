namespace K1s0.System.Messaging;

public class MessagingException : Exception
{
    public string Code { get; }

    public MessagingException(string code, string message)
        : base(message)
    {
        Code = code;
    }

    public MessagingException(string code, string message, Exception innerException)
        : base(message, innerException)
    {
        Code = code;
    }
}
