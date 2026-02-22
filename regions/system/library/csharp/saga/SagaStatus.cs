namespace K1s0.System.Saga;

public enum SagaStatus
{
    Started,
    Running,
    Completed,
    Compensating,
    Failed,
    Cancelled,
}
