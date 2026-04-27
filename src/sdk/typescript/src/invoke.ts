// 本ファイルは k1s0 TypeScript SDK の ServiceInvoke 動詞統一 facade。
// InvokeStream は本リリース時点 では raw 経由（client.transport から直接 PromiseClient を組む）。
import { createPromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { InvokeService } from "./proto/k1s0/tier1/serviceinvoke/v1/serviceinvoke_service_connect.js";

/** InvokeFacade は InvokeService の動詞統一 facade。 */
export class InvokeFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  /** call は任意サービスの任意メソッドを呼び出す（unary）。 */
  async call(
    appId: string,
    method: string,
    data: Uint8Array,
    contentType: string,
    timeoutMs = 5000,
  ): Promise<{ data: Uint8Array; contentType: string; status: number }> {
    const raw = createPromiseClient(InvokeService, this.client.transport);
    const resp = await raw.invoke({
      appId,
      method,
      data,
      contentType,
      context: this.client.tenantContext(),
      timeoutMs,
    });
    return { data: resp.data, contentType: resp.contentType, status: resp.status };
  }
}
