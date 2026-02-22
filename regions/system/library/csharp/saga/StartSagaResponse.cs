namespace K1s0.System.Saga;

public sealed record StartSagaResponse(
    string SagaId,
    SagaStatus Status);
