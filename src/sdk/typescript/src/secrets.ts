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
}
