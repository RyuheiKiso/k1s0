namespace K1s0.System.Saga.Tests;

public class GrpcSagaClientTests
{
    [Fact]
    public async Task Constructor_AcceptsValidEndpoint()
    {
        await using var client = new GrpcSagaClient("http://localhost:5000");
        Assert.NotNull(client);
    }

    [Fact]
    public void Constructor_ThrowsOnEmptyEndpoint()
    {
        Assert.Throws<ArgumentException>(() => new GrpcSagaClient(string.Empty));
    }

    [Fact]
    public async Task StartSagaAsync_ThrowsNotImplemented()
    {
        await using var client = new GrpcSagaClient("http://localhost:5000");
        var request = new StartSagaRequest("wf", "{}", "corr");

        await Assert.ThrowsAsync<NotImplementedException>(
            () => client.StartSagaAsync(request));
    }

    [Fact]
    public async Task GetSagaAsync_ThrowsNotImplemented()
    {
        await using var client = new GrpcSagaClient("http://localhost:5000");

        await Assert.ThrowsAsync<NotImplementedException>(
            () => client.GetSagaAsync("saga-001"));
    }

    [Fact]
    public async Task CancelSagaAsync_ThrowsNotImplemented()
    {
        await using var client = new GrpcSagaClient("http://localhost:5000");

        await Assert.ThrowsAsync<NotImplementedException>(
            () => client.CancelSagaAsync("saga-001"));
    }
}
