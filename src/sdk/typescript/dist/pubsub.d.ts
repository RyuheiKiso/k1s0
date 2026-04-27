import type { K1s0Client } from "./client.js";
import type { Event } from "./proto/k1s0/tier1/pubsub/v1/pubsub_service_pb.js";
export interface PublishOptions {
    idempotencyKey?: string;
    metadata?: Record<string, string>;
}
export declare class PubSubFacade {
    private readonly client;
    constructor(client: K1s0Client);
    publish(topic: string, data: Uint8Array, contentType: string, opts?: PublishOptions): Promise<bigint>;
    /** subscribe はトピックの購読。AsyncIterable<Event> を返す。
     *  利用例:
     *    for await (const event of client.pubsub.subscribe("orders", "consumer-A")) {
     *      handle(event);
     *    }
     */
    subscribe(topic: string, consumerGroup: string): AsyncIterable<Event>;
}
//# sourceMappingURL=pubsub.d.ts.map