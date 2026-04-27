// 本ファイルは k1s0 TypeScript SDK の PubSub 動詞統一 facade（publish + subscribe）。
// PubSubFacade は PubSubService の動詞統一 facade。
export class PubSubFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    // Publish は単発 Publish。Kafka offset を返す。
    async publish(topic, data, contentType, opts = {}) {
        // raw client を生成する。
        const raw = this.client.rawPubSub();
        // RPC 呼出。
        const resp = await raw.publish({
            topic,
            data,
            contentType,
            idempotencyKey: opts.idempotencyKey ?? "",
            metadata: opts.metadata ?? {},
            context: this.client.tenantContext(),
        });
        // offset は proto3 int64 のため bigint を返却する。
        return resp.offset;
    }
    /** subscribe はトピックの購読。AsyncIterable<Event> を返す。
     *  利用例:
     *    for await (const event of client.pubsub.subscribe("orders", "consumer-A")) {
     *      handle(event);
     *    }
     */
    subscribe(topic, consumerGroup) {
        const raw = this.client.rawPubSub();
        return raw.subscribe({
            topic,
            consumerGroup,
            context: this.client.tenantContext(),
        });
    }
}
//# sourceMappingURL=pubsub.js.map