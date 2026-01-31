using FluentAssertions;
using Moq;
using StackExchange.Redis;
using Xunit;

namespace K1s0.Cache.Tests;

public class CacheClientTests
{
    private readonly Mock<IDatabase> _dbMock;
    private readonly CacheClient _client;
    private readonly CacheConfig _config;

    public CacheClientTests()
    {
        _config = new CacheConfig(Prefix: "test");
        _dbMock = new Mock<IDatabase>();

        var connMock = new Mock<ConnectionManager>(new CacheConfig()) { CallBase = false };

        // We cannot easily mock ConnectionManager.GetDatabase() since it's not virtual.
        // Instead, we test CacheClient through a helper that exposes the prefixed key logic
        // and use a real CacheClient with a mock-friendly approach.
        // For unit tests, we create a testable subclass approach via composition.
        _client = CreateClientWithMockDb(_dbMock, _config);
    }

    private static CacheClient CreateClientWithMockDb(Mock<IDatabase> dbMock, CacheConfig config)
    {
        var connectionMock = new Mock<ConnectionManager>(config) { CallBase = false };
        connectionMock.Setup(c => c.GetDatabase()).Returns(dbMock.Object);
        return new CacheClient(connectionMock.Object, config);
    }

    [Fact]
    public async Task GetAsync_ReturnsValue_WhenKeyExists()
    {
        _dbMock.Setup(db => db.StringGetAsync(
                (RedisKey)"test:mykey", It.IsAny<CommandFlags>()))
            .ReturnsAsync((RedisValue)"hello");

        var result = await _client.GetAsync("mykey");

        result.Should().Be("hello");
    }

    [Fact]
    public async Task GetAsync_ReturnsNull_WhenKeyDoesNotExist()
    {
        _dbMock.Setup(db => db.StringGetAsync(
                (RedisKey)"test:mykey", It.IsAny<CommandFlags>()))
            .ReturnsAsync(RedisValue.Null);

        var result = await _client.GetAsync("mykey");

        result.Should().BeNull();
    }

    [Fact]
    public async Task SetAsync_CallsStringSet_WithPrefix()
    {
        await _client.SetAsync("mykey", "myvalue", TimeSpan.FromSeconds(60));

        _dbMock.Verify(db => db.StringSetAsync(
            (RedisKey)"test:mykey",
            (RedisValue)"myvalue",
            TimeSpan.FromSeconds(60),
            It.IsAny<bool>(),
            It.IsAny<When>(),
            It.IsAny<CommandFlags>()), Times.Once);
    }

    [Fact]
    public async Task DeleteAsync_ReturnsTrue_WhenKeyDeleted()
    {
        _dbMock.Setup(db => db.KeyDeleteAsync(
                (RedisKey)"test:mykey", It.IsAny<CommandFlags>()))
            .ReturnsAsync(true);

        var result = await _client.DeleteAsync("mykey");

        result.Should().BeTrue();
    }

    [Fact]
    public async Task ExistsAsync_ReturnsTrue_WhenKeyExists()
    {
        _dbMock.Setup(db => db.KeyExistsAsync(
                (RedisKey)"test:mykey", It.IsAny<CommandFlags>()))
            .ReturnsAsync(true);

        var result = await _client.ExistsAsync("mykey");

        result.Should().BeTrue();
    }

    [Fact]
    public async Task IncrAsync_ReturnsIncrementedValue()
    {
        _dbMock.Setup(db => db.StringIncrementAsync(
                (RedisKey)"test:counter", 5, It.IsAny<CommandFlags>()))
            .ReturnsAsync(10);

        var result = await _client.IncrAsync("counter", 5);

        result.Should().Be(10);
    }

    [Fact]
    public async Task DecrAsync_ReturnsDecrementedValue()
    {
        _dbMock.Setup(db => db.StringDecrementAsync(
                (RedisKey)"test:counter", 3, It.IsAny<CommandFlags>()))
            .ReturnsAsync(7);

        var result = await _client.DecrAsync("counter", 3);

        result.Should().Be(7);
    }

    [Fact]
    public void PrefixedKey_AppliesPrefix()
    {
        _client.PrefixedKey("foo").Should().Be("test:foo");
    }

    [Fact]
    public void PrefixedKey_NoPrefix_ReturnsOriginal()
    {
        var config = new CacheConfig(Prefix: "");
        var client = CreateClientWithMockDb(_dbMock, config);

        client.PrefixedKey("foo").Should().Be("foo");
    }
}
