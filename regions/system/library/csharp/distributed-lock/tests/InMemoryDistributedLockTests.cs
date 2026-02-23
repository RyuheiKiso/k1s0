using K1s0.System.DistributedLock;

namespace K1s0.System.DistributedLock.Tests;

public class InMemoryDistributedLockTests
{
    [Fact]
    public async Task Acquire_ReturnsGuard()
    {
        var dl = new InMemoryDistributedLock();
        var guard = await dl.AcquireAsync("key1", TimeSpan.FromSeconds(10));

        Assert.Equal("key1", guard.Key);
        Assert.False(string.IsNullOrEmpty(guard.Token));
    }

    [Fact]
    public async Task Acquire_AlreadyLocked_Throws()
    {
        var dl = new InMemoryDistributedLock();
        await dl.AcquireAsync("key1", TimeSpan.FromSeconds(10));

        await Assert.ThrowsAsync<LockException>(
            () => dl.AcquireAsync("key1", TimeSpan.FromSeconds(10)));
    }

    [Fact]
    public async Task Release_ThenReacquire_Succeeds()
    {
        var dl = new InMemoryDistributedLock();
        var guard = await dl.AcquireAsync("key1", TimeSpan.FromSeconds(10));
        await dl.ReleaseAsync(guard);

        var guard2 = await dl.AcquireAsync("key1", TimeSpan.FromSeconds(10));
        Assert.Equal("key1", guard2.Key);
    }

    [Fact]
    public async Task Release_WrongToken_Throws()
    {
        var dl = new InMemoryDistributedLock();
        await dl.AcquireAsync("key1", TimeSpan.FromSeconds(10));
        var fakeGuard = new LockGuard("key1", "wrong-token");

        await Assert.ThrowsAsync<LockException>(() => dl.ReleaseAsync(fakeGuard));
    }

    [Fact]
    public async Task IsLocked_WhenLocked_ReturnsTrue()
    {
        var dl = new InMemoryDistributedLock();
        await dl.AcquireAsync("key1", TimeSpan.FromSeconds(10));

        Assert.True(await dl.IsLockedAsync("key1"));
    }

    [Fact]
    public async Task IsLocked_WhenUnlocked_ReturnsFalse()
    {
        var dl = new InMemoryDistributedLock();
        Assert.False(await dl.IsLockedAsync("key1"));
    }

    [Fact]
    public async Task Acquire_ExpiredLock_Succeeds()
    {
        var dl = new InMemoryDistributedLock();
        await dl.AcquireAsync("key1", TimeSpan.FromMilliseconds(1));
        await Task.Delay(50);

        var guard = await dl.AcquireAsync("key1", TimeSpan.FromSeconds(10));
        Assert.Equal("key1", guard.Key);
    }
}
