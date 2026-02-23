using K1s0.System.Migration;

namespace K1s0.System.Migration.Tests;

public class InMemoryMigrationRunnerTests
{
    private static InMemoryMigrationRunner CreateRunner()
    {
        var config = new MigrationConfig(".", "memory://");
        var ups = new[]
        {
            new MigrationEntry("20240101000001", "create_users", "CREATE TABLE users (id INT);"),
            new MigrationEntry("20240101000002", "add_email", "ALTER TABLE users ADD COLUMN email TEXT;"),
            new MigrationEntry("20240201000001", "create_orders", "CREATE TABLE orders (id INT);"),
        };
        var downs = new[]
        {
            new MigrationEntry("20240101000001", "create_users", "DROP TABLE users;"),
            new MigrationEntry("20240101000002", "add_email", "ALTER TABLE users DROP COLUMN email;"),
            new MigrationEntry("20240201000001", "create_orders", "DROP TABLE orders;"),
        };
        return new InMemoryMigrationRunner(config, ups, downs);
    }

    [Fact]
    public async Task RunUp_AppliesAll()
    {
        var runner = CreateRunner();
        var report = await runner.RunUpAsync();
        Assert.Equal(3, report.AppliedCount);
        Assert.Empty(report.Errors);
    }

    [Fact]
    public async Task RunUp_IsIdempotent()
    {
        var runner = CreateRunner();
        await runner.RunUpAsync();
        var report = await runner.RunUpAsync();
        Assert.Equal(0, report.AppliedCount);
    }

    [Fact]
    public async Task RunDown_RollsBackOneStep()
    {
        var runner = CreateRunner();
        await runner.RunUpAsync();
        var report = await runner.RunDownAsync(1);
        Assert.Equal(1, report.AppliedCount);

        var pending = await runner.PendingAsync();
        Assert.Single(pending);
        Assert.Equal("20240201000001", pending[0].Version);
    }

    [Fact]
    public async Task RunDown_RollsBackMultipleSteps()
    {
        var runner = CreateRunner();
        await runner.RunUpAsync();
        var report = await runner.RunDownAsync(2);
        Assert.Equal(2, report.AppliedCount);

        var pending = await runner.PendingAsync();
        Assert.Equal(2, pending.Count);
    }

    [Fact]
    public async Task RunDown_HandlesMoreThanApplied()
    {
        var runner = CreateRunner();
        await runner.RunUpAsync();
        var report = await runner.RunDownAsync(10);
        Assert.Equal(3, report.AppliedCount);
    }

    [Fact]
    public async Task Status_AllPendingInitially()
    {
        var runner = CreateRunner();
        var statuses = await runner.StatusAsync();
        Assert.Equal(3, statuses.Count);
        Assert.All(statuses, s => Assert.Null(s.AppliedAt));
    }

    [Fact]
    public async Task Status_AllAppliedAfterRunUp()
    {
        var runner = CreateRunner();
        await runner.RunUpAsync();
        var statuses = await runner.StatusAsync();
        Assert.Equal(3, statuses.Count);
        Assert.All(statuses, s => Assert.NotNull(s.AppliedAt));
    }

    [Fact]
    public async Task Pending_ReturnsAllUnapplied()
    {
        var runner = CreateRunner();
        var pending = await runner.PendingAsync();
        Assert.Equal(3, pending.Count);
        Assert.Equal("20240101000001", pending[0].Version);
        Assert.Equal("20240101000002", pending[1].Version);
        Assert.Equal("20240201000001", pending[2].Version);
    }

    [Fact]
    public async Task Pending_EmptyAfterFullApply()
    {
        var runner = CreateRunner();
        await runner.RunUpAsync();
        var pending = await runner.PendingAsync();
        Assert.Empty(pending);
    }
}

public class MigrationFileParserTests
{
    [Fact]
    public void ParseFilename_UpMigration()
    {
        var result = MigrationFileParser.ParseFilename("20240101000001_create_users.up.sql");
        Assert.NotNull(result);
        Assert.Equal("20240101000001", result.Version);
        Assert.Equal("create_users", result.Name);
        Assert.Equal(MigrationDirection.Up, result.Direction);
    }

    [Fact]
    public void ParseFilename_DownMigration()
    {
        var result = MigrationFileParser.ParseFilename("20240101000001_create_users.down.sql");
        Assert.NotNull(result);
        Assert.Equal("20240101000001", result.Version);
        Assert.Equal("create_users", result.Name);
        Assert.Equal(MigrationDirection.Down, result.Direction);
    }

    [Fact]
    public void ParseFilename_InvalidReturnsNull()
    {
        Assert.Null(MigrationFileParser.ParseFilename("invalid.sql"));
        Assert.Null(MigrationFileParser.ParseFilename("no_direction.sql"));
        Assert.Null(MigrationFileParser.ParseFilename("_.up.sql"));
    }

    [Fact]
    public void ComputeChecksum_Deterministic()
    {
        var content = "CREATE TABLE users (id SERIAL PRIMARY KEY);";
        Assert.Equal(
            MigrationFileParser.ComputeChecksum(content),
            MigrationFileParser.ComputeChecksum(content));
    }

    [Fact]
    public void ComputeChecksum_DiffersForDifferentContent()
    {
        Assert.NotEqual(
            MigrationFileParser.ComputeChecksum("CREATE TABLE users;"),
            MigrationFileParser.ComputeChecksum("CREATE TABLE orders;"));
    }
}

public class MigrationConfigTests
{
    [Fact]
    public void DefaultTableName()
    {
        var config = new MigrationConfig(".", "memory://");
        Assert.Equal("_migrations", config.TableName);
    }

    [Fact]
    public void CustomTableName()
    {
        var config = new MigrationConfig(".", "memory://", "custom");
        Assert.Equal("custom", config.TableName);
    }
}

public class MigrationExceptionTests
{
    [Fact]
    public void ConnectionFailed()
    {
        var ex = MigrationException.ConnectionFailed("fail");
        Assert.Equal("CONNECTION_FAILED", ex.ErrorCode);
    }

    [Fact]
    public void ChecksumMismatch()
    {
        var ex = MigrationException.ChecksumMismatch("v1", "abc", "def");
        Assert.Equal("CHECKSUM_MISMATCH", ex.ErrorCode);
    }

    [Fact]
    public void DirectoryNotFound()
    {
        var ex = MigrationException.DirectoryNotFound("/tmp");
        Assert.Equal("DIRECTORY_NOT_FOUND", ex.ErrorCode);
    }
}
