using Microsoft.Extensions.DependencyInjection;

namespace K1s0.System.Saga;

public static class SagaExtensions
{
    public static IServiceCollection AddK1s0Saga(
        this IServiceCollection services,
        SagaConfig config)
    {
        ArgumentNullException.ThrowIfNull(config);

        services.AddSingleton(config);

        switch (config.Protocol)
        {
            case SagaProtocol.Http:
                services.AddHttpClient<ISagaClient, HttpSagaClient>(client =>
                {
                    client.BaseAddress = new Uri(config.RestBaseUrl.TrimEnd('/') + "/");
                    client.Timeout = TimeSpan.FromSeconds(config.TimeoutSeconds);
                });
                break;

            case SagaProtocol.Grpc:
                if (string.IsNullOrEmpty(config.GrpcEndpoint))
                {
                    throw new SagaException(
                        SagaErrorCodes.InvalidStatus,
                        "GrpcEndpoint is required when Protocol is Grpc");
                }

                services.AddSingleton<ISagaClient>(new GrpcSagaClient(config.GrpcEndpoint));
                break;

            default:
                throw new SagaException(
                    SagaErrorCodes.InvalidStatus,
                    $"Unsupported protocol: {config.Protocol}");
        }

        return services;
    }
}
