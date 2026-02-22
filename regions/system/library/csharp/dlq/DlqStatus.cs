namespace K1s0.System.Dlq;

public enum DlqStatus
{
    Pending,
    Retrying,
    Resolved,
    Dead,
}
