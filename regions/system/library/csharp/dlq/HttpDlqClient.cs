using System.Net;
using System.Net.Http.Json;

namespace K1s0.System.Dlq;

public sealed class HttpDlqClient : IDlqClient
{
    private readonly HttpClient _httpClient;

    public HttpDlqClient(HttpClient httpClient)
    {
        _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
    }

    public async Task<ListDlqMessagesResponse> ListMessagesAsync(
        string topic, int page = 1, int pageSize = 20, CancellationToken ct = default)
    {
        var url = $"api/v1/dlq/{Uri.EscapeDataString(topic)}?page={page}&pageSize={pageSize}";
        return await GetAsync<ListDlqMessagesResponse>(url, ct);
    }

    public async Task<DlqMessage> GetMessageAsync(Guid messageId, CancellationToken ct = default)
    {
        var url = $"api/v1/dlq/messages/{messageId}";
        return await GetAsync<DlqMessage>(url, ct);
    }

    public async Task<RetryDlqMessageResponse> RetryMessageAsync(Guid messageId, CancellationToken ct = default)
    {
        var url = $"api/v1/dlq/messages/{messageId}/retry";
        try
        {
            var response = await _httpClient.PostAsync(url, content: null, ct);
            await EnsureSuccessAsync(response, ct);
            return await response.Content.ReadFromJsonAsync<RetryDlqMessageResponse>(ct)
                ?? throw new DlqException(DlqErrorCodes.ServerError, "Empty response body");
        }
        catch (DlqException)
        {
            throw;
        }
        catch (HttpRequestException ex)
        {
            throw new DlqException(DlqErrorCodes.Network, "Network error during retry", ex);
        }
    }

    public async Task DeleteMessageAsync(Guid messageId, CancellationToken ct = default)
    {
        var url = $"api/v1/dlq/messages/{messageId}";
        try
        {
            var response = await _httpClient.DeleteAsync(url, ct);
            await EnsureSuccessAsync(response, ct);
        }
        catch (DlqException)
        {
            throw;
        }
        catch (HttpRequestException ex)
        {
            throw new DlqException(DlqErrorCodes.Network, "Network error during delete", ex);
        }
    }

    public async Task RetryAllAsync(string topic, CancellationToken ct = default)
    {
        var url = $"api/v1/dlq/{Uri.EscapeDataString(topic)}/retry-all";
        try
        {
            var response = await _httpClient.PostAsync(url, content: null, ct);
            await EnsureSuccessAsync(response, ct);
        }
        catch (DlqException)
        {
            throw;
        }
        catch (HttpRequestException ex)
        {
            throw new DlqException(DlqErrorCodes.Network, "Network error during retry-all", ex);
        }
    }

    public ValueTask DisposeAsync()
    {
        // HttpClient lifetime is managed by IHttpClientFactory; no disposal needed here
        return ValueTask.CompletedTask;
    }

    private async Task<T> GetAsync<T>(string url, CancellationToken ct)
    {
        try
        {
            var response = await _httpClient.GetAsync(url, ct);
            await EnsureSuccessAsync(response, ct);
            return await response.Content.ReadFromJsonAsync<T>(ct)
                ?? throw new DlqException(DlqErrorCodes.ServerError, "Empty response body");
        }
        catch (DlqException)
        {
            throw;
        }
        catch (HttpRequestException ex)
        {
            throw new DlqException(DlqErrorCodes.Network, $"Network error requesting {url}", ex);
        }
    }

    private static async Task EnsureSuccessAsync(HttpResponseMessage response, CancellationToken ct)
    {
        if (response.IsSuccessStatusCode)
        {
            return;
        }

        var body = await response.Content.ReadAsStringAsync(ct);

        throw response.StatusCode switch
        {
            HttpStatusCode.NotFound => new DlqException(DlqErrorCodes.NotFound, body),
            _ => new DlqException(DlqErrorCodes.ServerError, $"HTTP {(int)response.StatusCode}: {body}"),
        };
    }
}
