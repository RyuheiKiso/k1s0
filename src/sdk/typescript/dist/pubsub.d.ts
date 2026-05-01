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
    /**
     * bulkPublish は複数エントリの一括 Publish（FR-T1-PUBSUB-001）。
     * 各エントリの結果を個別に返す（部分成功あり、全体エラーにはしない）。
     */
    bulkPublish(topic: string, entries: BulkPublishEntryInput[]): Promise<Array<{
        entryIndex: number;
        offset: bigint;
        errorCode: string;
    }>>;
}
/** BulkPublishEntryInput は bulkPublish の 1 件分の入力。 */
export interface BulkPublishEntryInput {
    /** データ本文。 */
    data: Uint8Array;
    /** Content-Type（application/json / application/protobuf 等）。 */
    contentType: string;
    /** 冪等性キー（24h 重複抑止）。省略可。 */
    idempotencyKey?: string;
    /** メタデータ（partition_key 等）。省略可。 */
    metadata?: Record<string, string>;
}
//# sourceMappingURL=pubsub.d.ts.map