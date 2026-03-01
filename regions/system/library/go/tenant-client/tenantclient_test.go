package tenantclient_test

import (
	"context"
	"testing"
	"time"

	tenantclient "github.com/k1s0-platform/system-library-go-tenant-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func makeTenant(id string, status tenantclient.TenantStatus, plan string) tenantclient.Tenant {
	return tenantclient.Tenant{
		ID:        id,
		Name:      "Tenant " + id,
		Status:    status,
		Plan:      plan,
		Settings:  map[string]string{"max_users": "100"},
		CreatedAt: time.Now(),
	}
}

func TestGetTenant_Found(t *testing.T) {
	c := tenantclient.NewInMemoryTenantClient()
	c.AddTenant(makeTenant("T-001", tenantclient.TenantStatusActive, "enterprise"))
	tenant, err := c.GetTenant(context.Background(), "T-001")
	require.NoError(t, err)
	assert.Equal(t, "T-001", tenant.ID)
	assert.Equal(t, tenantclient.TenantStatusActive, tenant.Status)
}

func TestGetTenant_NotFound(t *testing.T) {
	c := tenantclient.NewInMemoryTenantClient()
	_, err := c.GetTenant(context.Background(), "T-999")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "tenant not found")
}

func TestListTenants_FilterByStatus(t *testing.T) {
	c := tenantclient.NewInMemoryTenantClientWithTenants([]tenantclient.Tenant{
		makeTenant("T-001", tenantclient.TenantStatusActive, "enterprise"),
		makeTenant("T-002", tenantclient.TenantStatusSuspended, "basic"),
		makeTenant("T-003", tenantclient.TenantStatusActive, "basic"),
	})
	status := tenantclient.TenantStatusActive
	tenants, err := c.ListTenants(context.Background(), tenantclient.TenantFilter{Status: &status})
	require.NoError(t, err)
	assert.Len(t, tenants, 2)
}

func TestListTenants_FilterByPlan(t *testing.T) {
	c := tenantclient.NewInMemoryTenantClientWithTenants([]tenantclient.Tenant{
		makeTenant("T-001", tenantclient.TenantStatusActive, "enterprise"),
		makeTenant("T-002", tenantclient.TenantStatusActive, "basic"),
	})
	plan := "enterprise"
	tenants, err := c.ListTenants(context.Background(), tenantclient.TenantFilter{Plan: &plan})
	require.NoError(t, err)
	assert.Len(t, tenants, 1)
	assert.Equal(t, "T-001", tenants[0].ID)
}

func TestListTenants_NoFilter(t *testing.T) {
	c := tenantclient.NewInMemoryTenantClientWithTenants([]tenantclient.Tenant{
		makeTenant("T-001", tenantclient.TenantStatusActive, "enterprise"),
		makeTenant("T-002", tenantclient.TenantStatusSuspended, "basic"),
	})
	tenants, err := c.ListTenants(context.Background(), tenantclient.TenantFilter{})
	require.NoError(t, err)
	assert.Len(t, tenants, 2)
}

func TestIsActive_True(t *testing.T) {
	c := tenantclient.NewInMemoryTenantClient()
	c.AddTenant(makeTenant("T-001", tenantclient.TenantStatusActive, "basic"))
	active, err := c.IsActive(context.Background(), "T-001")
	require.NoError(t, err)
	assert.True(t, active)
}

func TestIsActive_False(t *testing.T) {
	c := tenantclient.NewInMemoryTenantClient()
	c.AddTenant(makeTenant("T-001", tenantclient.TenantStatusSuspended, "basic"))
	active, err := c.IsActive(context.Background(), "T-001")
	require.NoError(t, err)
	assert.False(t, active)
}

func TestGetSettings(t *testing.T) {
	c := tenantclient.NewInMemoryTenantClient()
	c.AddTenant(makeTenant("T-001", tenantclient.TenantStatusActive, "basic"))
	settings, err := c.GetSettings(context.Background(), "T-001")
	require.NoError(t, err)
	v, ok := settings.Get("max_users")
	assert.True(t, ok)
	assert.Equal(t, "100", v)
	_, ok = settings.Get("nonexistent")
	assert.False(t, ok)
}

func TestTenantSettings_Get(t *testing.T) {
	s := tenantclient.TenantSettings{Values: map[string]string{"key": "val"}}
	v, ok := s.Get("key")
	assert.True(t, ok)
	assert.Equal(t, "val", v)
}

func TestInMemoryTenantClient_CreateTenant(t *testing.T) {
	client := tenantclient.NewInMemoryTenantClient()
	tenant, err := client.CreateTenant(context.Background(), tenantclient.CreateTenantRequest{
		Name: "Test Corp",
		Plan: "enterprise",
	})
	assert.NoError(t, err)
	assert.Equal(t, "Test Corp", tenant.Name)
	assert.Equal(t, tenantclient.TenantStatusActive, tenant.Status)
}

func TestInMemoryTenantClient_MemberManagement(t *testing.T) {
	client := tenantclient.NewInMemoryTenantClient()
	tenant, _ := client.CreateTenant(context.Background(), tenantclient.CreateTenantRequest{Name: "T1", Plan: "pro"})

	_, err := client.AddMember(context.Background(), tenant.ID, "user-1", "admin")
	assert.NoError(t, err)
	_, err = client.AddMember(context.Background(), tenant.ID, "user-2", "member")
	assert.NoError(t, err)

	members, err := client.ListMembers(context.Background(), tenant.ID)
	assert.NoError(t, err)
	assert.Len(t, members, 2)

	err = client.RemoveMember(context.Background(), tenant.ID, "user-1")
	assert.NoError(t, err)
	members, _ = client.ListMembers(context.Background(), tenant.ID)
	assert.Len(t, members, 1)
	assert.Equal(t, "user-2", members[0].UserID)
}

func TestInMemoryTenantClient_ProvisioningStatus(t *testing.T) {
	client := tenantclient.NewInMemoryTenantClient()
	tenant, _ := client.CreateTenant(context.Background(), tenantclient.CreateTenantRequest{Name: "T2", Plan: "starter"})

	status, err := client.GetProvisioningStatus(context.Background(), tenant.ID)
	assert.NoError(t, err)
	assert.Equal(t, tenantclient.ProvisioningStatusPending, status)
}
