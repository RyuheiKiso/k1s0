using K1s0.System.AuditClient;

namespace K1s0.System.AuditClient.Tests;

public class BufferedAuditClientTests
{
    private static AuditEvent MakeEvent(string id = "1") =>
        new(id, "tenant-1", "actor-1", "create", "user", "res-1", DateTimeOffset.UtcNow);

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
}
