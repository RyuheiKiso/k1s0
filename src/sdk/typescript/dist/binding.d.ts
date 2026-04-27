import type { K1s0Client } from "./client.js";
/** BindingFacade は BindingService の動詞統一 facade。 */
export declare class BindingFacade {
    private readonly client;
    constructor(client: K1s0Client);
    /** invoke は出力バインディング呼出。 */
    invoke(name: string, operation: string, data: Uint8Array, metadata?: Record<string, string>): Promise<{
        data: Uint8Array;
        metadata: Record<string, string>;
    }>;
}
//# sourceMappingURL=binding.d.ts.map