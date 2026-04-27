// 本ファイルは k1s0 TypeScript SDK の Decision 動詞統一 facade（評価部のみ）。
import { createPromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { DecisionService } from "./proto/k1s0/tier1/decision/v1/decision_service_connect.js";

/** DecisionFacade は DecisionService（評価）の動詞統一 facade。 */
export class DecisionFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  /** evaluate はルール評価（同期）。返り値は { outputJson, traceJson, elapsedUs }。 */
  async evaluate(
    ruleId: string,
    ruleVersion: string,
    inputJson: Uint8Array,
    includeTrace = false,
  ): Promise<{ outputJson: Uint8Array; traceJson: Uint8Array; elapsedUs: bigint }> {
    const raw = createPromiseClient(DecisionService, this.client.transport);
    const resp = await raw.evaluate({
      ruleId,
      ruleVersion,
      inputJson,
      includeTrace,
      context: this.client.tenantContext(),
    });
    return { outputJson: resp.outputJson, traceJson: resp.traceJson, elapsedUs: resp.elapsedUs };
  }

  /** batchEvaluate はバッチ評価。 */
  async batchEvaluate(
    ruleId: string,
    ruleVersion: string,
    inputs: Uint8Array[],
  ): Promise<Uint8Array[]> {
    const raw = createPromiseClient(DecisionService, this.client.transport);
    const resp = await raw.batchEvaluate({
      ruleId,
      ruleVersion,
      inputsJson: inputs,
      context: this.client.tenantContext(),
    });
    return resp.outputsJson;
  }
}
