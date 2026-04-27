// @k1s0/config エントリポイント。
//
// tier3 web の app / package が読む環境変数を一元化する。
// Vite 環境では import.meta.env、Node 環境では process.env を読む。

// 設定スキーマ。
export interface AppConfig {
  // BFF endpoint URL（例: "http://localhost:8080"）。
  bffUrl: string;
  // tenant ID（リクエストの `x-tenant-id` ヘッダに付与）。
  tenantId: string;
  // 環境名（dev / staging / prod）。
  environment: 'dev' | 'staging' | 'prod';
  // OTel collector URL（空なら observability 未設定）。
  otelCollectorUrl: string;
  // Keycloak issuer URL（空なら認証スキップ）。
  keycloakIssuer: string;
}

// 環境変数 → AppConfig の型変換ヘルパ。
//
// Vite では import.meta.env、Node / Vitest では process.env を入力に取る前提で、
// caller は最低限のソース選択のみ提供する。
export function loadConfig(env: Record<string, unknown>): AppConfig {
  // BFF URL の必須チェック。
  const bffUrl = String(env.VITE_BFF_URL ?? env.BFF_URL ?? '');
  if (!bffUrl) {
    throw new Error('config: BFF_URL is required (VITE_BFF_URL or BFF_URL)');
  }
  // tenant ID。
  const tenantId = String(env.VITE_TENANT_ID ?? env.TENANT_ID ?? 'tenant-dev');
  // 環境名は固定値のみ受理する。
  const envRaw = String(env.VITE_ENVIRONMENT ?? env.ENVIRONMENT ?? 'dev');
  const environment: AppConfig['environment'] =
    envRaw === 'staging' || envRaw === 'prod' ? envRaw : 'dev';
  // OTel collector URL（任意）。
  const otelCollectorUrl = String(env.VITE_OTEL_COLLECTOR_URL ?? env.OTEL_COLLECTOR_URL ?? '');
  // Keycloak issuer URL（任意）。
  const keycloakIssuer = String(env.VITE_KEYCLOAK_ISSUER ?? env.KEYCLOAK_ISSUER ?? '');
  return { bffUrl, tenantId, environment, otelCollectorUrl, keycloakIssuer };
}

// テスト容易性のための Config スタブヘルパ。
export function stubConfig(overrides: Partial<AppConfig> = {}): AppConfig {
  return {
    bffUrl: 'http://localhost:8080',
    tenantId: 'tenant-test',
    environment: 'dev',
    otelCollectorUrl: '',
    keycloakIssuer: '',
    ...overrides,
  };
}
