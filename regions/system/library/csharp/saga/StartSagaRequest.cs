namespace K1s0.System.Saga;

public sealed record StartSagaRequest(
    string WorkflowName,
    string Payload,
    string CorrelationId);
