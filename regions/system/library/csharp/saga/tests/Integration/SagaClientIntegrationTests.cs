using System.Text.Json;
using WireMock.RequestBuilders;
using WireMock.ResponseBuilders;
using WireMock.Server;

namespace K1s0.System.Saga.Tests.Integration;

[Trait("Category", "Integration")]
public class SagaClientIntegrationTests : IAsyncDisposable
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    };

    private readonly WireMockServer _server;
    private readonly HttpSagaClient _client;

    public SagaClientIntegrationTests()
    {
        _server = WireMockServer.Start();
        var httpClient = new HttpClient
        {
            BaseAddress = new Uri(_server.Url + "/"),
        };
        _client = new HttpSagaClient(httpClient);
    }

    public async ValueTask DisposeAsync()
    {
        await _client.DisposeAsync();
        _server.Stop();
        _server.Dispose();
    }

    [Fact]
    public async Task StartSagaAsync_SendsPostAndReturnsResponse()
    {
        var body = new StartSagaResponse("saga-001", SagaStatus.Started);

        _server.Given(
            Request.Create()
                .WithPath("/api/v1/sagas")
                .UsingPost())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(JsonSerializer.Serialize(body, JsonOptions)));

        var request = new StartSagaRequest("order-fulfillment", """{"orderId":"123"}""", "corr-001");
        var result = await _client.StartSagaAsync(request);

        Assert.Equal("saga-001", result.SagaId);
        Assert.Equal(SagaStatus.Started, result.Status);
    }

    [Fact]
    public async Task GetSagaAsync_ReturnsState()
    {
        var body = new SagaState(
            SagaId: "saga-001",
            WorkflowName: "order-fulfillment",
            CurrentStep: "validate",
            Status: SagaStatus.Running,
            Payload: "{}",
            CorrelationId: "corr-001",
            CreatedAt: DateTimeOffset.UtcNow,
            UpdatedAt: DateTimeOffset.UtcNow,
            StepLogs:
            [
                new SagaStepLog("validate", "Completed", DateTimeOffset.UtcNow.AddSeconds(-10), DateTimeOffset.UtcNow, null),
            ]);

        _server.Given(
            Request.Create()
                .WithPath("/api/v1/sagas/saga-001")
                .UsingGet())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(JsonSerializer.Serialize(body, JsonOptions)));

        var result = await _client.GetSagaAsync("saga-001");

        Assert.Equal("saga-001", result.SagaId);
        Assert.Equal(SagaStatus.Running, result.Status);
        Assert.Single(result.StepLogs);
    }

    [Fact]
    public async Task GetSagaAsync_Returns404()
    {
        _server.Given(
            Request.Create()
                .WithPath("/api/v1/sagas/nonexistent")
                .UsingGet())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(404)
                    .WithBody("Saga not found"));

        var ex = await Assert.ThrowsAsync<SagaException>(
            () => _client.GetSagaAsync("nonexistent"));

        Assert.Equal(SagaErrorCodes.NotFound, ex.Code);
    }

    [Fact]
    public async Task CancelSagaAsync_Succeeds()
    {
        _server.Given(
            Request.Create()
                .WithPath("/api/v1/sagas/saga-001/cancel")
                .UsingPost())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200));

        await _client.CancelSagaAsync("saga-001");
    }

    [Fact]
    public async Task CancelSagaAsync_Returns409()
    {
        _server.Given(
            Request.Create()
                .WithPath("/api/v1/sagas/saga-002/cancel")
                .UsingPost())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(409)
                    .WithBody("Saga already completed"));

        var ex = await Assert.ThrowsAsync<SagaException>(
            () => _client.CancelSagaAsync("saga-002"));

        Assert.Equal(SagaErrorCodes.InvalidStatus, ex.Code);
    }
}
