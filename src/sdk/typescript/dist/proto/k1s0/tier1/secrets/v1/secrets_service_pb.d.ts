import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Get リクエスト
 *
 * @generated from message k1s0.tier1.secrets.v1.GetSecretRequest
 */
export declare class GetSecretRequest extends Message<GetSecretRequest> {
    /**
     * シークレット名（テナント境界を超えた参照は即 PermissionDenied）
     *
     * @generated from field: string name = 1;
     */
    name: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    /**
     * 省略時は最新、明示で旧バージョン取得可（grace_period 中のみ）
     *
     * @generated from field: optional int32 version = 3;
     */
    version?: number;
    constructor(data?: PartialMessage<GetSecretRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.GetSecretRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetSecretRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetSecretRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetSecretRequest;
    static equals(a: GetSecretRequest | PlainMessage<GetSecretRequest> | undefined, b: GetSecretRequest | PlainMessage<GetSecretRequest> | undefined): boolean;
}
/**
 * Get 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.GetSecretResponse
 */
export declare class GetSecretResponse extends Message<GetSecretResponse> {
    /**
     * 値（Base64 エンコード必要時はクライアント側で判断、複数キーの key=value マップ）
     *
     * @generated from field: map<string, string> values = 1;
     */
    values: {
        [key: string]: string;
    };
    /**
     * バージョン（ローテーション追跡用）
     *
     * @generated from field: int32 version = 2;
     */
    version: number;
    constructor(data?: PartialMessage<GetSecretResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.GetSecretResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetSecretResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetSecretResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetSecretResponse;
    static equals(a: GetSecretResponse | PlainMessage<GetSecretResponse> | undefined, b: GetSecretResponse | PlainMessage<GetSecretResponse> | undefined): boolean;
}
/**
 * BulkGet リクエスト
 *
 * @generated from message k1s0.tier1.secrets.v1.BulkGetSecretRequest
 */
export declare class BulkGetSecretRequest extends Message<BulkGetSecretRequest> {
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<BulkGetSecretRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.BulkGetSecretRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkGetSecretRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkGetSecretRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkGetSecretRequest;
    static equals(a: BulkGetSecretRequest | PlainMessage<BulkGetSecretRequest> | undefined, b: BulkGetSecretRequest | PlainMessage<BulkGetSecretRequest> | undefined): boolean;
}
/**
 * BulkGet 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.BulkGetSecretResponse
 */
export declare class BulkGetSecretResponse extends Message<BulkGetSecretResponse> {
    /**
     * 結果マップ（シークレット名 → GetSecretResponse）
     *
     * @generated from field: map<string, k1s0.tier1.secrets.v1.GetSecretResponse> results = 1;
     */
    results: {
        [key: string]: GetSecretResponse;
    };
    constructor(data?: PartialMessage<BulkGetSecretResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.BulkGetSecretResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkGetSecretResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkGetSecretResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkGetSecretResponse;
    static equals(a: BulkGetSecretResponse | PlainMessage<BulkGetSecretResponse> | undefined, b: BulkGetSecretResponse | PlainMessage<BulkGetSecretResponse> | undefined): boolean;
}
/**
 * Rotate リクエスト
 *
 * @generated from message k1s0.tier1.secrets.v1.RotateSecretRequest
 */
export declare class RotateSecretRequest extends Message<RotateSecretRequest> {
    /**
     * ローテーション対象シークレット名
     *
     * @generated from field: string name = 1;
     */
    name: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    /**
     * 旧バージョンの猶予時間（0 は即無効、既定 3600 秒）
     * tier2 側の接続プール drain 時間を想定
     *
     * @generated from field: int32 grace_period_sec = 3;
     */
    gracePeriodSec: number;
    /**
     * 動的シークレット（DB 資格情報等）の場合の発行ポリシー名
     *
     * @generated from field: optional string policy = 4;
     */
    policy?: string;
    /**
     * 冪等性キー（同一キーでの再試行は同じ new_version を返す）
     *
     * @generated from field: string idempotency_key = 5;
     */
    idempotencyKey: string;
    constructor(data?: PartialMessage<RotateSecretRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.RotateSecretRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RotateSecretRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RotateSecretRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RotateSecretRequest;
    static equals(a: RotateSecretRequest | PlainMessage<RotateSecretRequest> | undefined, b: RotateSecretRequest | PlainMessage<RotateSecretRequest> | undefined): boolean;
}
/**
 * Rotate 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.RotateSecretResponse
 */
export declare class RotateSecretResponse extends Message<RotateSecretResponse> {
    /**
     * ローテーション後の新バージョン
     *
     * @generated from field: int32 new_version = 1;
     */
    newVersion: number;
    /**
     * 旧バージョン（grace_period_sec まで Get 可能）
     *
     * @generated from field: int32 previous_version = 2;
     */
    previousVersion: number;
    /**
     * 新バージョン発効時刻（Unix epoch ミリ秒）
     *
     * @generated from field: int64 rotated_at_ms = 3;
     */
    rotatedAtMs: bigint;
    /**
     * 動的シークレット時の TTL（静的シークレットでは 0）
     *
     * @generated from field: int32 ttl_sec = 4;
     */
    ttlSec: number;
    constructor(data?: PartialMessage<RotateSecretResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.RotateSecretResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RotateSecretResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RotateSecretResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RotateSecretResponse;
    static equals(a: RotateSecretResponse | PlainMessage<RotateSecretResponse> | undefined, b: RotateSecretResponse | PlainMessage<RotateSecretResponse> | undefined): boolean;
}
//# sourceMappingURL=secrets_service_pb.d.ts.map