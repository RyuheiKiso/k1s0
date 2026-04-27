// 本ファイルは k1s0 TypeScript SDK の FeatureAdmin 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { FeatureAdminService } from "./proto/k1s0/tier1/feature/v1/feature_service_connect.js";
import type {
  FlagDefinition,
  FlagKind,
  FlagState,
} from "./proto/k1s0/tier1/feature/v1/feature_service_pb.js";

/** FeatureAdminFacade は FeatureAdminService の動詞統一 facade。 */
export class FeatureAdminFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  /** registerFlag は Flag 定義の登録（permission 種別は approvalId 必須）。 */
  async registerFlag(
    flag: FlagDefinition,
    changeReason: string,
    approvalId = "",
  ): Promise<bigint> {
    const raw = createPromiseClient(FeatureAdminService, this.client.transport);
    const resp = await raw.registerFlag({
      flag,
      changeReason,
      approvalId,
      context: this.client.tenantContext(),
    });
    return resp.version;
  }

  /** getFlag は Flag 定義の取得。version 省略で最新。 */
  async getFlag(
    flagKey: string,
    version?: bigint,
  ): Promise<{ flag?: FlagDefinition; version: bigint }> {
    const raw = createPromiseClient(FeatureAdminService, this.client.transport);
    const resp = await raw.getFlag({
      flagKey,
      version,
      context: this.client.tenantContext(),
    });
    return { flag: resp.flag, version: resp.version };
  }

  /** listFlags は Flag 定義の一覧。 */
  async listFlags(
    kind?: FlagKind,
    state?: FlagState,
  ): Promise<FlagDefinition[]> {
    const raw = createPromiseClient(FeatureAdminService, this.client.transport);
    const resp = await raw.listFlags({
      kind,
      state,
      context: this.client.tenantContext(),
    });
    return resp.flags;
  }
}
