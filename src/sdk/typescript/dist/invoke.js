// 本ファイルは k1s0 TypeScript SDK の ServiceInvoke 動詞統一 facade（unary + server streaming）。
import { createPromiseClient } from "@connectrpc/connect";
import { InvokeService } from "./proto/k1s0/tier1/serviceinvoke/v1/serviceinvoke_service_connect.js";
/** InvokeFacade は InvokeService の動詞統一 facade。 */
export class InvokeFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    /** call は任意サービスの任意メソッドを呼び出す（unary）。 */
    async call(appId, method, data, contentType, timeoutMs = 5000) {
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
    /** stream はサーバストリーミング呼出。AsyncIterable<InvokeChunk> を返す。
     *  利用例:
     *    for await (const chunk of client.invoke.stream(appId, method, data, contentType)) {
     *      console.log(chunk.data);
     *      if (chunk.eof) break;
     *    }
     */
    stream(appId, method, data, contentType, timeoutMs = 5000) {
        const raw = createPromiseClient(InvokeService, this.client.transport);
        return raw.invokeStream({
            appId,
            method,
            data,
            contentType,
            context: this.client.tenantContext(),
            timeoutMs,
        });
    }
}
//# sourceMappingURL=invoke.js.map