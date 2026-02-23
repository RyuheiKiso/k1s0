namespace K1s0.System.TenantClient;

public class InMemoryTenantClient : ITenantClient
{
    private readonly List<Tenant> _tenants = new();

    public IReadOnlyList<Tenant> Tenants => _tenants.AsReadOnly();

    public void AddTenant(Tenant tenant) => _tenants.Add(tenant);

    public Task<Tenant> GetTenantAsync(string tenantId, CancellationToken ct = default)
    {
        var tenant = _tenants.Find(t => t.Id == tenantId);
        if (tenant is null)
        {
            throw new TenantException($"Tenant not found: {tenantId}", TenantErrorCode.NotFound);
        }

        return Task.FromResult(tenant);
    }

    public Task<IReadOnlyList<Tenant>> ListTenantsAsync(TenantFilter? filter = null, CancellationToken ct = default)
    {
        IEnumerable<Tenant> result = _tenants;
        if (filter?.Status is not null)
        {
            result = result.Where(t => t.Status == filter.Status);
        }

        if (filter?.Plan is not null)
        {
            result = result.Where(t => t.Plan == filter.Plan);
        }

        IReadOnlyList<Tenant> list = result.ToList().AsReadOnly();
        return Task.FromResult(list);
    }

    public async Task<bool> IsActiveAsync(string tenantId, CancellationToken ct = default)
    {
        var tenant = await GetTenantAsync(tenantId, ct);
        return tenant.Status == TenantStatus.Active;
    }

    public async Task<TenantSettings> GetSettingsAsync(string tenantId, CancellationToken ct = default)
    {
        var tenant = await GetTenantAsync(tenantId, ct);
        return new TenantSettings(tenant.Settings);
    }
}
