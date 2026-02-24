using K1s0.System.AuditClient;

namespace K1s0.System.AuditClient.Tests;

public class BufferedAuditClientTests
{
    private static AuditEvent MakeEvent(string id = "1", string tenantId = "tenant-1", string actorId = "actor-1", string action = "create") =>
        new(id, tenantId, actorId, action, "user", "res-1", DateTimeOffset.UtcNow);

    [Fact]
    public async Task Record_AddsToBuffer()
    {
        var client = new BufferedAuditClient();
        await client.RecordAsync(MakeEvent());

        var events = await client.FlushAsync();
        Assert.Single(events);
    }

    [Fact]
    public async Task Flush_ClearsBuffer()
    {
        var client = new BufferedAuditClient();
        await client.RecordAsync(MakeEvent("1"));
        await client.RecordAsync(MakeEvent("2"));

        var first = await client.FlushAsync();
        Assert.Equal(2, first.Count);

        var second = await client.FlushAsync();
        Assert.Empty(second);
    }

    [Fact]
    public async Task Flush_EmptyBuffer_ReturnsEmpty()
    {
        var client = new BufferedAuditClient();
        var events = await client.FlushAsync();
        Assert.Empty(events);
    }

    [Fact]
    public async Task Record_PreservesEventData()
    {
        var client = new BufferedAuditClient();
        var evt = MakeEvent("42");
        await client.RecordAsync(evt);

        var events = await client.FlushAsync();
        Assert.Equal("42", events[0].Id);
        Assert.Equal("tenant-1", events[0].TenantId);
        Assert.Equal("actor-1", events[0].ActorId);
    }

    [Fact]
    public async Task Query_ByTenantId_FiltersCorrectly()
    {
        var client = new BufferedAuditClient();
        await client.RecordAsync(MakeEvent("1", tenantId: "tenant-A"));
        await client.RecordAsync(MakeEvent("2", tenantId: "tenant-B"));
        await client.RecordAsync(MakeEvent("3", tenantId: "tenant-A"));

        var result = await client.QueryAsync(new AuditFilter(TenantId: "tenant-A"));

        Assert.Equal(2, result.Count);
        Assert.All(result, e => Assert.Equal("tenant-A", e.TenantId));
    }

    [Fact]
    public async Task Query_ByAction_FiltersCorrectly()
    {
        var client = new BufferedAuditClient();
        await client.RecordAsync(MakeEvent("1", action: "create"));
        await client.RecordAsync(MakeEvent("2", action: "delete"));
        await client.RecordAsync(MakeEvent("3", action: "create"));

        var result = await client.QueryAsync(new AuditFilter(Action: "delete"));

        Assert.Single(result);
        Assert.Equal("2", result[0].Id);
    }

    [Fact]
    public async Task Query_EmptyFilter_ReturnsAll()
    {
        var client = new BufferedAuditClient();
        await client.RecordAsync(MakeEvent("1"));
        await client.RecordAsync(MakeEvent("2"));

        var result = await client.QueryAsync(new AuditFilter());

        Assert.Equal(2, result.Count);
    }

    [Fact]
    public async Task Query_ByActorId_FiltersCorrectly()
    {
        var client = new BufferedAuditClient();
        await client.RecordAsync(MakeEvent("1", actorId: "user-A"));
        await client.RecordAsync(MakeEvent("2", actorId: "user-B"));

        var result = await client.QueryAsync(new AuditFilter(ActorId: "user-A"));

        Assert.Single(result);
        Assert.Equal("user-A", result[0].ActorId);
    }
}
