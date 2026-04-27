// 本ファイルは k1s0 TypeScript SDK の DecisionAdmin 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { DecisionAdminService } from "./proto/k1s0/tier1/decision/v1/decision_service_connect.js";
import type { RuleVersionMeta } from "./proto/k1s0/tier1/decision/v1/decision_service_pb.js";

/** DecisionAdminFacade は DecisionAdminService の動詞統一 facade。 */
export class DecisionAdminFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  /** registerRule は JDM 文書の登録。 */
  async registerRule(
    ruleId: string,
    jdmDocument: Uint8Array,
    sigstoreSignature: Uint8Array,
    commitHash: string,
  ): Promise<{ ruleVersion: string; effectiveAtMs: bigint }> {
    const raw = createPromiseClient(DecisionAdminService, this.client.transport);
    const resp = await raw.registerRule({
      ruleId,
      jdmDocument,
      sigstoreSignature,
      commitHash,
      context: this.client.tenantContext(),
    });
    return { ruleVersion: resp.ruleVersion, effectiveAtMs: resp.effectiveAtMs };
  }

  /** listVersions はバージョン一覧。 */
  async listVersions(ruleId: string): Promise<RuleVersionMeta[]> {
    const raw = createPromiseClient(DecisionAdminService, this.client.transport);
    const resp = await raw.listVersions({ ruleId, context: this.client.tenantContext() });
    return resp.versions;
  }

  /** getRule は特定バージョンの取得。 */
  async getRule(
    ruleId: string,
    ruleVersion: string,
  ): Promise<{ jdmDocument: Uint8Array; meta?: RuleVersionMeta }> {
    const raw = createPromiseClient(DecisionAdminService, this.client.transport);
    const resp = await raw.getRule({ ruleId, ruleVersion, context: this.client.tenantContext() });
    return { jdmDocument: resp.jdmDocument, meta: resp.meta };
  }
}
