// 本ファイルは k1s0 TypeScript SDK の Secrets 動詞統一 facade。
// `client.secrets.get(...)` 形式で SecretsService への呼出を提供する。

import type { K1s0Client } from "./client.js";

// Rotate オプション。
export interface RotateOptions {
  // 旧バージョンの猶予時間（秒、既定 3600）。
  gracePeriodSec?: number;
  // 動的シークレットの発行ポリシー名。
  policy?: string;
  // 冪等性キー。
  idempotencyKey?: string;
}

// SecretsFacade は SecretsService の動詞統一 facade。
export class SecretsFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  // Get はシークレット名で値（key=value マップ）と version を取得する。
  async get(
    name: string,
  ): Promise<{ values: Record<string, string>; version: number }> {
    // raw client を生成する。
    const raw = this.client.rawSecrets();
    // RPC 呼出。
    const resp = await raw.get({
      name,
      context: this.client.tenantContext(),
    });
    // (values, version) を返却する。
    return { values: resp.values, version: resp.version };
  }

  // Rotate はシークレットのローテーション。新バージョンと旧バージョンを返す。
  async rotate(
    name: string,
    opts: RotateOptions = {},
  ): Promise<{ newVersion: number; previousVersion: number }> {
    // raw client を生成する。
    const raw = this.client.rawSecrets();
    // RPC 呼出。
    const resp = await raw.rotate({
      name,
      context: this.client.tenantContext(),
      gracePeriodSec: opts.gracePeriodSec ?? 3600,
      policy: opts.policy,
      idempotencyKey: opts.idempotencyKey ?? "",
    });
    // (newVersion, previousVersion) を返却する。
    return {
      newVersion: resp.newVersion,
      previousVersion: resp.previousVersion,
    };
  }

  /**
   * getDynamic は動的 Secret 発行（FR-T1-SECRETS-002）。
   * engine="postgres" / "mysql" / "kafka" 等の OpenBao Database Engine 種別を指定する。
   * ttlSec=0 で既定 1 時間（3600）、上限 24 時間（86400）に clamp される。
   */
  async getDynamic(
    engine: string,
    role: string,
    ttlSec = 0,
  ): Promise<DynamicSecret> {
    const raw = this.client.rawSecrets();
    const resp = await raw.getDynamic({
      engine,
      role,
      ttlSec,
      context: this.client.tenantContext(),
    });
    return {
      values: resp.values,
      leaseId: resp.leaseId,
      ttlSec: resp.ttlSec,
      issuedAtMs: Number(resp.issuedAtMs),
    };
  }
}

/**
 * 動的 Secret 発行（FR-T1-SECRETS-002）の応答を SDK 利用者向けに整理した型。
 */
export interface DynamicSecret {
  /** credential 一式（"username" / "password" など、engine 別の field）。 */
  values: Record<string, string>;
  /** OpenBao の lease ID（renewal / revoke 用）。 */
  leaseId: string;
  /** 実際に付与された TTL 秒（要求値から ceiling までクランプされる）。 */
  ttlSec: number;
  /** 発効時刻（Unix epoch ミリ秒）。 */
  issuedAtMs: number;
}
