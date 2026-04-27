import type { K1s0Client } from "./client.js";
export interface PublishOptions {
    idempotencyKey?: string;
    metadata?: Record<string, string>;
}
export declare class PubSubFacade {
    private readonly client;
    constructor(client: K1s0Client);
    publish(topic: string, data: Uint8Array, contentType: string, opts?: PublishOptions): Promise<bigint>;
}
//# sourceMappingURL=pubsub.d.ts.map