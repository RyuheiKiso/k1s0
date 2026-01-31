using FluentAssertions;
using Moq;
using Xunit;

namespace K1s0.Cache.Tests;

public class WriteThroughTests
{
    private readonly Mock<ICacheOperations> _cacheMock;
    private readonly Patterns.WriteThrough _writeThrough;

    public WriteThroughTests()
    {
        _cacheMock = new Mock<ICacheOperations>();
        _writeThrough = new Patterns.WriteThrough(_cacheMock.Object, TimeSpan.FromMinutes(10));
    }

    [Fact]
    public async Task WriteAsync_CallsWriterBeforeCacheSet()
    {
        var callOrder = new List<string>();

        var writer = new Func<string, string, Task>((k, v) =>
        {
            callOrder.Add("writer");
            return Task.CompletedTask;
        });

        _cacheMock.Setup(c => c.SetAsync(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<TimeSpan?>(), It.IsAny<CancellationToken>()))
            .Callback(() => callOrder.Add("cache"))
            .Returns(Task.CompletedTask);

        await _writeThrough.WriteAsync("key", "value", writer);

        callOrder.Should().ContainInOrder("writer", "cache");
    }

    [Fact]
    public async Task WriteAsync_UsesDefaultTtl()
    {
        await _writeThrough.WriteAsync("key", "value", (_, _) => Task.CompletedTask);

        _cacheMock.Verify(c => c.SetAsync("key", "value", TimeSpan.FromMinutes(10), It.IsAny<CancellationToken>()), Times.Once);
    }

    [Fact]
    public async Task WriteAsync_PropagatesWriterException()
    {
        var writer = new Func<string, string, Task>((_, _) => throw new InvalidOperationException("db error"));

        var act = () => _writeThrough.WriteAsync("key", "value", writer);

        await act.Should().ThrowAsync<InvalidOperationException>().WithMessage("db error");

        // Cache should NOT be updated if writer fails.
        _cacheMock.Verify(c => c.SetAsync(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<TimeSpan?>(), It.IsAny<CancellationToken>()), Times.Never);
    }
}
