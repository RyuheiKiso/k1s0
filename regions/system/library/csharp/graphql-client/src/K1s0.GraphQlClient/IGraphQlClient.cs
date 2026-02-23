namespace K1s0.GraphQlClient;

public interface IGraphQlClient
{
    Task<GraphQlResponse<T>> ExecuteAsync<T>(GraphQlQuery query, CancellationToken cancellationToken = default);

    Task<GraphQlResponse<T>> ExecuteMutationAsync<T>(GraphQlQuery mutation, CancellationToken cancellationToken = default);
}
