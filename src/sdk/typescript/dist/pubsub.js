// 本ファイルは k1s0 TypeScript SDK の PubSub 動詞統一 facade。
// `client.pubsub.publish(...)` 形式で PubSubService への呼出を提供する。
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
}
//# sourceMappingURL=pubsub.js.map