using System.Text.Json;

namespace K1s0.System.SearchClient;

public class InMemorySearchClient : ISearchClient
{
    private readonly Dictionary<string, List<IndexDocument>> _indexes = new();

    public Task CreateIndexAsync(string name, IndexMapping mapping, CancellationToken ct = default)
    {
        _indexes[name] = new List<IndexDocument>();
        return Task.CompletedTask;
    }

    public Task<IndexResult> IndexDocumentAsync(string index, IndexDocument doc, CancellationToken ct = default)
    {
        if (!_indexes.TryGetValue(index, out var docs))
        {
            throw new SearchException($"Index not found: {index}", SearchErrorCode.IndexNotFound);
        }

        docs.Add(doc);
        return Task.FromResult(new IndexResult(doc.Id, docs.Count));
    }

    public Task<BulkResult> BulkIndexAsync(string index, IEnumerable<IndexDocument> docs, CancellationToken ct = default)
    {
        if (!_indexes.TryGetValue(index, out var existing))
        {
            throw new SearchException($"Index not found: {index}", SearchErrorCode.IndexNotFound);
        }

        var docList = docs.ToList();
        existing.AddRange(docList);
        return Task.FromResult(new BulkResult(docList.Count, 0, Array.Empty<BulkFailure>()));
    }

    public Task<SearchResult<T>> SearchAsync<T>(string index, SearchQuery query, CancellationToken ct = default)
    {
        if (!_indexes.TryGetValue(index, out var docs))
        {
            throw new SearchException($"Index not found: {index}", SearchErrorCode.IndexNotFound);
        }

        var filtered = string.IsNullOrEmpty(query.Query)
            ? docs
            : docs.Where(d => d.Fields.Values.Any(v =>
                v?.ToString()?.Contains(query.Query, StringComparison.Ordinal) == true)).ToList();

        var paged = filtered
            .Skip((int)(query.Page * query.Size))
            .Take((int)query.Size)
            .ToList();

        var hits = paged.Select(d =>
        {
            var json = JsonSerializer.Serialize(d);
            return JsonSerializer.Deserialize<T>(json)!;
        }).ToList();

        var facets = new Dictionary<string, IReadOnlyList<FacetBucket>>();
        if (query.Facets != null)
        {
            foreach (var f in query.Facets)
            {
                facets[f] = new List<FacetBucket> { new("default", (ulong)paged.Count) };
            }
        }

        return Task.FromResult(new SearchResult<T>(
            hits,
            (ulong)filtered.Count,
            facets,
            1));
    }

    public Task DeleteDocumentAsync(string index, string id, CancellationToken ct = default)
    {
        if (_indexes.TryGetValue(index, out var docs))
        {
            docs.RemoveAll(d => d.Id == id);
        }

        return Task.CompletedTask;
    }

    public int DocumentCount(string index) =>
        _indexes.TryGetValue(index, out var docs) ? docs.Count : 0;

    public ValueTask DisposeAsync()
    {
        _indexes.Clear();
        return ValueTask.CompletedTask;
    }
}
