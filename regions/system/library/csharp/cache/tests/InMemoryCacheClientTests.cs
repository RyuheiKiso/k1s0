using K1s0.System.Cache;

namespace K1s0.System.Cache.Tests;

public class CacheTests
{
    [Fact]
    public async Task SetAndGet_ReturnsValue()
    {
        var cache = new InMemoryCacheClient();
        await cache.SetAsync("k1", "hello");
        var result = await cache.GetAsync("k1");
        Assert.Equal("hello", result);
    }

    [Fact]
    public async Task Get_NonExistent_ReturnsNull()
    {
        var cache = new InMemoryCacheClient();
        Assert.Null(await cache.GetAsync("missing"));
    }

    [Fact]
    public async Task Delete_Existing_ReturnsTrue()
    {
        var cache = new InMemoryCacheClient();
        await cache.SetAsync("k1", "val");
        Assert.True(await cache.DeleteAsync("k1"));
        Assert.Null(await cache.GetAsync("k1"));
    }

    [Fact]
    public async Task Delete_NonExistent_ReturnsFalse()
    {
        var cache = new InMemoryCacheClient();
        Assert.False(await cache.DeleteAsync("nope"));
    }

    [Fact]
    public async Task Exists_True_WhenPresent()
    {
        var cache = new InMemoryCacheClient();
        await cache.SetAsync("k1", "val");
        Assert.True(await cache.ExistsAsync("k1"));
    }

    [Fact]
    public async Task Exists_False_WhenMissing()
    {
        var cache = new InMemoryCacheClient();
        Assert.False(await cache.ExistsAsync("missing"));
    }

    [Fact]
    public async Task TTL_ExpiredEntry_ReturnsNull()
    {
        var cache = new InMemoryCacheClient();
        await cache.SetAsync("k1", "val", ttl: TimeSpan.FromMilliseconds(50));
        await Task.Delay(100);
        Assert.Null(await cache.GetAsync("k1"));
    }

    [Fact]
    public async Task SetNx_NewKey_ReturnsTrue()
    {
        var cache = new InMemoryCacheClient();
        var result = await cache.SetNxAsync("lock", "1", TimeSpan.FromSeconds(10));
        Assert.True(result);
        Assert.Equal("1", await cache.GetAsync("lock"));
    }

    [Fact]
    public async Task SetNx_ExistingKey_ReturnsFalse()
    {
        var cache = new InMemoryCacheClient();
        await cache.SetNxAsync("lock", "1", TimeSpan.FromSeconds(10));
        var result = await cache.SetNxAsync("lock", "2", TimeSpan.FromSeconds(10));
        Assert.False(result);
        Assert.Equal("1", await cache.GetAsync("lock"));
    }

    [Fact]
    public async Task SetNx_ExpiredKey_ReturnsTrue()
    {
        var cache = new InMemoryCacheClient();
        await cache.SetNxAsync("lock", "1", TimeSpan.FromMilliseconds(50));
        await Task.Delay(100);
        var result = await cache.SetNxAsync("lock", "2", TimeSpan.FromSeconds(10));
        Assert.True(result);
        Assert.Equal("2", await cache.GetAsync("lock"));
    }
}
