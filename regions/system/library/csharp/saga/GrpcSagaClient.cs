using Grpc.Net.Client;

namespace K1s0.System.Saga;

/// <summary>
/// gRPC implementation of ISagaClient.
/// Stub implementation until proto files are available for code generation.
/// </summary>
public sealed class GrpcSagaClient : ISagaClient
{
    private readonly GrpcChannel _channel;

    public GrpcSagaClient(string endpoint)
    {
        ArgumentException.ThrowIfNullOrEmpty(endpoint);
        _channel = GrpcChannel.ForAddress(endpoint);
    }

    public Task<StartSagaResponse> StartSagaAsync(StartSagaRequest request, CancellationToken ct = default)
    {
        throw new NotImplementedException(
            "gRPC saga client requires proto-generated code. Use SagaProtocol.Http or provide proto files.");
    }

    public Task<SagaState> GetSagaAsync(string sagaId, CancellationToken ct = default)
    {
        throw new NotImplementedException(
            "gRPC saga client requires proto-generated code. Use SagaProtocol.Http or provide proto files.");
    }

    public Task CancelSagaAsync(string sagaId, CancellationToken ct = default)
    {
        throw new NotImplementedException(
            "gRPC saga client requires proto-generated code. Use SagaProtocol.Http or provide proto files.");
    }

    public async ValueTask DisposeAsync()
    {
        _channel.Dispose();
        await Task.CompletedTask;
    }
}
