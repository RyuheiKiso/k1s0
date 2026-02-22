using System.Data;
using Dapper;
using Npgsql;

namespace K1s0.System.Outbox;

public sealed class PostgresOutboxStore : IOutboxStore
{
    private readonly string _connectionString;

    public PostgresOutboxStore(string connectionString)
    {
        _connectionString = connectionString ?? throw new ArgumentNullException(nameof(connectionString));
    }

    public async Task SaveAsync(OutboxMessage message, CancellationToken ct = default)
    {
        const string sql = """
            INSERT INTO outbox_messages (id, topic, payload, status, retry_count, created_at, updated_at, last_error)
            VALUES (@Id, @Topic, @Payload, @Status, @RetryCount, @CreatedAt, @UpdatedAt, @LastError)
            """;

        try
        {
            await using var connection = new NpgsqlConnection(_connectionString);
            await connection.OpenAsync(ct);
            await connection.ExecuteAsync(new CommandDefinition(
                sql,
                new
                {
                    message.Id,
                    message.Topic,
                    message.Payload,
                    Status = message.Status.ToString(),
                    message.RetryCount,
                    CreatedAt = message.CreatedAt.UtcDateTime,
                    UpdatedAt = message.UpdatedAt.UtcDateTime,
                    message.LastError,
                },
                cancellationToken: ct));
        }
        catch (NpgsqlException ex)
        {
            throw new OutboxException(OutboxErrorCodes.Save, $"Failed to save outbox message {message.Id}", ex);
        }
    }

    public async Task<IReadOnlyList<OutboxMessage>> FetchPendingAsync(int limit = 100, CancellationToken ct = default)
    {
        const string sql = """
            SELECT id, topic, payload, status, retry_count, created_at, updated_at, last_error
            FROM outbox_messages
            WHERE status = 'Pending'
            ORDER BY created_at ASC
            LIMIT @Limit
            """;

        try
        {
            await using var connection = new NpgsqlConnection(_connectionString);
            await connection.OpenAsync(ct);
            var rows = await connection.QueryAsync<OutboxRow>(new CommandDefinition(
                sql,
                new { Limit = limit },
                cancellationToken: ct));

            return rows.Select(r => new OutboxMessage(
                r.Id,
                r.Topic,
                r.Payload,
                Enum.Parse<OutboxStatus>(r.Status),
                r.RetryCount,
                new DateTimeOffset(r.CreatedAt, TimeSpan.Zero),
                new DateTimeOffset(r.UpdatedAt, TimeSpan.Zero),
                r.LastError)).ToList();
        }
        catch (NpgsqlException ex)
        {
            throw new OutboxException(OutboxErrorCodes.Fetch, "Failed to fetch pending outbox messages", ex);
        }
    }

    public async Task MarkPublishedAsync(Guid id, CancellationToken ct = default)
    {
        const string sql = """
            UPDATE outbox_messages
            SET status = 'Published', updated_at = @Now
            WHERE id = @Id
            """;

        try
        {
            await using var connection = new NpgsqlConnection(_connectionString);
            await connection.OpenAsync(ct);
            await connection.ExecuteAsync(new CommandDefinition(
                sql,
                new { Id = id, Now = DateTime.UtcNow },
                cancellationToken: ct));
        }
        catch (NpgsqlException ex)
        {
            throw new OutboxException(OutboxErrorCodes.DatabaseError, $"Failed to mark message {id} as published", ex);
        }
    }

    public async Task MarkFailedAsync(Guid id, string error, CancellationToken ct = default)
    {
        const string sql = """
            UPDATE outbox_messages
            SET status = 'Failed', last_error = @Error, retry_count = retry_count + 1, updated_at = @Now
            WHERE id = @Id
            """;

        try
        {
            await using var connection = new NpgsqlConnection(_connectionString);
            await connection.OpenAsync(ct);
            await connection.ExecuteAsync(new CommandDefinition(
                sql,
                new { Id = id, Error = error, Now = DateTime.UtcNow },
                cancellationToken: ct));
        }
        catch (NpgsqlException ex)
        {
            throw new OutboxException(OutboxErrorCodes.DatabaseError, $"Failed to mark message {id} as failed", ex);
        }
    }

    private sealed record OutboxRow(
        Guid Id,
        string Topic,
        byte[] Payload,
        string Status,
        int RetryCount,
        DateTime CreatedAt,
        DateTime UpdatedAt,
        string? LastError);
}
