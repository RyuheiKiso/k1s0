using K1s0.System.EventBus;

namespace K1s0.System.EventBus.Tests;

public class InMemoryEventBusTests
{
    private static Event MakeEvent(string type = "test.event") =>
        new("1", type, new Dictionary<string, object> { ["key"] = "value" }, DateTimeOffset.UtcNow);

    [Fact]
    public async Task Publish_WithSubscriber_CallsHandler()
    {
        var bus = new InMemoryEventBus();
        Event? received = null;
        bus.Subscribe("test.event", (e, ct) => { received = e; return Task.CompletedTask; });

        await bus.PublishAsync(MakeEvent());

        Assert.NotNull(received);
        Assert.Equal("test.event", received!.EventType);
    }

    [Fact]
    public async Task Publish_NoSubscribers_DoesNotThrow()
    {
        var bus = new InMemoryEventBus();
        await bus.PublishAsync(MakeEvent());
    }

    [Fact]
    public async Task Unsubscribe_RemovesHandlers()
    {
        var bus = new InMemoryEventBus();
        var called = false;
        bus.Subscribe("test.event", (e, ct) => { called = true; return Task.CompletedTask; });
        bus.Unsubscribe("test.event");

        await bus.PublishAsync(MakeEvent());

        Assert.False(called);
    }

    [Fact]
    public async Task Subscribe_MultipleHandlers_AllCalled()
    {
        var bus = new InMemoryEventBus();
        var count = 0;
        bus.Subscribe("test.event", (e, ct) => { count++; return Task.CompletedTask; });
        bus.Subscribe("test.event", (e, ct) => { count++; return Task.CompletedTask; });

        await bus.PublishAsync(MakeEvent());

        Assert.Equal(2, count);
    }

    [Fact]
    public async Task Publish_DifferentEventType_DoesNotTrigger()
    {
        var bus = new InMemoryEventBus();
        var called = false;
        bus.Subscribe("type.a", (e, ct) => { called = true; return Task.CompletedTask; });

        await bus.PublishAsync(MakeEvent("type.b"));

        Assert.False(called);
    }

    [Fact]
    public void Event_RecordEquality()
    {
        var ts = DateTimeOffset.UtcNow;
        var payload = new Dictionary<string, object>();
        var a = new Event("1", "type", payload, ts);
        var b = new Event("1", "type", payload, ts);
        Assert.Equal(a, b);
    }
}
