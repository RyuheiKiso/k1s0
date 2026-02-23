using K1s0.System.TenantClient;

namespace K1s0.System.TenantClient.Tests;

public class InMemoryTenantClientTests
{
    private static Tenant MakeTenant(string id, TenantStatus status = TenantStatus.Active, string plan = "basic")
    {
        return new Tenant(id, $"Tenant {id}", status, plan,
            new Dictionary<string, string> { { "max_users", "100" } },
            DateTimeOffset.UtcNow);
    }

    [Fact]
    public async Task GetTenant_Found()
    {
        var client = new InMemoryTenantClient();
        client.AddTenant(MakeTenant("T-001"));
        var tenant = await client.GetTenantAsync("T-001");
        Assert.Equal("T-001", tenant.Id);
        Assert.Equal(TenantStatus.Active, tenant.Status);
    }

    [Fact]
    public async Task GetTenant_NotFound_ThrowsException()
    {
        var client = new InMemoryTenantClient();
        var ex = await Assert.ThrowsAsync<TenantException>(() => client.GetTenantAsync("T-999"));
        Assert.Equal(TenantErrorCode.NotFound, ex.Code);
    }

    [Fact]
    public async Task ListTenants_FilterByStatus()
    {
        var client = new InMemoryTenantClient();
        client.AddTenant(MakeTenant("T-001", TenantStatus.Active));
        client.AddTenant(MakeTenant("T-002", TenantStatus.Suspended));
        client.AddTenant(MakeTenant("T-003", TenantStatus.Active));

        var tenants = await client.ListTenantsAsync(new TenantFilter(Status: TenantStatus.Active));
        Assert.Equal(2, tenants.Count);
    }

    [Fact]
    public async Task ListTenants_FilterByPlan()
    {
        var client = new InMemoryTenantClient();
        client.AddTenant(MakeTenant("T-001", plan: "enterprise"));
        client.AddTenant(MakeTenant("T-002", plan: "basic"));

        var tenants = await client.ListTenantsAsync(new TenantFilter(Plan: "enterprise"));
        Assert.Single(tenants);
        Assert.Equal("T-001", tenants[0].Id);
    }

    [Fact]
    public async Task ListTenants_NoFilter_ReturnsAll()
    {
        var client = new InMemoryTenantClient();
        client.AddTenant(MakeTenant("T-001"));
        client.AddTenant(MakeTenant("T-002"));

        var tenants = await client.ListTenantsAsync();
        Assert.Equal(2, tenants.Count);
    }

    [Fact]
    public async Task IsActive_True()
    {
        var client = new InMemoryTenantClient();
        client.AddTenant(MakeTenant("T-001", TenantStatus.Active));
        Assert.True(await client.IsActiveAsync("T-001"));
    }

    [Fact]
    public async Task IsActive_False()
    {
        var client = new InMemoryTenantClient();
        client.AddTenant(MakeTenant("T-001", TenantStatus.Suspended));
        Assert.False(await client.IsActiveAsync("T-001"));
    }

    [Fact]
    public async Task GetSettings_ReturnsValues()
    {
        var client = new InMemoryTenantClient();
        client.AddTenant(MakeTenant("T-001"));
        var settings = await client.GetSettingsAsync("T-001");
        Assert.Equal("100", settings.Get("max_users"));
        Assert.Null(settings.Get("nonexistent"));
    }

    [Fact]
    public void TenantException_ContainsCode()
    {
        var ex = new TenantException("not found", TenantErrorCode.NotFound);
        Assert.Equal(TenantErrorCode.NotFound, ex.Code);
        Assert.Equal("not found", ex.Message);
    }

    [Fact]
    public void Tenants_InitiallyEmpty()
    {
        var client = new InMemoryTenantClient();
        Assert.Empty(client.Tenants);
    }
}
