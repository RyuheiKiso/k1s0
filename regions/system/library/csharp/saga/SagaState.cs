namespace K1s0.System.Saga;

public sealed record SagaState(
    string SagaId,
    string WorkflowName,
    string CurrentStep,
    SagaStatus Status,
    string Payload,
    string CorrelationId,
    DateTimeOffset CreatedAt,
    DateTimeOffset UpdatedAt,
    IReadOnlyList<SagaStepLog> StepLogs);
