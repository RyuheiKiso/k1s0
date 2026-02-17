import { describe, it, expect } from 'vitest';
import { mergeVaultSecrets } from '../src/index.js';
import type { Config } from '../src/index.js';

function minimalConfig(overrides?: Partial<Config>): Config {
  return {
    app: {
      name: 'test',
      version: '1.0',
      tier: 'system',
      environment: 'dev',
    },
    server: { host: '0.0.0.0', port: 8080 },
    observability: {
      log: { level: 'info', format: 'json' },
      trace: { enabled: false },
      metrics: { enabled: false },
    },
    auth: {
      jwt: { issuer: 'http://localhost', audience: 'test' },
    },
    ...overrides,
  } as Config;
}

describe('mergeVaultSecrets', () => {
  it('should merge database.password', () => {
    const cfg = minimalConfig({
      database: {
        host: 'localhost',
        port: 5432,
        name: 'test_db',
        user: 'app',
        password: 'old',
      },
    });
    const merged = mergeVaultSecrets(cfg, {
      'database.password': 'vault-db-pass',
    });
    expect(merged.database?.password).toBe('vault-db-pass');
  });

  it('should merge redis.password', () => {
    const cfg = minimalConfig({
      redis: { host: 'localhost', port: 6379, password: 'old' },
    });
    const merged = mergeVaultSecrets(cfg, {
      'redis.password': 'vault-redis-pass',
    });
    expect(merged.redis?.password).toBe('vault-redis-pass');
  });

  it('should merge kafka.sasl credentials', () => {
    const cfg = minimalConfig({
      kafka: {
        brokers: ['localhost:9092'],
        consumer_group: 'test.default',
        security_protocol: 'SASL_SSL',
        sasl: {
          mechanism: 'SCRAM-SHA-512',
          username: '',
          password: '',
        },
        topics: { publish: [], subscribe: [] },
      },
    });
    const merged = mergeVaultSecrets(cfg, {
      'kafka.sasl.username': 'vault-kafka-user',
      'kafka.sasl.password': 'vault-kafka-pass',
    });
    expect(merged.kafka?.sasl?.username).toBe('vault-kafka-user');
    expect(merged.kafka?.sasl?.password).toBe('vault-kafka-pass');
  });

  it('should merge redis_session.password', () => {
    const cfg = minimalConfig({
      redis_session: { host: 'localhost', port: 6380, password: '' },
    });
    const merged = mergeVaultSecrets(cfg, {
      'redis_session.password': 'vault-session-pass',
    });
    expect(merged.redis_session?.password).toBe('vault-session-pass');
  });

  it('should merge oidc.client_secret', () => {
    const cfg = minimalConfig({
      auth: {
        jwt: { issuer: 'http://localhost', audience: 'test' },
        oidc: {
          discovery_url: 'http://localhost/.well-known',
          client_id: 'test',
          redirect_uri: 'http://localhost/callback',
          scopes: ['openid'],
          jwks_uri: 'http://localhost/jwks',
        },
      },
    });
    const merged = mergeVaultSecrets(cfg, {
      'auth.oidc.client_secret': 'vault-oidc-secret',
    });
    expect(merged.auth.oidc?.client_secret).toBe('vault-oidc-secret');
  });

  it('should not change config when secrets are empty', () => {
    const cfg = minimalConfig({
      database: {
        host: 'localhost',
        port: 5432,
        name: 'test_db',
        user: 'app',
        password: 'original',
      },
      redis: { host: 'localhost', port: 6379, password: 'original' },
    });
    const merged = mergeVaultSecrets(cfg, {});
    expect(merged.database?.password).toBe('original');
    expect(merged.redis?.password).toBe('original');
  });

  it('should handle nil/undefined sections safely', () => {
    const cfg = minimalConfig();
    // No database, redis, kafka, redis_session, or oidc sections
    const merged = mergeVaultSecrets(cfg, {
      'database.password': 'secret',
      'redis.password': 'secret',
      'kafka.sasl.username': 'user',
      'kafka.sasl.password': 'pass',
      'redis_session.password': 'secret',
      'auth.oidc.client_secret': 'secret',
    });
    expect(merged.database).toBeUndefined();
    expect(merged.redis).toBeUndefined();
    expect(merged.kafka).toBeUndefined();
    expect(merged.redis_session).toBeUndefined();
    expect(merged.auth.oidc).toBeUndefined();
  });

  it('should merge only existing partial secrets', () => {
    const cfg = minimalConfig({
      database: {
        host: 'localhost',
        port: 5432,
        name: 'test_db',
        user: 'app',
        password: 'old-db',
      },
      redis: { host: 'localhost', port: 6379, password: 'old-redis' },
      auth: {
        jwt: { issuer: 'http://localhost', audience: 'test' },
        oidc: {
          discovery_url: 'http://localhost/.well-known',
          client_id: 'test',
          client_secret: 'old-oidc',
          redirect_uri: 'http://localhost/callback',
          scopes: ['openid'],
          jwks_uri: 'http://localhost/jwks',
        },
      },
    });
    // Only database.password is provided
    const merged = mergeVaultSecrets(cfg, {
      'database.password': 'new-db',
    });
    expect(merged.database?.password).toBe('new-db');
    expect(merged.redis?.password).toBe('old-redis');
    expect(merged.auth.oidc?.client_secret).toBe('old-oidc');
  });
});
