namespace K1s0.System.TenantClient;

public interface ITenantClient
{
    Task<Tenant> GetTenantAsync(string tenantId, CancellationToken ct = default);

    Task<IReadOnlyList<Tenant>> ListTenantsAsync(TenantFilter? filter = null, CancellationToken ct = default);

    Task<bool> IsActiveAsync(string tenantId, CancellationToken ct = default);

    Task<TenantSettings> GetSettingsAsync(string tenantId, CancellationToken ct = default);
}
