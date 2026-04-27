import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Get リクエスト
 *
 * @generated from message k1s0.tier1.state.v1.GetRequest
 */
export declare class GetRequest extends Message<GetRequest> {
    /**
     * Store 名（valkey-default / postgres-tenant 等、運用側で設定）
     *
     * @generated from field: string store = 1;
     */
    store: string;
    /**
     * キー（テナント境界は tier1 が自動付与、クライアントはテナント内キーのみ指定）
     *
     * @generated from field: string key = 2;
     */
    key: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<GetRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.GetRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetRequest;
    static equals(a: GetRequest | PlainMessage<GetRequest> | undefined, b: GetRequest | PlainMessage<GetRequest> | undefined): boolean;
}
/**
 * Get 応答
 *
 * @generated from message k1s0.tier1.state.v1.GetResponse
 */
export declare class GetResponse extends Message<GetResponse> {
    /**
     * 値本文（bytes で透過、encoding はクライアント責務）
     *
     * @generated from field: bytes data = 1;
     */
    data: Uint8Array;
    /**
     * 楽観的排他のための ETag（Set / Delete 時に expected_etag に再送する）
     *
     * @generated from field: string etag = 2;
     */
    etag: string;
    /**
     * キー未存在時は true（このとき data / etag は空、エラーではない）
     *
     * @generated from field: bool not_found = 3;
     */
    notFound: boolean;
    constructor(data?: PartialMessage<GetResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.GetResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetResponse;
    static equals(a: GetResponse | PlainMessage<GetResponse> | undefined, b: GetResponse | PlainMessage<GetResponse> | undefined): boolean;
}
/**
 * Set リクエスト
 *
 * @generated from message k1s0.tier1.state.v1.SetRequest
 */
export declare class SetRequest extends Message<SetRequest> {
    /**
     * Store 名
     *
     * @generated from field: string store = 1;
     */
    store: string;
    /**
     * キー
     *
     * @generated from field: string key = 2;
     */
    key: string;
    /**
     * 保存値本文
     *
     * @generated from field: bytes data = 3;
     */
    data: Uint8Array;
    /**
     * 期待 ETag（空は未存在前提、新規作成時は空文字列）
     *
     * @generated from field: string expected_etag = 4;
     */
    expectedEtag: string;
    /**
     * TTL（秒、0 は永続）
     *
     * @generated from field: int32 ttl_sec = 5;
     */
    ttlSec: number;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 6;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<SetRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.SetRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): SetRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): SetRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): SetRequest;
    static equals(a: SetRequest | PlainMessage<SetRequest> | undefined, b: SetRequest | PlainMessage<SetRequest> | undefined): boolean;
}
/**
 * Set 応答
 *
 * @generated from message k1s0.tier1.state.v1.SetResponse
 */
export declare class SetResponse extends Message<SetResponse> {
    /**
     * 保存後の新 ETag（次回 Set / Delete 時の expected_etag に渡す）
     *
     * @generated from field: string new_etag = 1;
     */
    newEtag: string;
    constructor(data?: PartialMessage<SetResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.SetResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): SetResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): SetResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): SetResponse;
    static equals(a: SetResponse | PlainMessage<SetResponse> | undefined, b: SetResponse | PlainMessage<SetResponse> | undefined): boolean;
}
/**
 * Delete リクエスト
 *
 * @generated from message k1s0.tier1.state.v1.DeleteRequest
 */
export declare class DeleteRequest extends Message<DeleteRequest> {
    /**
     * Store 名
     *
     * @generated from field: string store = 1;
     */
    store: string;
    /**
     * キー
     *
     * @generated from field: string key = 2;
     */
    key: string;
    /**
     * 期待 ETag（空は無条件削除、指定時は楽観的排他で削除）
     *
     * @generated from field: string expected_etag = 3;
     */
    expectedEtag: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 4;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<DeleteRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.DeleteRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): DeleteRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): DeleteRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): DeleteRequest;
    static equals(a: DeleteRequest | PlainMessage<DeleteRequest> | undefined, b: DeleteRequest | PlainMessage<DeleteRequest> | undefined): boolean;
}
/**
 * Delete 応答
 *
 * @generated from message k1s0.tier1.state.v1.DeleteResponse
 */
