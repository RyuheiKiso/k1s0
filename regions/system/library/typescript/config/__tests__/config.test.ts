import { describe, it, expect } from 'vitest';
import { writeFileSync, mkdtempSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { load, validate, mergeVaultSecrets } from '../src/index.js';

const MINIMAL_CONFIG_YAML = `
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
`;

function writeConfig(dir: string, filename: string, content: string): string {
  const path = join(dir, filename);
  writeFileSync(path, content);
  return path;
}

describe('load', () => {
  it('should load a valid config', () => {
    const dir = mkdtempSync(join(tmpdir(), 'k1s0-'));
    const path = writeConfig(dir, 'config.yaml', MINIMAL_CONFIG_YAML);

    const cfg = load(path);
    expect(cfg.app.name).toBe('test-server');
    expect(cfg.server.port).toBe(8080);
  });

  it('should throw on file not found', () => {
    expect(() => load('/nonexistent/config.yaml')).toThrow();
  });

  it('should merge env override', () => {
    const dir = mkdtempSync(join(tmpdir(), 'k1s0-'));
    const basePath = writeConfig(dir, 'config.yaml', MINIMAL_CONFIG_YAML);
    const envPath = writeConfig(
      dir,
      'config.staging.yaml',
      `
app:
  environment: staging
server:
  port: 9090
observability:
  log:
    level: info
`,
    );

    const cfg = load(basePath, envPath);
    expect(cfg.app.environment).toBe('staging');
    expect(cfg.server.port).toBe(9090);
    expect(cfg.app.name).toBe('test-server'); // base value preserved
  });
});

describe('validate', () => {
  it('should pass for valid config', () => {
    const dir = mkdtempSync(join(tmpdir(), 'k1s0-'));
    const path = writeConfig(dir, 'config.yaml', MINIMAL_CONFIG_YAML);

    const cfg = load(path);
    expect(() => validate(cfg)).not.toThrow();
  });

  it('should reject empty app name', () => {
    const cfg = {
      app: { name: '', version: '1.0', tier: 'system', environment: 'dev' },
      server: { host: '0.0.0.0', port: 8080 },
      observability: {
        log: { level: 'info', format: 'json' },
        trace: { enabled: false },
        metrics: { enabled: false },
      },
      auth: {
        jwt: { issuer: 'http://localhost', audience: 'test' },
      },
    } as any;
    expect(() => validate(cfg)).toThrow();
  });

  it('should reject invalid tier', () => {
    const cfg = {
      app: {
        name: 'test',
        version: '1.0',
        tier: 'invalid',
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
    } as any;
    expect(() => validate(cfg)).toThrow();
  });

  it('should reject invalid port', () => {
    const cfg = {
      app: {
        name: 'test',
        version: '1.0',
        tier: 'system',
        environment: 'dev',
      },
      server: { host: '0.0.0.0', port: 0 },
      observability: {
        log: { level: 'info', format: 'json' },
        trace: { enabled: false },
        metrics: { enabled: false },
      },
      auth: {
        jwt: { issuer: 'http://localhost', audience: 'test' },
      },
    } as any;
    expect(() => validate(cfg)).toThrow();
  });
});

describe('mergeVaultSecrets', () => {
  it('should merge database password', () => {
    const dir = mkdtempSync(join(tmpdir(), 'k1s0-'));
    const path = writeConfig(
      dir,
      'config.yaml',
      `
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: localhost
  port: 5432
  name: test_db
  user: app
  password: ""
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
`,
    );

    const cfg = load(path);
    const merged = mergeVaultSecrets(cfg, {
      'database.password': 'secret123',
    });
    expect(merged.database?.password).toBe('secret123');
  });

  it('should merge redis password', () => {
    const dir = mkdtempSync(join(tmpdir(), 'k1s0-'));
    const path = writeConfig(
      dir,
      'config.yaml',
      `
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
redis:
  host: localhost
  port: 6379
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
`,
    );

    const cfg = load(path);
    const merged = mergeVaultSecrets(cfg, {
      'redis.password': 'redis-secret',
    });
    expect(merged.redis?.password).toBe('redis-secret');
  });

  it('should merge oidc client secret', () => {
    const dir = mkdtempSync(join(tmpdir(), 'k1s0-'));
    const path = writeConfig(
      dir,
      'config.yaml',
      `
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
  oidc:
    discovery_url: "http://localhost/.well-known"
    client_id: "test"
    redirect_uri: "http://localhost/callback"
    scopes: ["openid"]
    jwks_uri: "http://localhost/jwks"
`,
    );

    const cfg = load(path);
    const merged = mergeVaultSecrets(cfg, {
      'auth.oidc.client_secret': 'oidc-secret',
    });
    expect(merged.auth.oidc?.client_secret).toBe('oidc-secret');
  });

  it('should not mutate original config', () => {
    const dir = mkdtempSync(join(tmpdir(), 'k1s0-'));
    const path = writeConfig(
      dir,
      'config.yaml',
      `
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: localhost
  port: 5432
  name: test_db
  user: app
  password: ""
observability:
  log:
    level: info
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost"
    audience: "test"
`,
    );

    const cfg = load(path);
    mergeVaultSecrets(cfg, { 'database.password': 'secret123' });
    expect(cfg.database?.password).toBe('');
  });

  it('should handle nil optional fields safely', () => {
    const dir = mkdtempSync(join(tmpdir(), 'k1s0-'));
    const path = writeConfig(dir, 'config.yaml', MINIMAL_CONFIG_YAML);
    const cfg = load(path);
    const merged = mergeVaultSecrets(cfg, {
      'database.password': 'secret',
      'redis.password': 'secret',
      'auth.oidc.client_secret': 'secret',
    });
    expect(merged.database).toBeUndefined();
    expect(merged.redis).toBeUndefined();
    expect(merged.auth.oidc).toBeUndefined();
  });
});

describe('full config', () => {
  it('should load and validate full config', () => {
    const dir = mkdtempSync(join(tmpdir(), 'k1s0-'));
    const path = writeConfig(
      dir,
      'config.yaml',
      `
app:
  name: order-server
  version: "1.0.0"
  tier: service
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
  read_timeout: "30s"
  write_timeout: "30s"
  shutdown_timeout: "10s"
grpc:
  port: 50051
  max_recv_msg_size: 4194304
database:
  host: "localhost"
  port: 5432
  name: "order_db"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
kafka:
  brokers:
    - "localhost:9092"
  consumer_group: "order-server.default"
  security_protocol: "PLAINTEXT"
  topics:
    publish:
      - "k1s0.service.order.created.v1"
    subscribe:
      - "k1s0.service.payment.completed.v1"
redis:
  host: "localhost"
  port: 6379
  db: 0
  pool_size: 10
observability:
  log:
    level: info
    format: json
  trace:
    enabled: true
    endpoint: "localhost:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
  oidc:
    discovery_url: "http://localhost:8180/realms/k1s0/.well-known/openid-configuration"
    client_id: "k1s0-bff"
    redirect_uri: "http://localhost:3000/callback"
    scopes:
      - "openid"
      - "profile"
    jwks_uri: "http://localhost:8180/realms/k1s0/protocol/openid-connect/certs"
    jwks_cache_ttl: "10m"
`,
    );

    const cfg = load(path);
    expect(cfg.app.name).toBe('order-server');
    expect(cfg.app.tier).toBe('service');
    expect(cfg.grpc?.port).toBe(50051);
    expect(cfg.database?.name).toBe('order_db');
    expect(cfg.kafka?.security_protocol).toBe('PLAINTEXT');
    expect(cfg.redis?.port).toBe(6379);
    expect(cfg.auth.oidc?.client_id).toBe('k1s0-bff');
    expect(() => validate(cfg)).not.toThrow();
  });
});
