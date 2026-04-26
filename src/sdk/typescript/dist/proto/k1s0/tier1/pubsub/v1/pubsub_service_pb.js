// 本ファイルは tier1 公開 PubSub API の正式 proto。
// Kafka 抽象 Publish / Subscribe を提供する（テナント境界はトピック接頭辞で自動隔離）。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/03_PubSub_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/03_PubSub_API.md
//
// 関連要件: FR-T1-PUBSUB-001〜005
// proto 構文宣言（proto3）
import { Message, proto3, protoInt64 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Publish リクエスト
 *
 * @generated from message k1s0.tier1.pubsub.v1.PublishRequest
 */
export class PublishRequest extends Message {
    /**
     * トピック名（テナント接頭辞は tier1 が自動付与、クライアントはテナント内名のみ）
     *
     * @generated from field: string topic = 1;
     */
    topic = "";
    /**
     * イベント本文（bytes で透過、encoding は content_type で示す）
     *
     * @generated from field: bytes data = 2;
     */
    data = new Uint8Array(0);
    /**
     * Content-Type（application/json / application/protobuf 等）
     *
     * @generated from field: string content_type = 3;
     */
    contentType = "";
    /**
     * 冪等性キー（重複 Publish を抑止、TTL 24h）
     *
     * @generated from field: string idempotency_key = 4;
     */
    idempotencyKey = "";
    /**
     * メタデータ（partition_key / trace_id 等の Kafka メッセージヘッダ相当）
     *
     * @generated from field: map<string, string> metadata = 5;
     */
    metadata = {};
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 6;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pubsub.v1.PublishRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "topic", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "data", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 3, name: "content_type", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 4, name: "idempotency_key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 5, name: "metadata", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
        { no: 6, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new PublishRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new PublishRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new PublishRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(PublishRequest, a, b);
    }
}
/**
 * Publish 応答
 *
 * @generated from message k1s0.tier1.pubsub.v1.PublishResponse
 */
export class PublishResponse extends Message {
    /**
     * Kafka 側のオフセット
     *
     * @generated from field: int64 offset = 1;
     */
    offset = protoInt64.zero;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pubsub.v1.PublishResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "offset", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
    ]);
    static fromBinary(bytes, options) {
        return new PublishResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new PublishResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new PublishResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(PublishResponse, a, b);
    }
}
/**
 * BulkPublish リクエスト
 *
 * @generated from message k1s0.tier1.pubsub.v1.BulkPublishRequest
 */
export class BulkPublishRequest extends Message {
    /**
     * トピック名（全エントリで共通）
     *
     * @generated from field: string topic = 1;
     */
    topic = "";
    /**
     * 公開するエントリ列
     *
     * @generated from field: repeated k1s0.tier1.pubsub.v1.PublishRequest entries = 2;
     */
    entries = [];
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pubsub.v1.BulkPublishRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "topic", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "entries", kind: "message", T: PublishRequest, repeated: true },
    ]);
    static fromBinary(bytes, options) {
        return new BulkPublishRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BulkPublishRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BulkPublishRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BulkPublishRequest, a, b);
    }
}
/**
 * BulkPublish 応答
 *
 * @generated from message k1s0.tier1.pubsub.v1.BulkPublishResponse
 */
export class BulkPublishResponse extends Message {
    /**
     * 各エントリの結果（失敗時は error_code に詳細）
     *
     * @generated from field: repeated k1s0.tier1.pubsub.v1.BulkPublishEntry results = 1;
     */
    results = [];
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pubsub.v1.BulkPublishResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "results", kind: "message", T: BulkPublishEntry, repeated: true },
    ]);
    static fromBinary(bytes, options) {
        return new BulkPublishResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BulkPublishResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BulkPublishResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BulkPublishResponse, a, b);
    }
}
/**
 * BulkPublish の個別エントリ結果
 *
 * @generated from message k1s0.tier1.pubsub.v1.BulkPublishEntry
 */
export class BulkPublishEntry extends Message {
    /**
     * 入力 entries 配列内のインデックス（0 始まり）
     *
     * @generated from field: int32 entry_index = 1;
     */
    entryIndex = 0;
    /**
     * Kafka 側のオフセット（成功時のみ意味を持つ）
     *
     * @generated from field: int64 offset = 2;
     */
    offset = protoInt64.zero;
    /**
     * 失敗時のエラーコード（成功時は空文字列）
     *
     * @generated from field: string error_code = 3;
     */
    errorCode = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pubsub.v1.BulkPublishEntry";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "entry_index", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 2, name: "offset", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
        { no: 3, name: "error_code", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new BulkPublishEntry().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BulkPublishEntry().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BulkPublishEntry().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BulkPublishEntry, a, b);
    }
}
/**
 * Subscribe リクエスト
 *
 * @generated from message k1s0.tier1.pubsub.v1.SubscribeRequest
 */
export class SubscribeRequest extends Message {
    /**
     * トピック名
     *
     * @generated from field: string topic = 1;
     */
    topic = "";
    /**
     * コンシューマグループ（テナント単位で分離）
     *
     * @generated from field: string consumer_group = 2;
     */
    consumerGroup = "";
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pubsub.v1.SubscribeRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "topic", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "consumer_group", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new SubscribeRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new SubscribeRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new SubscribeRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(SubscribeRequest, a, b);
    }
}
/**
 * Event（Subscribe の stream 要素）
 *
 * @generated from message k1s0.tier1.pubsub.v1.Event
 */
export class Event extends Message {
    /**
     * トピック名（接頭辞除去済みのテナント内名）
     *
     * @generated from field: string topic = 1;
     */
    topic = "";
    /**
     * イベント本文
     *
     * @generated from field: bytes data = 2;
     */
    data = new Uint8Array(0);
    /**
     * Content-Type
     *
     * @generated from field: string content_type = 3;
     */
    contentType = "";
    /**
     * Kafka 側のオフセット
     *
     * @generated from field: int64 offset = 4;
     */
    offset = protoInt64.zero;
    /**
     * メタデータ（Publish 時に付与されたヘッダがそのまま伝わる）
     *
     * @generated from field: map<string, string> metadata = 5;
     */
    metadata = {};
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pubsub.v1.Event";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "topic", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "data", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 3, name: "content_type", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 4, name: "offset", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
        { no: 5, name: "metadata", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
    ]);
    static fromBinary(bytes, options) {
        return new Event().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new Event().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new Event().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(Event, a, b);
    }
}
//# sourceMappingURL=pubsub_service_pb.js.map