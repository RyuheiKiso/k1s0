import { describe, it, expect } from 'vitest';
import { InMemoryTenantClient, TenantError } from '../src/index.js';
import type { Tenant } from '../src/index.js';

function makeTenant(
  id: string,
  status: 'active' | 'suspended' | 'deleted' = 'active',
  plan = 'basic',
): Tenant {
  return {
    id,
    name: `Tenant ${id}`,
    status,
    plan,
    settings: { max_users: '100' },
    createdAt: new Date(),
  };
}

describe('InMemoryTenantClient', () => {
  it('テナントを取得できる', async () => {
    const client = new InMemoryTenantClient([makeTenant('T-001')]);
    const tenant = await client.getTenant('T-001');
    expect(tenant.id).toBe('T-001');
    expect(tenant.status).toBe('active');
  });

  it('存在しないテナントでエラーを返す', async () => {
    const client = new InMemoryTenantClient();
    await expect(client.getTenant('T-999')).rejects.toThrow(TenantError);
  });

  it('ステータスでフィルターできる', async () => {
    const client = new InMemoryTenantClient([
      makeTenant('T-001', 'active'),
      makeTenant('T-002', 'suspended'),
      makeTenant('T-003', 'active'),
    ]);
    const tenants = await client.listTenants({ status: 'active' });
    expect(tenants).toHaveLength(2);
  });

  it('プランでフィルターできる', async () => {
    const client = new InMemoryTenantClient([
      makeTenant('T-001', 'active', 'enterprise'),
      makeTenant('T-002', 'active', 'basic'),
    ]);
    const tenants = await client.listTenants({ plan: 'enterprise' });
    expect(tenants).toHaveLength(1);
    expect(tenants[0].id).toBe('T-001');
  });

  it('フィルターなしで全件返す', async () => {
    const client = new InMemoryTenantClient([makeTenant('T-001'), makeTenant('T-002')]);
    const tenants = await client.listTenants();
    expect(tenants).toHaveLength(2);
  });

  it('アクティブテナントをチェックできる', async () => {
    const client = new InMemoryTenantClient([makeTenant('T-001', 'active')]);
    expect(await client.isActive('T-001')).toBe(true);
  });

  it('非アクティブテナントを検出できる', async () => {
    const client = new InMemoryTenantClient([makeTenant('T-001', 'suspended')]);
    expect(await client.isActive('T-001')).toBe(false);
  });

  it('テナント設定を取得できる', async () => {
    const client = new InMemoryTenantClient([makeTenant('T-001')]);
    const settings = await client.getSettings('T-001');
    expect(settings.get('max_users')).toBe('100');
    expect(settings.get('nonexistent')).toBeUndefined();
  });

  it('TenantErrorにコードが含まれる', () => {
    const error = new TenantError('not found', 'NOT_FOUND');
    expect(error.code).toBe('NOT_FOUND');
    expect(error.message).toBe('not found');
  });

  it('addTenantでテナントを追加できる', async () => {
    const client = new InMemoryTenantClient();
    client.addTenant(makeTenant('T-001'));
    const tenant = await client.getTenant('T-001');
    expect(tenant.id).toBe('T-001');
  });

  it('createTenant creates a new active tenant', async () => {
    const client = new InMemoryTenantClient();
    const tenant = await client.createTenant({ name: 'Test Corp', plan: 'enterprise' });
    expect(tenant.name).toBe('Test Corp');
    expect(tenant.status).toBe('active');
    expect(tenant.plan).toBe('enterprise');
  });

  it('addMember and listMembers work correctly', async () => {
    const client = new InMemoryTenantClient();
    const tenant = await client.createTenant({ name: 'T1', plan: 'pro' });
    await client.addMember(tenant.id, 'user-1', 'admin');
    await client.addMember(tenant.id, 'user-2', 'member');
    const members = await client.listMembers(tenant.id);
    expect(members).toHaveLength(2);
    await client.removeMember(tenant.id, 'user-1');
    const updated = await client.listMembers(tenant.id);
    expect(updated).toHaveLength(1);
    expect(updated[0].userId).toBe('user-2');
  });

  it('getProvisioningStatus returns pending after creation', async () => {
    const client = new InMemoryTenantClient();
    const tenant = await client.createTenant({ name: 'T2', plan: 'starter' });
    const status = await client.getProvisioningStatus(tenant.id);
    expect(status).toBe('pending');
  });
});
