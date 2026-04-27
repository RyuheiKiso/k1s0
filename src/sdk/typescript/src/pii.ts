// 本ファイルは k1s0 TypeScript SDK の Pii 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { PiiService } from "./proto/k1s0/tier1/pii/v1/pii_service_connect.js";
import type { PiiFinding } from "./proto/k1s0/tier1/pii/v1/pii_service_pb.js";

/** PiiFacade は PiiService の動詞統一 facade。 */
export class PiiFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  /** classify は PII 種別の検出。 */
  async classify(text: string): Promise<{ findings: PiiFinding[]; containsPii: boolean }> {
    const raw = createPromiseClient(PiiService, this.client.transport);
    const resp = await raw.classify({ text, context: this.client.tenantContext() });
    return { findings: resp.findings, containsPii: resp.containsPii };
  }

  /** mask はマスキング。 */
  async mask(text: string): Promise<{ maskedText: string; findings: PiiFinding[] }> {
    const raw = createPromiseClient(PiiService, this.client.transport);
    const resp = await raw.mask({ text, context: this.client.tenantContext() });
    return { maskedText: resp.maskedText, findings: resp.findings };
  }
}
