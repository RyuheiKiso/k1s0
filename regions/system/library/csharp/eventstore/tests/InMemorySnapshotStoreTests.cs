using System.Text.Json;
using K1s0.System.EventStore;

namespace K1s0.System.EventStore.Tests;

public class InMemorySnapshotStoreTests
{
    private static JsonElement Json(string raw) => JsonDocument.Parse(raw).RootElement.Clone();

    [Fact]
    public async Task SaveAndLoad_ReturnsSnapshot()
    {
        var store = new InMemorySnapshotStore();
        var snapshot = new Snapshot("stream-1", 5, Json("{\"count\":10}"), DateTimeOffset.UtcNow);

        await store.SaveSnapshotAsync(snapshot);
        var loaded = await store.LoadSnapshotAsync("stream-1");

        Assert.NotNull(loaded);
        Assert.Equal("stream-1", loaded!.StreamId);
        Assert.Equal(5, loaded.Version);
    }

    [Fact]
    public async Task Load_NonExistent_ReturnsNull()
    {
        var store = new InMemorySnapshotStore();
        var result = await store.LoadSnapshotAsync("missing");
        Assert.Null(result);
    }

    [Fact]
    public async Task Save_OverwritesPrevious()
    {
        var store = new InMemorySnapshotStore();
        await store.SaveSnapshotAsync(new Snapshot("s1", 1, Json("{\"v\":1}"), DateTimeOffset.UtcNow));
        await store.SaveSnapshotAsync(new Snapshot("s1", 5, Json("{\"v\":5}"), DateTimeOffset.UtcNow));

        var loaded = await store.LoadSnapshotAsync("s1");
        Assert.NotNull(loaded);
        Assert.Equal(5, loaded!.Version);
    }
}
