export type TenantStatus = 'active' | 'suspended' | 'deleted';

export interface Tenant {
  id: string;
  name: string;
  status: TenantStatus;
  plan: string;
  settings: Record<string, string>;
  createdAt: Date;
}

export interface TenantFilter {
  status?: TenantStatus;
  plan?: string;
}

export interface TenantSettings {
  values: Record<string, string>;
  get(key: string): string | undefined;
}

export interface TenantClientConfig {
  serverUrl: string;
  cacheTtlMs?: number;
  cacheMaxCapacity?: number;
}

export interface CreateTenantRequest {
  name: string;
  plan: string;
  adminUserId?: string;
}

export interface TenantMember {
  userId: string;
  role: string;
  joinedAt: Date;
}

export type ProvisioningStatus = 'pending' | 'in_progress' | 'completed' | 'failed';

export class TenantError extends Error {
  constructor(
    message: string,
    public readonly code: 'NOT_FOUND' | 'SUSPENDED' | 'SERVER_ERROR' | 'TIMEOUT',
  ) {
    super(message);
    this.name = 'TenantError';
  }
}

export interface TenantClient {
  getTenant(tenantId: string): Promise<Tenant>;
  listTenants(filter?: TenantFilter): Promise<Tenant[]>;
  isActive(tenantId: string): Promise<boolean>;
  getSettings(tenantId: string): Promise<TenantSettings>;
  createTenant(req: CreateTenantRequest): Promise<Tenant>;
  addMember(tenantId: string, userId: string, role: string): Promise<TenantMember>;
  removeMember(tenantId: string, userId: string): Promise<void>;
  listMembers(tenantId: string): Promise<TenantMember[]>;
  getProvisioningStatus(tenantId: string): Promise<ProvisioningStatus>;
}

function createTenantSettings(values: Record<string, string>): TenantSettings {
  return {
    values,
    get(key: string): string | undefined {
      return values[key];
    },
  };
}

export class InMemoryTenantClient implements TenantClient {
  private tenants = new Map<string, Tenant>();
  private members = new Map<string, TenantMember[]>();
  private provisioning = new Map<string, ProvisioningStatus>();

  constructor(tenants: Tenant[] = []) {
    for (const tenant of tenants) {
      this.tenants.set(tenant.id, tenant);
    }
  }

  addTenant(tenant: Tenant): void {
    this.tenants.set(tenant.id, tenant);
  }

  async getTenant(tenantId: string): Promise<Tenant> {
    const tenant = this.tenants.get(tenantId);
    if (!tenant) {
      throw new TenantError(`Tenant not found: ${tenantId}`, 'NOT_FOUND');
    }
    return tenant;
  }

  async listTenants(filter?: TenantFilter): Promise<Tenant[]> {
    let list = Array.from(this.tenants.values());
    if (filter?.status) list = list.filter((t) => t.status === filter.status);
    if (filter?.plan) list = list.filter((t) => t.plan === filter.plan);
    return list;
  }

  async isActive(tenantId: string): Promise<boolean> {
    const tenant = this.tenants.get(tenantId);
    return tenant?.status === 'active';
  }

  async getSettings(tenantId: string): Promise<TenantSettings> {
    const tenant = await this.getTenant(tenantId);
    return createTenantSettings(tenant.settings);
  }

  async createTenant(req: CreateTenantRequest): Promise<Tenant> {
    const id = `tenant-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
    const tenant: Tenant = {
      id,
      name: req.name,
      status: 'active',
      plan: req.plan,
      settings: {},
      createdAt: new Date(),
    };
    this.tenants.set(id, tenant);
    this.provisioning.set(id, 'pending');
    return tenant;
  }

  async addMember(tenantId: string, userId: string, role: string): Promise<TenantMember> {
    if (!this.tenants.has(tenantId)) {
      throw new TenantError(`Tenant not found: ${tenantId}`, 'NOT_FOUND');
    }
    const member: TenantMember = { userId, role, joinedAt: new Date() };
    const list = this.members.get(tenantId) ?? [];
    list.push(member);
    this.members.set(tenantId, list);
    return member;
  }

  async removeMember(tenantId: string, userId: string): Promise<void> {
    const list = this.members.get(tenantId) ?? [];
    this.members.set(
      tenantId,
      list.filter((m) => m.userId !== userId),
    );
  }

  async listMembers(tenantId: string): Promise<TenantMember[]> {
    return this.members.get(tenantId) ?? [];
  }

  async getProvisioningStatus(tenantId: string): Promise<ProvisioningStatus> {
    const status = this.provisioning.get(tenantId);
    if (!status) throw new TenantError(`Provisioning status not found: ${tenantId}`, 'NOT_FOUND');
    return status;
  }
}

export class GrpcTenantClient implements TenantClient {
  private readonly config: TenantClientConfig;

  constructor(config: TenantClientConfig) {
    this.config = config;
  }

  async getTenant(_tenantId: string): Promise<Tenant> {
    throw new TenantError('gRPC client not yet connected', 'SERVER_ERROR');
  }

  async listTenants(_filter?: TenantFilter): Promise<Tenant[]> {
    throw new TenantError('gRPC client not yet connected', 'SERVER_ERROR');
  }

  async isActive(_tenantId: string): Promise<boolean> {
    throw new TenantError('gRPC client not yet connected', 'SERVER_ERROR');
  }

  async getSettings(_tenantId: string): Promise<TenantSettings> {
    throw new TenantError('gRPC client not yet connected', 'SERVER_ERROR');
  }

  async createTenant(_req: CreateTenantRequest): Promise<Tenant> {
    throw new Error('Not yet implemented');
  }

  async addMember(_tenantId: string, _userId: string, _role: string): Promise<TenantMember> {
    throw new Error('Not yet implemented');
  }

  async removeMember(_tenantId: string, _userId: string): Promise<void> {
    throw new Error('Not yet implemented');
  }

  async listMembers(_tenantId: string): Promise<TenantMember[]> {
    throw new Error('Not yet implemented');
  }

  async getProvisioningStatus(_tenantId: string): Promise<ProvisioningStatus> {
    throw new Error('Not yet implemented');
  }

  async close(): Promise<void> {
    // 接続クリーンアップ用プレースホルダー
  }
}
