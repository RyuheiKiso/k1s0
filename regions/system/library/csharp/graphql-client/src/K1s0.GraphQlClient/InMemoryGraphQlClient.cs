namespace K1s0.GraphQlClient;

public class InMemoryGraphQlClient : IGraphQlClient
{
    private readonly Dictionary<string, object> _responses = new();

    public void SetResponse(string operationName, object response) =>
        _responses[operationName] = response;

    public Task<GraphQlResponse<T>> ExecuteAsync<T>(GraphQlQuery query, CancellationToken cancellationToken = default)
    {
        return Task.FromResult(Resolve<T>(query));
    }

    public Task<GraphQlResponse<T>> ExecuteMutationAsync<T>(GraphQlQuery mutation, CancellationToken cancellationToken = default)
    {
        return Task.FromResult(Resolve<T>(mutation));
    }

    private GraphQlResponse<T> Resolve<T>(GraphQlQuery query)
    {
        var key = query.OperationName ?? query.Query;
        if (!_responses.TryGetValue(key, out var response))
        {
            return new GraphQlResponse<T>(
                default,
                new List<GraphQlError> { new($"No response configured for: {key}") });
        }

        if (response is T typed)
        {
            return new GraphQlResponse<T>(typed);
        }

        return new GraphQlResponse<T>(
            default,
            new List<GraphQlError> { new("Response type mismatch") });
    }
}