export declare class DeleteResponse extends Message<DeleteResponse> {
    /**
     * 削除実行可否（未存在キーへの削除も deleted=true で返す）
     *
     * @generated from field: bool deleted = 1;
     */
    deleted: boolean;
    constructor(data?: PartialMessage<DeleteResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.DeleteResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): DeleteResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): DeleteResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): DeleteResponse;
    static equals(a: DeleteResponse | PlainMessage<DeleteResponse> | undefined, b: DeleteResponse | PlainMessage<DeleteResponse> | undefined): boolean;
}
/**
 * BulkGet リクエスト
 *
 * @generated from message k1s0.tier1.state.v1.BulkGetRequest
 */
export declare class BulkGetRequest extends Message<BulkGetRequest> {
    /**
     * Store 名（全キーで共通）
     *
     * @generated from field: string store = 1;
     */
    store: string;
    /**
     * 取得するキー一覧
     *
     * @generated from field: repeated string keys = 2;
     */
    keys: string[];
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<BulkGetRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.BulkGetRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkGetRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkGetRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkGetRequest;
    static equals(a: BulkGetRequest | PlainMessage<BulkGetRequest> | undefined, b: BulkGetRequest | PlainMessage<BulkGetRequest> | undefined): boolean;
}
/**
 * BulkGet 応答
 *
 * @generated from message k1s0.tier1.state.v1.BulkGetResponse
 */
export declare class BulkGetResponse extends Message<BulkGetResponse> {
    /**
     * 結果マップ（キー → GetResponse、未存在キーも not_found=true で含める）
     *
     * @generated from field: map<string, k1s0.tier1.state.v1.GetResponse> results = 1;
     */
    results: {
        [key: string]: GetResponse;
    };
    constructor(data?: PartialMessage<BulkGetResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.BulkGetResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkGetResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkGetResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkGetResponse;
    static equals(a: BulkGetResponse | PlainMessage<BulkGetResponse> | undefined, b: BulkGetResponse | PlainMessage<BulkGetResponse> | undefined): boolean;
}
/**
 * トランザクション内の 1 操作（Set / Delete のいずれか）
 *
 * @generated from message k1s0.tier1.state.v1.TransactOp
 */
export declare class TransactOp extends Message<TransactOp> {
    /**
     * 操作種別（oneof で Set または Delete を排他選択）
     *
     * @generated from oneof k1s0.tier1.state.v1.TransactOp.op
     */
    op: {
        /**
         * Set 操作
         *
         * @generated from field: k1s0.tier1.state.v1.SetRequest set = 1;
         */
        value: SetRequest;
        case: "set";
    } | {
        /**
         * Delete 操作
         *
         * @generated from field: k1s0.tier1.state.v1.DeleteRequest delete = 2;
         */
        value: DeleteRequest;
        case: "delete";
    } | {
        case: undefined;
        value?: undefined;
    };
    constructor(data?: PartialMessage<TransactOp>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.TransactOp";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): TransactOp;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): TransactOp;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): TransactOp;
    static equals(a: TransactOp | PlainMessage<TransactOp> | undefined, b: TransactOp | PlainMessage<TransactOp> | undefined): boolean;
}
/**
 * Transact リクエスト
 *
 * @generated from message k1s0.tier1.state.v1.TransactRequest
 */
export declare class TransactRequest extends Message<TransactRequest> {
    /**
     * Store 名（全操作で共通、複数 Store を跨ぐトランザクションは不可）
     *
     * @generated from field: string store = 1;
     */
    store: string;
    /**
     * 操作列（記述順に実行、途中失敗で全ロールバック）
     *
     * @generated from field: repeated k1s0.tier1.state.v1.TransactOp operations = 2;
     */
    operations: TransactOp[];
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<TransactRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.TransactRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): TransactRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): TransactRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): TransactRequest;
    static equals(a: TransactRequest | PlainMessage<TransactRequest> | undefined, b: TransactRequest | PlainMessage<TransactRequest> | undefined): boolean;
}
/**
 * Transact 応答
 *
 * @generated from message k1s0.tier1.state.v1.TransactResponse
 */
export declare class TransactResponse extends Message<TransactResponse> {
    /**
     * コミット成功可否（false の場合は全ロールバック済み）
     *
     * @generated from field: bool committed = 1;
     */
    committed: boolean;
    constructor(data?: PartialMessage<TransactResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.state.v1.TransactResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): TransactResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): TransactResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): TransactResponse;
    static equals(a: TransactResponse | PlainMessage<TransactResponse> | undefined, b: TransactResponse | PlainMessage<TransactResponse> | undefined): boolean;
}
//# sourceMappingURL=state_service_pb.d.ts.map