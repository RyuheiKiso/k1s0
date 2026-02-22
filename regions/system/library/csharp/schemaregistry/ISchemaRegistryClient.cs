namespace K1s0.System.SchemaRegistry;

public interface ISchemaRegistryClient : IAsyncDisposable
{
    Task<int> RegisterSchemaAsync(string subject, string schema, SchemaType type, CancellationToken ct = default);

    Task<RegisteredSchema> GetSchemaByIdAsync(int id, CancellationToken ct = default);

    Task<bool> CheckCompatibilityAsync(string subject, string schema, CancellationToken ct = default);
}
