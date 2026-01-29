using K1s0.Grpc.Server.Interceptors;
using Microsoft.Extensions.DependencyInjection;

namespace K1s0.Grpc.Server;

/// <summary>
/// Extension methods for configuring a k1s0 gRPC server.
/// </summary>
public static class GrpcServerExtensions
{
    /// <summary>
    /// Adds k1s0 gRPC server services with standard interceptors for error handling and tracing.
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddK1s0GrpcServer(this IServiceCollection services)
    {
        ArgumentNullException.ThrowIfNull(services);

        services.AddGrpc(options =>
        {
            options.Interceptors.Add<ErrorHandlingInterceptor>();
            options.Interceptors.Add<TracingInterceptor>();
            options.EnableDetailedErrors = false;
            options.MaxReceiveMessageSize = 4 * 1024 * 1024; // 4 MB
        });

        return services;
    }
}
