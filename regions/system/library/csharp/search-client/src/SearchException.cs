namespace K1s0.System.SearchClient;

public enum SearchErrorCode
{
    IndexNotFound,
    InvalidQuery,
    ServerError,
    Timeout,
}

public class SearchException : Exception
{
    public SearchErrorCode Code { get; }

    public SearchException(string message, SearchErrorCode code)
        : base(message)
    {
        Code = code;
    }
}
