import type { K1s0Client } from "./client.js";
export interface SaveOptions {
    expectedEtag?: string;
    ttlSec?: number;
    idempotencyKey?: string;
}
export declare class StateFacade {
    private readonly client;
    constructor(client: K1s0Client);
    get(store: string, key: string): Promise<{
        data: Uint8Array;
        etag: string;
    } | null>;
    save(store: string, key: string, data: Uint8Array, opts?: SaveOptions): Promise<string>;
    delete(store: string, key: string, expectedEtag?: string): Promise<boolean>;
    bulkGet(store: string, keys: string[]): Promise<Map<string, {
        data: Uint8Array;
        etag: string;
        found: boolean;
    }>>;
    transact(store: string, ops: TransactOpInput[]): Promise<boolean>;
}
export interface TransactOpInput {
    kind: "set" | "delete";
    key: string;
    data?: Uint8Array;
    expectedEtag?: string;
    ttlSec?: number;
}
//# sourceMappingURL=state.d.ts.map