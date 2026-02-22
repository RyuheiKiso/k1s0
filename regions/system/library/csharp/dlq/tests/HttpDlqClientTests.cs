using System.Net;
using Xunit;

namespace K1s0.System.Dlq.Tests;

public class HttpDlqClientTests : IAsyncDisposable
{
    private static readonly Guid TestId = Guid.NewGuid();
    private static readonly DateTimeOffset Now = DateTimeOffset.UtcNow;

    [Fact]
    public async Task ListMessagesAsync_ReturnsMessages()
    {
        var json = """{"messages":[{"id":""" + $"\"{TestId}\"" + ""","originalTopic":"test-topic","errorMessage":"error","retryCount":1,"maxRetries":3,"payload":"{}","status":0,"createdAt":"2024-01-01T00:00:00+00:00","updatedAt":"2024-01-01T00:00:00+00:00"}],"total":1,"page":1,"pageSize":20}""";
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.OK, json);
        using var httpClient = new HttpClient(handler) { BaseAddress = new Uri("http://localhost/") };
        await using var client = new HttpDlqClient(httpClient);

        var result = await client.ListMessagesAsync("test-topic");

        Assert.NotNull(result);
        Assert.Equal(1, result.Total);
    }

    [Fact]
    public async Task GetMessageAsync_NotFound_ThrowsDlqException()
    {
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.NotFound, "not found");
        using var httpClient = new HttpClient(handler) { BaseAddress = new Uri("http://localhost/") };
        await using var client = new HttpDlqClient(httpClient);

        var ex = await Assert.ThrowsAsync<DlqException>(
            () => client.GetMessageAsync(TestId));

        Assert.Equal(DlqErrorCodes.NotFound, ex.Code);
    }

    [Fact]
    public async Task RetryMessageAsync_ServerError_ThrowsDlqException()
    {
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.InternalServerError, "server error");
        using var httpClient = new HttpClient(handler) { BaseAddress = new Uri("http://localhost/") };
        await using var client = new HttpDlqClient(httpClient);

        var ex = await Assert.ThrowsAsync<DlqException>(
            () => client.RetryMessageAsync(TestId));

        Assert.Equal(DlqErrorCodes.ServerError, ex.Code);
    }

    [Fact]
    public async Task DeleteMessageAsync_Success_DoesNotThrow()
    {
        using var handler = new FakeHttpMessageHandler(HttpStatusCode.NoContent, string.Empty);
        using var httpClient = new HttpClient(handler) { BaseAddress = new Uri("http://localhost/") };
        await using var client = new HttpDlqClient(httpClient);

        await client.DeleteMessageAsync(TestId);
    }

    public ValueTask DisposeAsync() => ValueTask.CompletedTask;

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
