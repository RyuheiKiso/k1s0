using System.Net;
using System.Net.Http.Json;

namespace K1s0.System.Saga;

public sealed class HttpSagaClient : ISagaClient
{
    private readonly HttpClient _httpClient;

    public HttpSagaClient(HttpClient httpClient)
    {
        _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
    }

    public async Task<StartSagaResponse> StartSagaAsync(StartSagaRequest request, CancellationToken ct = default)
    {
        try
        {
            var response = await _httpClient.PostAsJsonAsync("api/v1/sagas", request, ct);
            await EnsureSuccessAsync(response, ct);
            return await response.Content.ReadFromJsonAsync<StartSagaResponse>(ct)
                ?? throw new SagaException(SagaErrorCodes.ServerError, "Empty response body");
        }
        catch (SagaException)
        {
            throw;
        }
        catch (HttpRequestException ex)
        {
            throw new SagaException(SagaErrorCodes.Network, "Network error starting saga", ex);
        }
    }

    public async Task<SagaState> GetSagaAsync(string sagaId, CancellationToken ct = default)
    {
        var url = $"api/v1/sagas/{Uri.EscapeDataString(sagaId)}";
        try
        {
            var response = await _httpClient.GetAsync(url, ct);
            await EnsureSuccessAsync(response, ct);
            return await response.Content.ReadFromJsonAsync<SagaState>(ct)
                ?? throw new SagaException(SagaErrorCodes.ServerError, "Empty response body");
        }
        catch (SagaException)
        {
            throw;
        }
        catch (HttpRequestException ex)
        {
            throw new SagaException(SagaErrorCodes.Network, $"Network error getting saga {sagaId}", ex);
        }
    }

    public async Task CancelSagaAsync(string sagaId, CancellationToken ct = default)
    {
        var url = $"api/v1/sagas/{Uri.EscapeDataString(sagaId)}/cancel";
        try
        {
            var response = await _httpClient.PostAsync(url, content: null, ct);
            await EnsureSuccessAsync(response, ct);
        }
        catch (SagaException)
        {
            throw;
        }
        catch (HttpRequestException ex)
        {
            throw new SagaException(SagaErrorCodes.Network, $"Network error cancelling saga {sagaId}", ex);
        }
    }

    public ValueTask DisposeAsync()
    {
        return ValueTask.CompletedTask;
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
            HttpStatusCode.NotFound => new SagaException(SagaErrorCodes.NotFound, body),
            HttpStatusCode.Conflict => new SagaException(SagaErrorCodes.InvalidStatus, body),
            _ => new SagaException(SagaErrorCodes.ServerError, $"HTTP {(int)response.StatusCode}: {body}"),
        };
    }
}
