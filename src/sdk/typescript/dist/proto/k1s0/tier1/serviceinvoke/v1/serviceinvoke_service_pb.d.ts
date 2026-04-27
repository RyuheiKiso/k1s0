import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Invoke リクエスト
 *
 * @generated from message k1s0.tier1.serviceinvoke.v1.InvokeRequest
 */
export declare class InvokeRequest extends Message<InvokeRequest> {
    /**
     * 呼出先のアプリ識別子（Dapr app_id 互換、tier2 のサービス名に相当）
     *
     * @generated from field: string app_id = 1;
     */
    appId: string;
    /**
     * 呼出先のメソッド名（HTTP の場合は path に相当）
     *
     * @generated from field: string method = 2;
     */
    method: string;
    /**
     * 呼出データ（bytes で透過伝搬、encoding は content_type で示す）
     *
     * @generated from field: bytes data = 3;
     */
    data: Uint8Array;
    /**
     * Content-Type（application/json / application/grpc / application/protobuf 等）
     *
     * @generated from field: string content_type = 4;
     */
    contentType: string;
    /**
     * 呼出元コンテキスト（テナント識別と相関 ID）
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context?: TenantContext;
    /**
     * タイムアウト（ミリ秒、省略時は 5000ms）
     *
     * @generated from field: int32 timeout_ms = 6;
     */
    timeoutMs: number;
    constructor(data?: PartialMessage<InvokeRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.serviceinvoke.v1.InvokeRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): InvokeRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): InvokeRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): InvokeRequest;
    static equals(a: InvokeRequest | PlainMessage<InvokeRequest> | undefined, b: InvokeRequest | PlainMessage<InvokeRequest> | undefined): boolean;
}
/**
 * Invoke 応答
 *
 * @generated from message k1s0.tier1.serviceinvoke.v1.InvokeResponse
 */
export declare class InvokeResponse extends Message<InvokeResponse> {
    /**
     * 応答データ（bytes で透過伝搬、encoding は content_type で示す）
     *
     * @generated from field: bytes data = 1;
     */
    data: Uint8Array;
    /**
     * Content-Type（呼出先が決定）
     *
     * @generated from field: string content_type = 2;
     */
    contentType: string;
    /**
     * HTTP ステータス相当（成功 200、失敗時は詳細を Status に載せる）
     *
     * @generated from field: int32 status = 3;
     */
    status: number;
    constructor(data?: PartialMessage<InvokeResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.serviceinvoke.v1.InvokeResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): InvokeResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): InvokeResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): InvokeResponse;
    static equals(a: InvokeResponse | PlainMessage<InvokeResponse> | undefined, b: InvokeResponse | PlainMessage<InvokeResponse> | undefined): boolean;
}
/**
 * ストリーム応答のチャンク
 *
 * @generated from message k1s0.tier1.serviceinvoke.v1.InvokeChunk
 */
export declare class InvokeChunk extends Message<InvokeChunk> {
    /**
     * チャンク本文
     *
     * @generated from field: bytes data = 1;
     */
    data: Uint8Array;
    /**
     * ストリーム終端フラグ（true の場合は本チャンクが最終）
     *
     * @generated from field: bool eof = 2;
     */
    eof: boolean;
    constructor(data?: PartialMessage<InvokeChunk>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.serviceinvoke.v1.InvokeChunk";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): InvokeChunk;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): InvokeChunk;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): InvokeChunk;
    static equals(a: InvokeChunk | PlainMessage<InvokeChunk> | undefined, b: InvokeChunk | PlainMessage<InvokeChunk> | undefined): boolean;
}
//# sourceMappingURL=serviceinvoke_service_pb.d.ts.map