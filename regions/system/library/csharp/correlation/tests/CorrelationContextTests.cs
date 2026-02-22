using Xunit;

namespace K1s0.System.Correlation.Tests;

public class CorrelationContextTests
{
    [Fact]
    public void New_GeneratesUniqueIds()
    {
        var ctx1 = CorrelationContext.New();
        var ctx2 = CorrelationContext.New();

        Assert.NotEqual(ctx1.CorrelationId, ctx2.CorrelationId);
        Assert.NotEqual(ctx1.TraceId, ctx2.TraceId);
    }

    [Fact]
    public void New_CorrelationIdIsValidGuid()
    {
        var ctx = CorrelationContext.New();
        Assert.True(Guid.TryParse(ctx.CorrelationId, out _));
    }

    [Fact]
    public void New_TraceIdIs32CharHex()
    {
        var ctx = CorrelationContext.New();
        Assert.Equal(32, ctx.TraceId.Length);
        Assert.All(ctx.TraceId, c => Assert.True(
            char.IsAsciiHexDigitLower(c),
            $"Character '{c}' is not a lowercase hex digit"));
    }

    [Fact]
    public void Record_Equality_Works()
    {
        var ctx1 = new CorrelationContext("corr-1", "trace-1");
        var ctx2 = new CorrelationContext("corr-1", "trace-1");

        Assert.Equal(ctx1, ctx2);
    }

    [Fact]
    public void Record_Inequality_Works()
    {
        var ctx1 = new CorrelationContext("corr-1", "trace-1");
        var ctx2 = new CorrelationContext("corr-2", "trace-2");

        Assert.NotEqual(ctx1, ctx2);
    }
}
