using K1s0.System.Idempotency;

namespace K1s0.System.Idempotency.Tests;

public class IdempotencyTests
{
    [Fact]
    public async Task InsertAndGet_ReturnsRecord()
    {
        var store = new InMemoryIdempotencyStore();
        await store.InsertAsync(new IdempotencyRecord("key-1"));

        var fetched = await store.GetAsync("key-1");
        Assert.NotNull(fetched);
        Assert.Equal("key-1", fetched.Key);
        Assert.Equal(IdempotencyStatus.Pending, fetched.Status);
    }

    [Fact]
    public async Task InsertDuplicate_ThrowsDuplicateKeyException()
    {
        var store = new InMemoryIdempotencyStore();
        await store.InsertAsync(new IdempotencyRecord("dup"));

        await Assert.ThrowsAsync<DuplicateKeyException>(() =>
            store.InsertAsync(new IdempotencyRecord("dup")));
    }

    [Fact]
    public async Task Get_NonExistent_ReturnsNull()
    {
        var store = new InMemoryIdempotencyStore();
        Assert.Null(await store.GetAsync("missing"));
    }

    [Fact]
    public async Task Update_ToCompleted()
    {
        var store = new InMemoryIdempotencyStore();
        await store.InsertAsync(new IdempotencyRecord("key-c"));

        await store.UpdateAsync("key-c", IdempotencyStatus.Completed, body: "{\"ok\":true}", code: 200);

        var fetched = await store.GetAsync("key-c");
        Assert.NotNull(fetched);
        Assert.Equal(IdempotencyStatus.Completed, fetched.Status);
        Assert.Equal("{\"ok\":true}", fetched.ResponseBody);
        Assert.Equal(200, fetched.StatusCode);
        Assert.NotNull(fetched.CompletedAt);
    }

    [Fact]
    public async Task Update_ToFailed()
    {
        var store = new InMemoryIdempotencyStore();
        await store.InsertAsync(new IdempotencyRecord("key-f"));
        await store.UpdateAsync("key-f", IdempotencyStatus.Failed, body: "error", code: 500);

        var fetched = await store.GetAsync("key-f");
        Assert.NotNull(fetched);
        Assert.Equal(IdempotencyStatus.Failed, fetched.Status);
    }

    [Fact]
    public async Task Delete_Existing_ReturnsTrue()
    {
        var store = new InMemoryIdempotencyStore();
        await store.InsertAsync(new IdempotencyRecord("del"));
        Assert.True(await store.DeleteAsync("del"));
        Assert.Null(await store.GetAsync("del"));
    }

    [Fact]
    public async Task Delete_NonExistent_ReturnsFalse()
    {
        var store = new InMemoryIdempotencyStore();
        Assert.False(await store.DeleteAsync("nope"));
    }

    [Fact]
    public async Task Expired_Record_ReturnsNull()
    {
        var store = new InMemoryIdempotencyStore();
        var record = new IdempotencyRecord("exp", ExpiresAt: DateTimeOffset.UtcNow.AddMilliseconds(50));
        await store.InsertAsync(record);

        await Task.Delay(100);
        Assert.Null(await store.GetAsync("exp"));
    }

    [Fact]
    public void Record_IsExpired_WithPastDate_ReturnsTrue()
    {
        var record = new IdempotencyRecord("r1", ExpiresAt: DateTimeOffset.UtcNow.AddSeconds(-10));
        Assert.True(record.IsExpired());
    }

    [Fact]
    public void Record_IsExpired_WithNoExpiry_ReturnsFalse()
    {
        var record = new IdempotencyRecord("r1");
        Assert.False(record.IsExpired());
    }
}
