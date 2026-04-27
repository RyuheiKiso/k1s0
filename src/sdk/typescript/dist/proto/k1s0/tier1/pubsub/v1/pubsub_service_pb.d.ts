import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Publish リクエスト
 *
 * @generated from message k1s0.tier1.pubsub.v1.PublishRequest
 */
export declare class PublishRequest extends Message<PublishRequest> {
    /**
     * トピック名（テナント接頭辞は tier1 が自動付与、クライアントはテナント内名のみ）
     *
     * @generated from field: string topic = 1;
     */
    topic: string;
    /**
     * イベント本文（bytes で透過、encoding は content_type で示す）
     *
     * @generated from field: bytes data = 2;
     */
    data: Uint8Array;
    /**
     * Content-Type（application/json / application/protobuf 等）
     *
     * @generated from field: string content_type = 3;
     */
    contentType: string;
    /**
     * 冪等性キー（重複 Publish を抑止、TTL 24h）
     *
     * @generated from field: string idempotency_key = 4;
     */
    idempotencyKey: string;
    /**
     * メタデータ（partition_key / trace_id 等の Kafka メッセージヘッダ相当）
     *
     * @generated from field: map<string, string> metadata = 5;
     */
    metadata: {
        [key: string]: string;
    };
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 6;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<PublishRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pubsub.v1.PublishRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): PublishRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): PublishRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): PublishRequest;
    static equals(a: PublishRequest | PlainMessage<PublishRequest> | undefined, b: PublishRequest | PlainMessage<PublishRequest> | undefined): boolean;
}
/**
 * Publish 応答
 *
 * @generated from message k1s0.tier1.pubsub.v1.PublishResponse
 */
export declare class PublishResponse extends Message<PublishResponse> {
    /**
     * Kafka 側のオフセット
     *
     * @generated from field: int64 offset = 1;
     */
    offset: bigint;
    constructor(data?: PartialMessage<PublishResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pubsub.v1.PublishResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): PublishResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): PublishResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): PublishResponse;
    static equals(a: PublishResponse | PlainMessage<PublishResponse> | undefined, b: PublishResponse | PlainMessage<PublishResponse> | undefined): boolean;
}
/**
 * BulkPublish リクエスト
 *
 * @generated from message k1s0.tier1.pubsub.v1.BulkPublishRequest
 */
export declare class BulkPublishRequest extends Message<BulkPublishRequest> {
    /**
     * トピック名（全エントリで共通）
     *
     * @generated from field: string topic = 1;
     */
    topic: string;
    /**
     * 公開するエントリ列
     *
     * @generated from field: repeated k1s0.tier1.pubsub.v1.PublishRequest entries = 2;
     */
    entries: PublishRequest[];
    constructor(data?: PartialMessage<BulkPublishRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pubsub.v1.BulkPublishRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkPublishRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkPublishRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkPublishRequest;
    static equals(a: BulkPublishRequest | PlainMessage<BulkPublishRequest> | undefined, b: BulkPublishRequest | PlainMessage<BulkPublishRequest> | undefined): boolean;
}
/**
 * BulkPublish 応答
 *
 * @generated from message k1s0.tier1.pubsub.v1.BulkPublishResponse
 */
export declare class BulkPublishResponse extends Message<BulkPublishResponse> {
    /**
     * 各エントリの結果（失敗時は error_code に詳細）
     *
     * @generated from field: repeated k1s0.tier1.pubsub.v1.BulkPublishEntry results = 1;
     */
    results: BulkPublishEntry[];
    constructor(data?: PartialMessage<BulkPublishResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pubsub.v1.BulkPublishResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkPublishResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkPublishResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkPublishResponse;
    static equals(a: BulkPublishResponse | PlainMessage<BulkPublishResponse> | undefined, b: BulkPublishResponse | PlainMessage<BulkPublishResponse> | undefined): boolean;
}
/**
 * BulkPublish の個別エントリ結果
 *
 * @generated from message k1s0.tier1.pubsub.v1.BulkPublishEntry
 */
export declare class BulkPublishEntry extends Message<BulkPublishEntry> {
    /**
     * 入力 entries 配列内のインデックス（0 始まり）
     *
     * @generated from field: int32 entry_index = 1;
     */
    entryIndex: number;
    /**
     * Kafka 側のオフセット（成功時のみ意味を持つ）
     *
     * @generated from field: int64 offset = 2;
     */
    offset: bigint;
    /**
     * 失敗時のエラーコード（成功時は空文字列）
     *
     * @generated from field: string error_code = 3;
     */
    errorCode: string;
    constructor(data?: PartialMessage<BulkPublishEntry>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pubsub.v1.BulkPublishEntry";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkPublishEntry;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkPublishEntry;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkPublishEntry;
    static equals(a: BulkPublishEntry | PlainMessage<BulkPublishEntry> | undefined, b: BulkPublishEntry | PlainMessage<BulkPublishEntry> | undefined): boolean;
}
/**
 * Subscribe リクエスト
 *
 * @generated from message k1s0.tier1.pubsub.v1.SubscribeRequest
 */
export declare class SubscribeRequest extends Message<SubscribeRequest> {
    /**
     * トピック名
     *
     * @generated from field: string topic = 1;
     */
    topic: string;
    /**
     * コンシューマグループ（テナント単位で分離）
     *
     * @generated from field: string consumer_group = 2;
     */
    consumerGroup: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<SubscribeRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pubsub.v1.SubscribeRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): SubscribeRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): SubscribeRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): SubscribeRequest;
    static equals(a: SubscribeRequest | PlainMessage<SubscribeRequest> | undefined, b: SubscribeRequest | PlainMessage<SubscribeRequest> | undefined): boolean;
}
/**
 * Event（Subscribe の stream 要素）
 *
 * @generated from message k1s0.tier1.pubsub.v1.Event
 */
export declare class Event extends Message<Event> {
    /**
     * トピック名（接頭辞除去済みのテナント内名）
     *
     * @generated from field: string topic = 1;
     */
    topic: string;
    /**
     * イベント本文
     *
     * @generated from field: bytes data = 2;
     */
    data: Uint8Array;
    /**
     * Content-Type
     *
     * @generated from field: string content_type = 3;
     */
    contentType: string;
    /**
     * Kafka 側のオフセット
     *
     * @generated from field: int64 offset = 4;
     */
    offset: bigint;
    /**
     * メタデータ（Publish 時に付与されたヘッダがそのまま伝わる）
     *
     * @generated from field: map<string, string> metadata = 5;
     */
    metadata: {
        [key: string]: string;
    };
    constructor(data?: PartialMessage<Event>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pubsub.v1.Event";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): Event;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): Event;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): Event;
    static equals(a: Event | PlainMessage<Event> | undefined, b: Event | PlainMessage<Event> | undefined): boolean;
}
//# sourceMappingURL=pubsub_service_pb.d.ts.map