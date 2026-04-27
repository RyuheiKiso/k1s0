// 本ファイルは k1s0 TypeScript SDK の Binding 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { BindingService } from "./proto/k1s0/tier1/binding/v1/binding_service_connect.js";

/** BindingFacade は BindingService の動詞統一 facade。 */
export class BindingFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  /** invoke は出力バインディング呼出。 */
  async invoke(
    name: string,
    operation: string,
    data: Uint8Array,
    metadata: Record<string, string> = {},
  ): Promise<{ data: Uint8Array; metadata: Record<string, string> }> {
    const raw = createPromiseClient(BindingService, this.client.transport);
    const resp = await raw.invoke({
      name,
      operation,
      data,
      metadata,
      context: this.client.tenantContext(),
    });
    return { data: resp.data, metadata: resp.metadata };
  }
}
