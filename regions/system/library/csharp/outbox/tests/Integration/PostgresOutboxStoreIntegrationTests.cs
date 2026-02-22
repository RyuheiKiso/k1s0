using Dapper;
using Npgsql;
using Testcontainers.PostgreSql;

namespace K1s0.System.Outbox.Tests.Integration;

[Trait("Category", "Integration")]
public class PostgresOutboxStoreIntegrationTests : IAsyncLifetime
{
    private readonly PostgreSqlContainer _postgres = new PostgreSqlBuilder()
        .WithImage("postgres:16-alpine")
        .Build();

    private PostgresOutboxStore _store = null!;

    public async Task InitializeAsync()
    {
        await _postgres.StartAsync();

        await using var connection = new NpgsqlConnection(_postgres.GetConnectionString());
        await connection.OpenAsync();
        await connection.ExecuteAsync("""
            CREATE TABLE outbox_messages (
                id UUID PRIMARY KEY,
                topic TEXT NOT NULL,
                payload BYTEA NOT NULL,
                status TEXT NOT NULL DEFAULT 'Pending',
                retry_count INT NOT NULL DEFAULT 0,
                created_at TIMESTAMP NOT NULL,
                updated_at TIMESTAMP NOT NULL,
                last_error TEXT
            )
            """);

        _store = new PostgresOutboxStore(_postgres.GetConnectionString());
    }

    public async Task DisposeAsync()
    {
        await _postgres.DisposeAsync();
    }

    [Fact]
    public async Task SaveAsync_And_FetchPendingAsync_RoundTrips()
    {
        var message = new OutboxMessage(
            Id: Guid.NewGuid(),
            Topic: "test.topic.v1",
            Payload: [0xDE, 0xAD],
            Status: OutboxStatus.Pending,
            RetryCount: 0,
            CreatedAt: DateTimeOffset.UtcNow,
            UpdatedAt: DateTimeOffset.UtcNow,
            LastError: null);

        await _store.SaveAsync(message);

        var pending = await _store.FetchPendingAsync();
        Assert.Single(pending);
        Assert.Equal(message.Id, pending[0].Id);
        Assert.Equal(message.Topic, pending[0].Topic);
        Assert.Equal(OutboxStatus.Pending, pending[0].Status);
    }

    [Fact]
    public async Task MarkPublishedAsync_ChangesStatus()
    {
        var message = new OutboxMessage(
            Id: Guid.NewGuid(),
            Topic: "test.topic.v1",
            Payload: [0x01],
            Status: OutboxStatus.Pending,
            RetryCount: 0,
            CreatedAt: DateTimeOffset.UtcNow,
            UpdatedAt: DateTimeOffset.UtcNow,
            LastError: null);

        await _store.SaveAsync(message);
        await _store.MarkPublishedAsync(message.Id);

        var pending = await _store.FetchPendingAsync();
        Assert.Empty(pending);
    }

    [Fact]
    public async Task MarkFailedAsync_SetsErrorAndIncrementsRetry()
    {
        var message = new OutboxMessage(
            Id: Guid.NewGuid(),
            Topic: "test.topic.v1",
            Payload: [0x02],
            Status: OutboxStatus.Pending,
            RetryCount: 0,
            CreatedAt: DateTimeOffset.UtcNow,
            UpdatedAt: DateTimeOffset.UtcNow,
            LastError: null);

        await _store.SaveAsync(message);
        await _store.MarkFailedAsync(message.Id, "Connection timeout");

        await using var connection = new NpgsqlConnection(_postgres.GetConnectionString());
        await connection.OpenAsync();
        var row = await connection.QuerySingleAsync<dynamic>(
            "SELECT status, retry_count, last_error FROM outbox_messages WHERE id = @Id",
            new { message.Id });

        Assert.Equal("Failed", (string)row.status);
        Assert.Equal(1, (int)row.retry_count);
        Assert.Equal("Connection timeout", (string)row.last_error);
    }

    [Fact]
    public async Task FetchPendingAsync_RespectsLimit()
    {
        for (int i = 0; i < 5; i++)
        {
            var msg = new OutboxMessage(
                Id: Guid.NewGuid(),
                Topic: "test.topic.v1",
                Payload: [(byte)i],
                Status: OutboxStatus.Pending,
                RetryCount: 0,
                CreatedAt: DateTimeOffset.UtcNow.AddSeconds(i),
                UpdatedAt: DateTimeOffset.UtcNow,
                LastError: null);
            await _store.SaveAsync(msg);
        }

        var pending = await _store.FetchPendingAsync(limit: 3);
        Assert.Equal(3, pending.Count);
    }
}
