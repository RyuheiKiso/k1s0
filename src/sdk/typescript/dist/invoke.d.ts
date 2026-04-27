import type { K1s0Client } from "./client.js";
import type { InvokeChunk } from "./proto/k1s0/tier1/serviceinvoke/v1/serviceinvoke_service_pb.js";
/** InvokeFacade は InvokeService の動詞統一 facade。 */
export declare class InvokeFacade {
    private readonly client;
    constructor(client: K1s0Client);
    /** call は任意サービスの任意メソッドを呼び出す（unary）。 */
    call(appId: string, method: string, data: Uint8Array, contentType: string, timeoutMs?: number): Promise<{
        data: Uint8Array;
        contentType: string;
        status: number;
    }>;
    /** stream はサーバストリーミング呼出。AsyncIterable<InvokeChunk> を返す。
     *  利用例:
     *    for await (const chunk of client.invoke.stream(appId, method, data, contentType)) {
     *      console.log(chunk.data);
     *      if (chunk.eof) break;
     *    }
     */
    stream(appId: string, method: string, data: Uint8Array, contentType: string, timeoutMs?: number): AsyncIterable<InvokeChunk>;
}
//# sourceMappingURL=invoke.d.ts.map