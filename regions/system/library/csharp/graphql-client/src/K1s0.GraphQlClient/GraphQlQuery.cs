namespace K1s0.GraphQlClient;

public record GraphQlQuery(
    string Query,
    Dictionary<string, object>? Variables = null,
    string? OperationName = null);

public record GraphQlError(
    string Message,
    IReadOnlyList<ErrorLocation>? Locations = null,
    IReadOnlyList<object>? Path = null);

public record ErrorLocation(int Line, int Column);

public record GraphQlResponse<T>(T? Data, IReadOnlyList<GraphQlError>? Errors = null)
{
    public bool HasErrors => Errors is { Count: > 0 };
}
