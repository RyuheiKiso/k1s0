using System.Text.Json;
using WireMock.RequestBuilders;
using WireMock.ResponseBuilders;
using WireMock.Server;

namespace K1s0.System.Dlq.Tests.Integration;

[Trait("Category", "Integration")]
public class DlqClientIntegrationTests : IAsyncDisposable
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    };

    private readonly WireMockServer _server;
    private readonly HttpDlqClient _client;

    public DlqClientIntegrationTests()
    {
        _server = WireMockServer.Start();
        var httpClient = new HttpClient
        {
            BaseAddress = new Uri(_server.Url + "/"),
        };
        _client = new HttpDlqClient(httpClient);
    }

    public async ValueTask DisposeAsync()
    {
        await _client.DisposeAsync();
        _server.Stop();
        _server.Dispose();
    }

    [Fact]
    public async Task ListMessagesAsync_ReturnsMessages()
    {
        var body = new ListDlqMessagesResponse(
            Messages:
            [
                new DlqMessage(
                    Guid.NewGuid(), "orders.v1", "err", 0, 3,
                    "{}", DlqStatus.Pending,
                    DateTimeOffset.UtcNow, DateTimeOffset.UtcNow),
            ],
            Total: 1,
            Page: 1,
            PageSize: 20);

        _server.Given(
            Request.Create()
                .WithPath("/api/v1/dlq/orders.v1")
                .UsingGet())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(JsonSerializer.Serialize(body, JsonOptions)));

        var result = await _client.ListMessagesAsync("orders.v1");

        Assert.Equal(1, result.Total);
        Assert.Single(result.Messages);
    }

    [Fact]
    public async Task GetMessageAsync_Returns404()
    {
        var messageId = Guid.NewGuid();

        _server.Given(
            Request.Create()
                .WithPath($"/api/v1/dlq/messages/{messageId}")
                .UsingGet())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(404)
                    .WithBody("Not found"));

        var ex = await Assert.ThrowsAsync<DlqException>(
            () => _client.GetMessageAsync(messageId));

        Assert.Equal(DlqErrorCodes.NotFound, ex.Code);
    }

    [Fact]
    public async Task RetryMessageAsync_ReturnsRetryResponse()
    {
        var messageId = Guid.NewGuid();
        var body = new RetryDlqMessageResponse(messageId, DlqStatus.Retrying, "Retry started");

        _server.Given(
            Request.Create()
                .WithPath($"/api/v1/dlq/messages/{messageId}/retry")
                .UsingPost())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(JsonSerializer.Serialize(body, JsonOptions)));

        var result = await _client.RetryMessageAsync(messageId);

        Assert.Equal(DlqStatus.Retrying, result.Status);
    }

    [Fact]
    public async Task DeleteMessageAsync_Succeeds()
    {
        var messageId = Guid.NewGuid();

        _server.Given(
            Request.Create()
                .WithPath($"/api/v1/dlq/messages/{messageId}")
                .UsingDelete())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200));

        await _client.DeleteMessageAsync(messageId);
    }

    [Fact]
    public async Task RetryAllAsync_Succeeds()
    {
        _server.Given(
            Request.Create()
                .WithPath("/api/v1/dlq/orders.v1/retry-all")
                .UsingPost())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200));

        await _client.RetryAllAsync("orders.v1");
    }
}
