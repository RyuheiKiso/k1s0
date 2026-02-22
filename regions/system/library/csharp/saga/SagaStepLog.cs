namespace K1s0.System.Saga;

public sealed record SagaStepLog(
    string StepName,
    string Status,
    DateTimeOffset StartedAt,
    DateTimeOffset? CompletedAt,
    string? Error);
