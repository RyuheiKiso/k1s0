namespace K1s0.System.Saga;

public interface ISagaClient : IAsyncDisposable
{
    Task<StartSagaResponse> StartSagaAsync(StartSagaRequest request, CancellationToken ct = default);

    Task<SagaState> GetSagaAsync(string sagaId, CancellationToken ct = default);

    Task CancelSagaAsync(string sagaId, CancellationToken ct = default);
}
