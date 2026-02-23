using System.Text.Json;
using K1s0.System.EventStore;

namespace K1s0.System.EventStore.Tests;

public class InMemoryEventStoreTests
{
    private static JsonElement Json(string raw) => JsonDocument.Parse(raw).RootElement.Clone();

    [Fact]
    public async Task AppendAndLoad_ReturnsEvents()
    {
        var store = new InMemoryEventStore();
        var events = new List<EventEnvelope>
        {
            new("e1", "stream-1", 0, "Created", Json("{\"name\":\"test\"}")),
            new("e2", "stream-1", 0, "Updated", Json("{\"name\":\"test2\"}")),
        };

        var version = await store.AppendAsync("stream-1", events, expectedVersion: 0);

        Assert.Equal(2, version);

        var result = await store.LoadAsync("stream-1");
        Assert.Equal(2, result.Count);
        Assert.Equal("Created", result[0].EventType);
        Assert.Equal(1, result[0].Version);
        Assert.Equal("Updated", result[1].EventType);
        Assert.Equal(2, result[1].Version);
    }

    [Fact]
    public async Task Load_NonExistent_ReturnsEmpty()
    {
        var store = new InMemoryEventStore();
        var result = await store.LoadAsync("missing");
        Assert.Empty(result);
    }

    [Fact]
    public async Task LoadFrom_FiltersOlderEvents()
    {
        var store = new InMemoryEventStore();
        var events = new List<EventEnvelope>
        {
            new("e1", "s1", 0, "E1", Json("{}")),
            new("e2", "s1", 0, "E2", Json("{}")),
            new("e3", "s1", 0, "E3", Json("{}")),
        };
        await store.AppendAsync("s1", events, 0);

        var result = await store.LoadFromAsync("s1", fromVersion: 2);
        Assert.Equal(2, result.Count);
        Assert.Equal("E2", result[0].EventType);
        Assert.Equal("E3", result[1].EventType);
    }

    [Fact]
    public async Task Append_VersionConflict_Throws()
    {
        var store = new InMemoryEventStore();
        var events1 = new List<EventEnvelope> { new("e1", "s1", 0, "E1", Json("{}")) };
        await store.AppendAsync("s1", events1, 0);

        var events2 = new List<EventEnvelope> { new("e2", "s1", 0, "E2", Json("{}")) };

        var ex = await Assert.ThrowsAsync<VersionConflictException>(() =>
            store.AppendAsync("s1", events2, expectedVersion: 0));
        Assert.Equal(0, ex.Expected);
        Assert.Equal(1, ex.Actual);
    }

    [Fact]
    public async Task Append_WithoutExpectedVersion_Succeeds()
    {
        var store = new InMemoryEventStore();
        await store.AppendAsync("s1", [new("e1", "s1", 0, "E1", Json("{}"))], expectedVersion: 0);

        var version = await store.AppendAsync("s1", [new("e2", "s1", 0, "E2", Json("{}"))]);
        Assert.Equal(2, version);
    }

    [Fact]
    public async Task ExistsAsync_ReturnsTrueForExisting()
    {
        var store = new InMemoryEventStore();
        Assert.False(await store.ExistsAsync("s1"));

        await store.AppendAsync("s1", [new("e1", "s1", 0, "E1", Json("{}"))]);
        Assert.True(await store.ExistsAsync("s1"));
    }

    [Fact]
    public async Task CurrentVersionAsync_ReturnsLatestVersion()
    {
        var store = new InMemoryEventStore();
        Assert.Equal(0, await store.CurrentVersionAsync("s1"));

        await store.AppendAsync("s1", [new("e1", "s1", 0, "E1", Json("{}"))]);
        Assert.Equal(1, await store.CurrentVersionAsync("s1"));

        await store.AppendAsync("s1", [new("e2", "s1", 0, "E2", Json("{}"))]);
        Assert.Equal(2, await store.CurrentVersionAsync("s1"));
    }

    [Fact]
    public async Task MultipleStreams_AreIndependent()
    {
        var store = new InMemoryEventStore();
        await store.AppendAsync("a", [new("e1", "a", 0, "A1", Json("{}"))]);
        await store.AppendAsync("b", [new("e2", "b", 0, "B1", Json("{}"))]);

        var ra = await store.LoadAsync("a");
        var rb = await store.LoadAsync("b");
        Assert.Single(ra);
        Assert.Single(rb);
        Assert.Equal("A1", ra[0].EventType);
        Assert.Equal("B1", rb[0].EventType);
    }
}
