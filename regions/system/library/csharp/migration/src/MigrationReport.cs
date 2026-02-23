namespace K1s0.System.Migration;

public record MigrationReport(
    int AppliedCount,
    TimeSpan Elapsed,
    IReadOnlyList<string> Errors);
