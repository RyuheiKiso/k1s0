namespace K1s0.System.Outbox;

public class OutboxException : Exception
{
    public string Code { get; }

    public OutboxException(string code, string message, Exception? inner = null)
        : base(message, inner)
    {
        Code = code;
    }
}

public static class OutboxErrorCodes
{
    public const string Save = "SAVE_ERROR";
    public const string Fetch = "FETCH_ERROR";
    public const string Publish = "PUBLISH_ERROR";
    public const string DatabaseError = "DATABASE_ERROR";
}
