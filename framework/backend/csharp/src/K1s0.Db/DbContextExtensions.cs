using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;

namespace K1s0.Db;

/// <summary>
/// Extension methods for registering a k1s0 <see cref="DbContext"/> with PostgreSQL.
/// </summary>
public static class DbContextExtensions
{
    /// <summary>
    /// Registers a <see cref="DbContext"/> of type <typeparamref name="TContext"/> using
    /// the connection string from the configuration key "database:connection_string".
    /// </summary>
    /// <typeparam name="TContext">The DbContext type to register.</typeparam>
    /// <param name="services">The service collection.</param>
    /// <param name="configuration">The application configuration.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddK1s0DbContext<TContext>(
        this IServiceCollection services,
        IConfiguration configuration)
        where TContext : DbContext
    {
        ArgumentNullException.ThrowIfNull(services);
        ArgumentNullException.ThrowIfNull(configuration);

        string connectionString = configuration["database:connection_string"]
            ?? throw new InvalidOperationException(
                "Missing configuration key 'database:connection_string'. " +
                "Ensure your config/default.yaml contains a 'database.connection_string' entry.");

        services.AddDbContext<TContext>(options =>
        {
            options.UseNpgsql(connectionString, npgsql =>
            {
                npgsql.CommandTimeout(30);
                npgsql.EnableRetryOnFailure(3);
            });
        });

        services.AddScoped<IUnitOfWork, UnitOfWork<TContext>>();

        return services;
    }
}
