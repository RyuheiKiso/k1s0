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
