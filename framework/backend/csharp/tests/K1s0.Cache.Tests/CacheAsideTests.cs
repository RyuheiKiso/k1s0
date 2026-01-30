using FluentAssertions;
using Moq;
using Xunit;

namespace K1s0.Cache.Tests;

public class CacheAsideTests
{
    private readonly Mock<ICacheOperations> _cacheMock;
    private readonly Patterns.CacheAside _cacheAside;

    public CacheAsideTests()
    {
        _cacheMock = new Mock<ICacheOperations>();
        _cacheAside = new Patterns.CacheAside(_cacheMock.Object, TimeSpan.FromMinutes(5));
    }

    [Fact]
    public async Task GetOrLoadAsync_ReturnsCachedValue_OnHit()
    {
        _cacheMock.Setup(c => c.GetAsync("key1", It.IsAny<CancellationToken>()))
            .ReturnsAsync("cached-value");

        var loaderCalled = false;
        var result = await _cacheAside.GetOrLoadAsync("key1", () =>
        {
            loaderCalled = true;
            return Task.FromResult("loaded-value");
        });

        result.Should().Be("cached-value");
        loaderCalled.Should().BeFalse();
    }

    [Fact]
    public async Task GetOrLoadAsync_CallsLoader_OnMiss()
    {
        _cacheMock.Setup(c => c.GetAsync("key1", It.IsAny<CancellationToken>()))
            .ReturnsAsync((string?)null);

        var result = await _cacheAside.GetOrLoadAsync("key1", () => Task.FromResult("loaded-value"));

        result.Should().Be("loaded-value");
        _cacheMock.Verify(c => c.SetAsync("key1", "loaded-value", TimeSpan.FromMinutes(5), It.IsAny<CancellationToken>()), Times.Once);
    }

    [Fact]
    public async Task GetOrLoadAsync_UsesCustomTtl_WhenProvided()
    {
        _cacheMock.Setup(c => c.GetAsync("key1", It.IsAny<CancellationToken>()))
            .ReturnsAsync((string?)null);

        var customTtl = TimeSpan.FromSeconds(30);
        await _cacheAside.GetOrLoadAsync("key1", () => Task.FromResult("value"), customTtl);

        _cacheMock.Verify(c => c.SetAsync("key1", "value", customTtl, It.IsAny<CancellationToken>()), Times.Once);
    }
}
