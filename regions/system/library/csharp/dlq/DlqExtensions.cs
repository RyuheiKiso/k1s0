using Microsoft.Extensions.DependencyInjection;

namespace K1s0.System.Dlq;

public static class DlqExtensions
{
    public static IServiceCollection AddK1s0DlqClient(
        this IServiceCollection services,
        DlqConfig config)
    {
        ArgumentNullException.ThrowIfNull(config);

        services.AddHttpClient<IDlqClient, HttpDlqClient>(client =>
        {
            client.BaseAddress = new Uri(config.BaseUrl.TrimEnd('/') + "/");
            client.Timeout = TimeSpan.FromSeconds(config.TimeoutSeconds);
        });

        return services;
    }
}
