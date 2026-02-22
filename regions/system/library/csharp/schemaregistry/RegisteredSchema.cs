namespace K1s0.System.SchemaRegistry;

public sealed record RegisteredSchema
{
    public required int Id { get; init; }

    public required int Version { get; init; }

    public required string SchemaString { get; init; }

    public required SchemaType SchemaType { get; init; }
}
