namespace K1s0.System.SearchClient;

public record Filter(string Field, string Operator, object Value, object? ValueTo = null);

public record FacetBucket(string Value, ulong Count);

public record SearchQuery(
    string Query,
    IReadOnlyList<Filter>? Filters = null,
    IReadOnlyList<string>? Facets = null,
    uint Page = 0,
    uint Size = 20);

public record SearchResult<T>(
    IReadOnlyList<T> Hits,
    ulong Total,
    IReadOnlyDictionary<string, IReadOnlyList<FacetBucket>> Facets,
    ulong TookMs);

public record IndexDocument(
    string Id,
    IReadOnlyDictionary<string, object> Fields);

public record IndexResult(string Id, long Version);

public record BulkFailure(string Id, string Error);

public record BulkResult(
    int SuccessCount,
    int FailedCount,
    IReadOnlyList<BulkFailure> Failures);

public record FieldMapping(string FieldType, bool Indexed = true);

public class IndexMapping
{
    public Dictionary<string, FieldMapping> Fields { get; } = new();

    public IndexMapping WithField(string name, string fieldType)
    {
        Fields[name] = new FieldMapping(fieldType);
        return this;
    }
}
