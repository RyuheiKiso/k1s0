using Grpc.Net.Client;

namespace K1s0.Grpc.Client;

/// <summary>
/// Extension methods for <see cref="GrpcChannel"/> configuration.
/// </summary>
public static class GrpcChannelExtensions
{
    /// <summary>
    /// Creates a call options with a default deadline from <see cref="GrpcClientFactory.DefaultDeadline"/>.
    /// </summary>
    /// <param name="channel">The gRPC channel (used for fluent API).</param>
    /// <param name="deadline">Optional custom deadline. If null, uses the default.</param>
    /// <returns>Configured call options.</returns>
    public static global::Grpc.Core.CallOptions CreateDefaultCallOptions(
        this GrpcChannel channel,
        TimeSpan? deadline = null)
    {
        ArgumentNullException.ThrowIfNull(channel);

        var effectiveDeadline = DateTime.UtcNow.Add(deadline ?? GrpcClientFactory.DefaultDeadline);
        return new global::Grpc.Core.CallOptions(deadline: effectiveDeadline);
    }
}
