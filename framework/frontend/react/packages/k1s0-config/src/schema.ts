import { z } from "zod";

/**
 * API接続設定のスキーマ
 */
export const apiConfigSchema = z.object({
  baseUrl: z.string().url(),
  timeout: z.number().positive().default(30000),
  retryCount: z.number().nonnegative().default(3),
  retryDelay: z.number().nonnegative().default(1000),
});

export type ApiConfig = z.infer<typeof apiConfigSchema>;

/**
 * 認証設定のスキーマ
 */
export const authConfigSchema = z.object({
  enabled: z.boolean().default(true),
  provider: z.enum(["jwt", "oauth2", "session"]).default("jwt"),
  tokenRefreshThreshold: z.number().positive().default(300),
  storage: z.enum(["localStorage", "sessionStorage", "memory"]).default("localStorage"),
});

export type AuthConfig = z.infer<typeof authConfigSchema>;

/**
 * ロギング設定のスキーマ
 */
export const loggingConfigSchema = z.object({
  level: z.enum(["debug", "info", "warn", "error"]).default("info"),
  enableConsole: z.boolean().default(true),
  enableRemote: z.boolean().default(false),
  remoteEndpoint: z.string().url().optional(),
});

export type LoggingConfig = z.infer<typeof loggingConfigSchema>;

/**
 * テレメトリ設定のスキーマ
 */
export const telemetryConfigSchema = z.object({
  enabled: z.boolean().default(false),
  serviceName: z.string().default("k1s0-frontend"),
  endpoint: z.string().url().optional(),
  sampleRate: z.number().min(0).max(1).default(0.1),
});

export type TelemetryConfig = z.infer<typeof telemetryConfigSchema>;

/**
 * 機能フラグ設定のスキーマ
 */
export const featureFlagsSchema = z.record(z.string(), z.boolean());

export type FeatureFlags = z.infer<typeof featureFlagsSchema>;

/**
 * アプリケーション全体の設定スキーマ
 */
export const appConfigSchema = z.object({
  env: z.enum(["dev", "stg", "prod"]).default("dev"),
  appName: z.string().default("k1s0-app"),
  version: z.string().optional(),
  api: apiConfigSchema.optional(),
  auth: authConfigSchema.optional(),
  logging: loggingConfigSchema.optional(),
  telemetry: telemetryConfigSchema.optional(),
  features: featureFlagsSchema.optional(),
});

export type AppConfig = z.infer<typeof appConfigSchema>;

/**
 * 設定をバリデーションする
 */
export function validateConfig<T>(
  schema: z.ZodSchema<T>,
  config: unknown
): { success: true; data: T } | { success: false; errors: z.ZodError } {
  const result = schema.safeParse(config);
  if (result.success) {
    return { success: true, data: result.data };
  }
  return { success: false, errors: result.error };
}

/**
 * 部分的な設定をバリデーションする（マージ用）
 */
export function validatePartialConfig<T>(
  schema: z.ZodSchema<T>,
  config: unknown
): { success: true; data: Partial<T> } | { success: false; errors: z.ZodError } {
  const partialSchema = schema.partial();
  const result = partialSchema.safeParse(config);
  if (result.success) {
    return { success: true, data: result.data as Partial<T> };
  }
  return { success: false, errors: result.error };
}
