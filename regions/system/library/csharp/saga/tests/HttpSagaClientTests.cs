using System.Net;
using Xunit;

namespace K1s0.System.Saga.Tests;

public class HttpSagaClientTests
{
    private static readonly StartSagaRequest TestRequest = new("wf-001", "{\"orderId\":\"123\"}", "corr-001");

    [Fact]
    public async Task StartSagaAsync_Success_ReturnsSagaId()
    {
        var responseJson = """{"sagaId":"saga-001","workflowId":"wf-001","status":0,"currentStep":null,"createdAt":"2024-01-01T00:00:00+00:00","updatedAt":"2024-01-01T00:00:00+00:00"}""";
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.OK, responseJson);
        using var httpClient = new HttpClient(handler) { BaseAddress = new Uri("http://localhost/") };
        await using var client = new HttpSagaClient(httpClient);

        var result = await client.StartSagaAsync(TestRequest);

        Assert.NotNull(result);
    }

    [Fact]
    public async Task StartSagaAsync_ServerError_ThrowsSagaException()
    {
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.InternalServerError, "error");
        using var httpClient = new HttpClient(handler) { BaseAddress = new Uri("http://localhost/") };
        await using var client = new HttpSagaClient(httpClient);

        var ex = await Assert.ThrowsAsync<SagaException>(
            () => client.StartSagaAsync(TestRequest));

        Assert.Equal(SagaErrorCodes.ServerError, ex.Code);
    }

    [Fact]
    public async Task GetSagaAsync_NotFound_ThrowsSagaException()
    {
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.NotFound, "not found");
        using var httpClient = new HttpClient(handler) { BaseAddress = new Uri("http://localhost/") };
        await using var client = new HttpSagaClient(httpClient);

        var ex = await Assert.ThrowsAsync<SagaException>(
            () => client.GetSagaAsync("saga-001"));

        Assert.Equal(SagaErrorCodes.NotFound, ex.Code);
    }

    [Fact]
    public async Task CancelSagaAsync_Conflict_ThrowsSagaException()
    {
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.Conflict, "conflict");
        using var httpClient = new HttpClient(handler) { BaseAddress = new Uri("http://localhost/") };
        await using var client = new HttpSagaClient(httpClient);

        var ex = await Assert.ThrowsAsync<SagaException>(
            () => client.CancelSagaAsync("saga-001"));

        Assert.Equal(SagaErrorCodes.InvalidStatus, ex.Code);
    }

    private sealed class FakeHttpMessageHandler(HttpStatusCode statusCode, string responseBody) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request, CancellationToken cancellationToken)
        {
            var response = new HttpResponseMessage(statusCode)
            {
                Content = new StringContent(responseBody),
            };
            return Task.FromResult(response);
        }
    }
}
