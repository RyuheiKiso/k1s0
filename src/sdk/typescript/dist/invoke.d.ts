import type { K1s0Client } from "./client.js";
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
}
//# sourceMappingURL=invoke.d.ts.map