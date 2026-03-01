export interface Secret {
  path: string;
  data: Record<string, string>;
  version: number;
  createdAt: Date;
}

export interface SecretRotatedEvent {
  path: string;
  version: number;
}

export interface VaultClientConfig {
  serverUrl: string;
  cacheTtlMs?: number;
  cacheMaxCapacity?: number;
}

export type VaultErrorCode =
  | 'NOT_FOUND'
  | 'PERMISSION_DENIED'
  | 'SERVER_ERROR'
  | 'TIMEOUT'
  | 'LEASE_EXPIRED';

export class VaultError extends Error {
  constructor(
    message: string,
    public readonly code: VaultErrorCode,
  ) {
    super(message);
    this.name = 'VaultError';
  }
}

export interface VaultClient {
  getSecret(path: string): Promise<Secret>;
  getSecretValue(path: string, key: string): Promise<string>;
  listSecrets(pathPrefix: string): Promise<string[]>;
  watchSecret(path: string): AsyncIterable<SecretRotatedEvent>;
}

export class InMemoryVaultClient implements VaultClient {
  private store = new Map<string, Secret>();
  private readonly config: VaultClientConfig;

  constructor(config: VaultClientConfig) {
    this.config = config;
  }

  getConfig(): VaultClientConfig {
    return this.config;
  }

  putSecret(secret: Secret): void {
    this.store.set(secret.path, secret);
  }

  async getSecret(path: string): Promise<Secret> {
    const secret = this.store.get(path);
    if (!secret) {
      throw new VaultError(path, 'NOT_FOUND');
    }
    return secret;
  }

  async getSecretValue(path: string, key: string): Promise<string> {
    const secret = await this.getSecret(path);
    const value = secret.data[key];
    if (value === undefined) {
      throw new VaultError(`${path}/${key}`, 'NOT_FOUND');
    }
    return value;
  }

  async listSecrets(pathPrefix: string): Promise<string[]> {
    const paths: string[] = [];
    for (const key of this.store.keys()) {
      if (key.startsWith(pathPrefix)) {
        paths.push(key);
      }
    }
    return paths;
  }

  async *watchSecret(_path: string): AsyncIterable<SecretRotatedEvent> {
    // InMemory implementation yields nothing
  }
}

export class HttpVaultClient implements VaultClient {
  private readonly config: VaultClientConfig;
  private readonly cache = new Map<string, { secret: Secret; fetchedAt: number }>();

  constructor(config: VaultClientConfig) {
    this.config = config;
  }

  private isCacheValid(fetchedAt: number): boolean {
    const ttl = this.config.cacheTtlMs ?? 600_000;
    return Date.now() - fetchedAt < ttl;
  }

  async getSecret(path: string): Promise<Secret> {
    const cached = this.cache.get(path);
    if (cached && this.isCacheValid(cached.fetchedAt)) {
      return cached.secret;
    }

    const url = `${this.config.serverUrl}/api/v1/secrets/${path}`;
    const resp = await fetch(url);

    if (resp.status === 404) throw new VaultError(path, 'NOT_FOUND');
    if (resp.status === 403 || resp.status === 401)
      throw new VaultError(path, 'PERMISSION_DENIED');
    if (!resp.ok) throw new VaultError(`HTTP ${resp.status}`, 'SERVER_ERROR');

    const body = await resp.json();
    const secret: Secret = {
      path: body.path,
      data: body.data,
      version: body.version,
      createdAt: new Date(body.created_at),
    };
    this.cache.set(path, { secret, fetchedAt: Date.now() });
    return secret;
  }

  async getSecretValue(path: string, key: string): Promise<string> {
    const secret = await this.getSecret(path);
    const value = secret.data[key];
    if (value === undefined) throw new VaultError(`${path}/${key}`, 'NOT_FOUND');
    return value;
  }

  async listSecrets(pathPrefix: string): Promise<string[]> {
    const url = `${this.config.serverUrl}/api/v1/secrets?prefix=${encodeURIComponent(pathPrefix)}`;
    const resp = await fetch(url);
    if (!resp.ok) throw new VaultError(`HTTP ${resp.status}`, 'SERVER_ERROR');
    return resp.json();
  }

  async *watchSecret(path: string): AsyncIterable<SecretRotatedEvent> {
    const ttl = this.config.cacheTtlMs ?? 600_000;
    let lastVersion: number | undefined;
    while (true) {
      await new Promise<void>((resolve) => setTimeout(resolve, ttl));
      try {
        const secret = await this.getSecret(path);
        if (lastVersion !== undefined && secret.version !== lastVersion) {
          yield { path, version: secret.version };
        }
        lastVersion = secret.version;
      } catch {
        // skip errors during polling
      }
    }
  }
}
