import { BulkPublishRequest, BulkPublishResponse, Event, PublishRequest, PublishResponse, SubscribeRequest } from "./pubsub_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * PubSub API。Kafka をバックエンドとし、tier1 がテナント接頭辞付与と冪等性管理を行う。
 *
 * @generated from service k1s0.tier1.pubsub.v1.PubSubService
 */
export declare const PubSubService: {
    readonly typeName: "k1s0.tier1.pubsub.v1.PubSubService";
    readonly methods: {
        /**
         * 単発 Publish（idempotency_key で 24h 重複抑止）
         *
         * @generated from rpc k1s0.tier1.pubsub.v1.PubSubService.Publish
         */
        readonly publish: {
            readonly name: "Publish";
            readonly I: typeof PublishRequest;
            readonly O: typeof PublishResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * バッチ Publish（個別エントリの成否を BulkPublishEntry で返す）
         *
         * @generated from rpc k1s0.tier1.pubsub.v1.PubSubService.BulkPublish
         */
        readonly bulkPublish: {
            readonly name: "BulkPublish";
            readonly I: typeof BulkPublishRequest;
            readonly O: typeof BulkPublishResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * サブスクリプション（tier2/tier3 側は HTTP コールバック登録 / gRPC ストリームのいずれか）
         * server-streaming のため HTTP/JSON gateway 経由非対応（gRPC で直接呼出す運用）。
         *
         * @generated from rpc k1s0.tier1.pubsub.v1.PubSubService.Subscribe
         */
        readonly subscribe: {
            readonly name: "Subscribe";
            readonly I: typeof SubscribeRequest;
            readonly O: typeof Event;
            readonly kind: MethodKind.ServerStreaming;
        };
    };
};
//# sourceMappingURL=pubsub_service_connect.d.ts.map