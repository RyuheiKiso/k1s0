import { z } from 'zod';

export const AppConfigSchema = z.object({
  name: z.string().min(1),
  version: z.string().min(1),
  tier: z.enum(['system', 'business', 'service']),
  environment: z.enum(['dev', 'staging', 'prod']),
});

export const ServerConfigSchema = z.object({
  host: z.string().min(1),
  port: z.number().int().min(1).max(65535),
  read_timeout: z.string().optional(),
  write_timeout: z.string().optional(),
  shutdown_timeout: z.string().optional(),
});

export const GrpcConfigSchema = z
  .object({
    port: z.number().int().min(1).max(65535),
    max_recv_msg_size: z.number().optional(),
  })
  .optional();

export const DatabaseConfigSchema = z
  .object({
    host: z.string().min(1),
    port: z.number().int().min(1).max(65535),
    name: z.string().min(1),
    user: z.string().min(1),
    password: z.string(),
    ssl_mode: z.enum(['disable', 'require', 'verify-full']).optional(),
    max_open_conns: z.number().optional(),
    max_idle_conns: z.number().optional(),
    conn_max_lifetime: z.string().optional(),
  })
  .optional();

export const KafkaConfigSchema = z
  .object({
    brokers: z.array(z.string()).min(1),
    consumer_group: z.string().min(1),
    security_protocol: z.enum(['PLAINTEXT', 'SASL_SSL']),
    sasl: z
      .object({
        mechanism: z.enum(['SCRAM-SHA-512', 'PLAIN']),
        username: z.string(),
        password: z.string(),
      })
      .optional(),
    tls: z
      .object({
        ca_cert_path: z.string().optional(),
      })
      .optional(),
    topics: z.object({
      publish: z.array(z.string()),
      subscribe: z.array(z.string()),
    }),
  })
  .optional();

export const RedisConfigSchema = z
  .object({
    host: z.string().min(1),
    port: z.number().int().min(1).max(65535),
    password: z.string().optional(),
    db: z.number().optional(),
    pool_size: z.number().optional(),
  })
  .optional();

export const ObservabilityConfigSchema = z.object({
  log: z.object({
    level: z.enum(['debug', 'info', 'warn', 'error']),
    format: z.enum(['json', 'text']),
  }),
  trace: z.object({
    enabled: z.boolean(),
    endpoint: z.string().optional(),
    sample_rate: z.number().min(0).max(1).optional(),
  }),
  metrics: z.object({
    enabled: z.boolean(),
    path: z.string().optional(),
  }),
});

export const AuthConfigSchema = z.object({
  jwt: z.object({
    issuer: z.string().min(1),
    audience: z.string().min(1),
    public_key_path: z.string().optional(),
  }),
  oidc: z
    .object({
      discovery_url: z.string().url(),
      client_id: z.string().min(1),
      client_secret: z.string().optional(),
      redirect_uri: z.string().url(),
      scopes: z.array(z.string()),
      jwks_uri: z.string().url(),
      jwks_cache_ttl: z.string().optional(),
    })
    .optional(),
});

export const ConfigSchema = z.object({
  app: AppConfigSchema,
  server: ServerConfigSchema,
  grpc: GrpcConfigSchema,
  database: DatabaseConfigSchema,
  kafka: KafkaConfigSchema,
  redis: RedisConfigSchema,
  redis_session: RedisConfigSchema,
  observability: ObservabilityConfigSchema,
  auth: AuthConfigSchema,
});

export type Config = z.infer<typeof ConfigSchema>;
export type AppConfig = z.infer<typeof AppConfigSchema>;
export type ServerConfig = z.infer<typeof ServerConfigSchema>;
export type DatabaseConfig = z.infer<typeof DatabaseConfigSchema>;
export type AuthConfig = z.infer<typeof AuthConfigSchema>;
