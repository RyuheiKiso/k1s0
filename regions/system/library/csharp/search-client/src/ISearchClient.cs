namespace K1s0.System.SearchClient;

public interface ISearchClient : IAsyncDisposable
{
    Task<IndexResult> IndexDocumentAsync(string index, IndexDocument doc, CancellationToken ct = default);

    Task<BulkResult> BulkIndexAsync(string index, IEnumerable<IndexDocument> docs, CancellationToken ct = default);

    Task<SearchResult<T>> SearchAsync<T>(string index, SearchQuery query, CancellationToken ct = default);

    Task DeleteDocumentAsync(string index, string id, CancellationToken ct = default);

    Task CreateIndexAsync(string name, IndexMapping mapping, CancellationToken ct = default);
}
