using K1s0.Tracing;

namespace K1s0.Tracing.Tests;

public class TracingTests
{
    [Fact]
    public void TraceContext_ToTraceparent_FormatsCorrectly()
    {
        var ctx = new TraceContext(new string('a', 32), new string('b', 16), 1);
        Assert.Equal($"00-{new string('a', 32)}-{new string('b', 16)}-01", ctx.ToTraceparent());
    }

    [Fact]
    public void TraceContext_ToTraceparent_PadsFlags()
    {
        var ctx = new TraceContext(new string('0', 32), new string('0', 16), 0);
        Assert.EndsWith("-00", ctx.ToTraceparent());
    }

    [Fact]
    public void TraceContext_FromTraceparent_ParsesValidString()
    {
        var input = $"00-{new string('a', 32)}-{new string('b', 16)}-01";
        var ctx = TraceContext.FromTraceparent(input);
        Assert.NotNull(ctx);
        Assert.Equal(new string('a', 32), ctx!.TraceId);
        Assert.Equal(new string('b', 16), ctx.ParentId);
        Assert.Equal(1, ctx.Flags);
    }

    [Fact]
    public void TraceContext_FromTraceparent_ReturnsNullForInvalidVersion()
    {
        var input = $"01-{new string('a', 32)}-{new string('b', 16)}-01";
        Assert.Null(TraceContext.FromTraceparent(input));
    }

    [Fact]
    public void TraceContext_FromTraceparent_ReturnsNullForWrongPartCount()
    {
        Assert.Null(TraceContext.FromTraceparent("00-abc"));
    }

    [Fact]
    public void TraceContext_FromTraceparent_ReturnsNullForWrongTraceIdLength()
    {
        var input = $"00-abc-{new string('b', 16)}-01";
        Assert.Null(TraceContext.FromTraceparent(input));
    }

    [Fact]
    public void TraceContext_FromTraceparent_ReturnsNullForWrongParentIdLength()
    {
        var input = $"00-{new string('a', 32)}-abc-01";
        Assert.Null(TraceContext.FromTraceparent(input));
    }

    [Fact]
    public void TraceContext_FromTraceparent_ReturnsNullForWrongFlagsLength()
    {
        var input = $"00-{new string('a', 32)}-{new string('b', 16)}-1";
        Assert.Null(TraceContext.FromTraceparent(input));
    }

    [Fact]
    public void TraceContext_Roundtrip()
    {
        var original = new TraceContext("abcd1234abcd1234abcd1234abcd1234", "ef567890ef567890", 1);
        var parsed = TraceContext.FromTraceparent(original.ToTraceparent());
        Assert.NotNull(parsed);
        Assert.Equal(original.TraceId, parsed!.TraceId);
        Assert.Equal(original.ParentId, parsed.ParentId);
        Assert.Equal(original.Flags, parsed.Flags);
    }

    [Fact]
    public void Baggage_SetAndGet()
    {
        var baggage = new Baggage();
        baggage.Set("key1", "value1");
        Assert.Equal("value1", baggage.Get("key1"));
    }

    [Fact]
    public void Baggage_GetReturnsNullForMissingKey()
    {
        var baggage = new Baggage();
        Assert.Null(baggage.Get("missing"));
    }

    [Fact]
    public void Baggage_ToHeader_FormatsCorrectly()
    {
        var baggage = new Baggage();
        baggage.Set("k1", "v1");
        baggage.Set("k2", "v2");
        var header = baggage.ToHeader();
        Assert.Contains("k1=v1", header);
        Assert.Contains("k2=v2", header);
    }

    [Fact]
    public void Baggage_FromHeader_ParsesCorrectly()
    {
        var baggage = Baggage.FromHeader("k1=v1,k2=v2");
        Assert.Equal("v1", baggage.Get("k1"));
        Assert.Equal("v2", baggage.Get("k2"));
    }

    [Fact]
    public void Baggage_FromHeader_HandlesEmptyString()
    {
        var baggage = Baggage.FromHeader("");
        Assert.Empty(baggage.Entries);
    }

    [Fact]
    public void Baggage_Roundtrip()
    {
        var original = new Baggage();
        original.Set("tenant", "acme");
        original.Set("region", "us-east");
        var parsed = Baggage.FromHeader(original.ToHeader());
        Assert.Equal("acme", parsed.Get("tenant"));
        Assert.Equal("us-east", parsed.Get("region"));
    }

    [Fact]
    public void Propagation_InjectContext_AddsTraceparentHeader()
    {
        var headers = new Dictionary<string, string>();
        var ctx = new TraceContext(new string('a', 32), new string('b', 16), 1);
        Propagation.InjectContext(headers, ctx);
        Assert.True(headers.ContainsKey("traceparent"));
        Assert.False(headers.ContainsKey("baggage"));
    }

    [Fact]
    public void Propagation_InjectContext_AddsBaggageHeader()
    {
        var headers = new Dictionary<string, string>();
        var ctx = new TraceContext(new string('a', 32), new string('b', 16), 1);
        var baggage = new Baggage();
        baggage.Set("tenant", "acme");
        Propagation.InjectContext(headers, ctx, baggage);
        Assert.True(headers.ContainsKey("traceparent"));
        Assert.Equal("tenant=acme", headers["baggage"]);
    }

    [Fact]
    public void Propagation_InjectContext_SkipsEmptyBaggage()
    {
        var headers = new Dictionary<string, string>();
        var ctx = new TraceContext(new string('a', 32), new string('b', 16), 1);
        Propagation.InjectContext(headers, ctx, new Baggage());
        Assert.False(headers.ContainsKey("baggage"));
    }

    [Fact]
    public void Propagation_ExtractContext_ParsesHeaders()
    {
        var headers = new Dictionary<string, string>
        {
            ["traceparent"] = $"00-{new string('a', 32)}-{new string('b', 16)}-01",
            ["baggage"] = "key=val",
        };
        var (context, baggage) = Propagation.ExtractContext(headers);
        Assert.NotNull(context);
        Assert.Equal(new string('a', 32), context!.TraceId);
        Assert.Equal("val", baggage.Get("key"));
    }

    [Fact]
    public void Propagation_ExtractContext_WithMissingTraceparent()
    {
        var (context, baggage) = Propagation.ExtractContext(new Dictionary<string, string>());
        Assert.Null(context);
        Assert.Empty(baggage.Entries);
    }

    [Fact]
    public void Propagation_Roundtrip()
    {
        var ctx = new TraceContext(new string('c', 32), new string('d', 16), 0);
        var baggage = new Baggage();
        baggage.Set("env", "prod");

        var headers = new Dictionary<string, string>();
        Propagation.InjectContext(headers, ctx, baggage);
        var (extractedCtx, extractedBaggage) = Propagation.ExtractContext(headers);

        Assert.Equal(ctx.TraceId, extractedCtx!.TraceId);
        Assert.Equal(ctx.ParentId, extractedCtx.ParentId);
        Assert.Equal(ctx.Flags, extractedCtx.Flags);
        Assert.Equal("prod", extractedBaggage.Get("env"));
    }
}
