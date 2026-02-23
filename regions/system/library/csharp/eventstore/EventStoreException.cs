namespace K1s0.System.EventStore;

public class VersionConflictException(long expected, long actual)
    : Exception($"Version conflict: expected {expected}, actual {actual}")
{
    public long Expected { get; } = expected;

    public long Actual { get; } = actual;
}
