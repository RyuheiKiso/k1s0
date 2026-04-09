// E2E テスト用のサービス接続設定を一元管理する。
// 全ての URL やポートはここから取得し、spec ファイルにハードコードしない。

/** BFF Proxy のベース URL */
export const BFF_BASE = process.env.BASE_URL ?? "http://localhost:8082";

/** Keycloak のベース URL */
export const KEYCLOAK_BASE = process.env.KEYCLOAK_URL ?? "http://localhost:8180";

/** サービスごとのホストポートマッピング（docker-compose に対応） */
export const SERVICE_PORTS: Record<string, number> = {
  auth: Number(process.env.AUTH_PORT) || 8083,
  config: Number(process.env.CONFIG_PORT) || 8084,
  saga: Number(process.env.SAGA_PORT) || 8085,
  "dlq-manager": Number(process.env.DLQ_MANAGER_PORT) || 8086,
  // MED-016 修正: featureflag の実際のポートは 8187（8087 は誤り）
  featureflag: Number(process.env.FEATUREFLAG_PORT) || 8187,
  ratelimit: Number(process.env.RATELIMIT_PORT) || 8088,
  tenant: Number(process.env.TENANT_PORT) || 8089,
  vault: Number(process.env.VAULT_PORT) || 8091,
  "bff-proxy": Number(process.env.BFF_PROXY_PORT) || 8082,
};

/**
 * 指定サービスの URL を返すヘルパー関数。
 * ホスト名は "localhost" を使用するが、将来的に環境変数化できる構造にしている。
 */
export function serviceUrl(service: string, path: string): string {
  const port = SERVICE_PORTS[service];
  if (!port) throw new Error(`Unknown service: ${service}`);
  return `http://localhost:${port}${path}`;
}
