using Xunit;

namespace K1s0.System.Correlation.Tests;

public class CorrelationHeadersTests
{
    [Fact]
    public void CorrelationId_HasExpectedValue()
    {
        Assert.Equal("X-Correlation-Id", CorrelationHeaders.CorrelationId);
    }

    [Fact]
    public void TraceId_HasExpectedValue()
    {
        Assert.Equal("X-Trace-Id", CorrelationHeaders.TraceId);
    }

    [Fact]
    public void RequestId_HasExpectedValue()
    {
        Assert.Equal("X-Request-Id", CorrelationHeaders.RequestId);
    }
}
