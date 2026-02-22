using ConfluentRegistry = Confluent.SchemaRegistry;

namespace K1s0.System.SchemaRegistry;

public sealed class ConfluentSchemaRegistryClient : ISchemaRegistryClient
{
    private readonly ConfluentRegistry.CachedSchemaRegistryClient _client;

    public ConfluentSchemaRegistryClient(SchemaRegistryConfig config)
    {
        var confluentConfig = new ConfluentRegistry.SchemaRegistryConfig
        {
            Url = config.Url,
        };

        if (config.Username is not null && config.Password is not null)
        {
            confluentConfig.BasicAuthCredentialsSource = ConfluentRegistry.AuthCredentialsSource.UserInfo;
            confluentConfig.BasicAuthUserInfo = $"{config.Username}:{config.Password}";
        }

        _client = new ConfluentRegistry.CachedSchemaRegistryClient(confluentConfig);
    }

    public async Task<int> RegisterSchemaAsync(string subject, string schema, SchemaType type, CancellationToken ct = default)
    {
        try
        {
            var confluentType = type switch
            {
                SchemaType.Avro => ConfluentRegistry.SchemaType.Avro,
                SchemaType.Json => ConfluentRegistry.SchemaType.Json,
                SchemaType.Protobuf => ConfluentRegistry.SchemaType.Protobuf,
                _ => throw new ArgumentOutOfRangeException(nameof(type)),
            };

            var registeredSchema = new ConfluentRegistry.Schema(schema, confluentType);
            return await _client.RegisterSchemaAsync(subject, registeredSchema).ConfigureAwait(false);
        }
        catch (ConfluentRegistry.SchemaRegistryException ex)
        {
            throw new SchemaRegistryException(
                SchemaRegistryException.ErrorCodes.RegistrationFailed,
                $"Failed to register schema for subject '{subject}': {ex.Message}",
                ex);
        }
    }

    public async Task<RegisteredSchema> GetSchemaByIdAsync(int id, CancellationToken ct = default)
    {
        try
        {
            var schema = await _client.GetSchemaAsync(id).ConfigureAwait(false);

            return new RegisteredSchema
            {
                Id = id,
                Version = 0,
                SchemaString = schema.SchemaString,
                SchemaType = schema.SchemaType switch
                {
                    ConfluentRegistry.SchemaType.Avro => SchemaType.Avro,
                    ConfluentRegistry.SchemaType.Json => SchemaType.Json,
                    ConfluentRegistry.SchemaType.Protobuf => SchemaType.Protobuf,
                    _ => SchemaType.Avro,
                },
            };
        }
        catch (ConfluentRegistry.SchemaRegistryException ex)
        {
            throw new SchemaRegistryException(
                SchemaRegistryException.ErrorCodes.SchemaNotFound,
                $"Failed to get schema with ID {id}: {ex.Message}",
                ex);
        }
    }

    public async Task<bool> CheckCompatibilityAsync(string subject, string schema, CancellationToken ct = default)
    {
        try
        {
            return await _client.IsCompatibleAsync(subject, new ConfluentRegistry.Schema(schema, ConfluentRegistry.SchemaType.Avro))
                .ConfigureAwait(false);
        }
        catch (ConfluentRegistry.SchemaRegistryException ex)
        {
            throw new SchemaRegistryException(
                SchemaRegistryException.ErrorCodes.CompatibilityCheckFailed,
                $"Compatibility check failed for subject '{subject}': {ex.Message}",
                ex);
        }
    }

    public ValueTask DisposeAsync()
    {
        _client.Dispose();
        return ValueTask.CompletedTask;
    }
}
