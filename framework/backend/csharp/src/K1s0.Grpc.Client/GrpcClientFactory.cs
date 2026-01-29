using Grpc.Net.Client;

namespace K1s0.Grpc.Client;

/// <summary>
/// Factory for creating gRPC channels with k1s0 default options.
/// </summary>
public static class GrpcClientFactory
{
    /// <summary>
    /// Default deadline for gRPC calls (30 seconds).
    /// </summary>
    public static readonly TimeSpan DefaultDeadline = TimeSpan.FromSeconds(30);

    /// <summary>
    /// Creates a <see cref="GrpcChannel"/> with k1s0 default options.
    /// </summary>
    /// <param name="address">The server address (e.g., "https://localhost:5001").</param>
    /// <param name="configure">Optional action to further configure channel options.</param>
    /// <returns>A configured <see cref="GrpcChannel"/>.</returns>
    public static GrpcChannel CreateChannel(string address, Action<GrpcChannelOptions>? configure = null)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(address);

        var options = new GrpcChannelOptions
        {
            MaxReceiveMessageSize = 4 * 1024 * 1024,   // 4 MB
            MaxSendMessageSize = 4 * 1024 * 1024,       // 4 MB
        };

        configure?.Invoke(options);

        return GrpcChannel.ForAddress(address, options);
    }
}
