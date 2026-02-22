using Xunit;

namespace K1s0.System.Correlation.Tests;

public class CorrelationIdGeneratorTests
{
    [Fact]
    public void NewCorrelationId_ReturnsValidGuid()
    {
        string id = CorrelationIdGenerator.NewCorrelationId();
        Assert.True(Guid.TryParse(id, out _));
    }

    [Fact]
    public void NewCorrelationId_IsUnique()
    {
        string id1 = CorrelationIdGenerator.NewCorrelationId();
        string id2 = CorrelationIdGenerator.NewCorrelationId();
        Assert.NotEqual(id1, id2);
    }

    [Fact]
    public void NewTraceId_Returns32CharLowercaseHex()
    {
        string traceId = CorrelationIdGenerator.NewTraceId();
        Assert.Equal(32, traceId.Length);
        Assert.All(traceId, c => Assert.True(
            char.IsAsciiHexDigitLower(c),
            $"Character '{c}' is not a lowercase hex digit"));
    }

    [Fact]
    public void NewTraceId_IsUnique()
    {
        string id1 = CorrelationIdGenerator.NewTraceId();
        string id2 = CorrelationIdGenerator.NewTraceId();
        Assert.NotEqual(id1, id2);
    }

    [Fact]
    public void NewTraceId_MultipleCallsAllValid()
    {
        for (int i = 0; i < 100; i++)
        {
            string traceId = CorrelationIdGenerator.NewTraceId();
            Assert.Equal(32, traceId.Length);
        }
    }
}
