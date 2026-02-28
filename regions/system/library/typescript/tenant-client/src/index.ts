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
  private tenants: Tenant[] = [];

  constructor(tenants: Tenant[] = []) {
    this.tenants = [...tenants];
  }

  addTenant(tenant: Tenant): void {
    this.tenants.push(tenant);
  }

  async getTenant(tenantId: string): Promise<Tenant> {
    const tenant = this.tenants.find((t) => t.id === tenantId);
    if (!tenant) {
      throw new TenantError(`Tenant not found: ${tenantId}`, 'NOT_FOUND');
    }
    return tenant;
  }

  async listTenants(filter?: TenantFilter): Promise<Tenant[]> {
    return this.tenants.filter((t) => {
      if (filter?.status && t.status !== filter.status) return false;
      if (filter?.plan && t.plan !== filter.plan) return false;
      return true;
    });
  }

  async isActive(tenantId: string): Promise<boolean> {
    const tenant = await this.getTenant(tenantId);
    return tenant.status === 'active';
  }

  async getSettings(tenantId: string): Promise<TenantSettings> {
    const tenant = await this.getTenant(tenantId);
    return createTenantSettings(tenant.settings);
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

  async close(): Promise<void> {
    // 接続クリーンアップ用プレースホルダー
  }
}
