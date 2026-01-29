using System.Diagnostics;

namespace K1s0.Observability;

/// <summary>
/// Provides a shared <see cref="ActivitySource"/> for k1s0 services.
/// </summary>
public static class K1s0ActivitySource
{
    /// <summary>
    /// The name used for the activity source.
    /// </summary>
    public const string Name = "k1s0";

    /// <summary>
    /// The shared <see cref="ActivitySource"/> instance.
    /// </summary>
    public static ActivitySource Instance { get; } = new(Name, "1.0.0");
}
