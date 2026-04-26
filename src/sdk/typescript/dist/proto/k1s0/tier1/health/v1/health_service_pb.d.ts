import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
/**
 * Liveness probe のリクエスト（現状フィールドなし、将来拡張用）
 *
 * @generated from message k1s0.tier1.health.v1.LivenessRequest
 */
export declare class LivenessRequest extends Message<LivenessRequest> {
    constructor(data?: PartialMessage<LivenessRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.health.v1.LivenessRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): LivenessRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): LivenessRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): LivenessRequest;
    static equals(a: LivenessRequest | PlainMessage<LivenessRequest> | undefined, b: LivenessRequest | PlainMessage<LivenessRequest> | undefined): boolean;
}
/**
 * Readiness probe のリクエスト（現状フィールドなし、将来拡張用）
 *
 * @generated from message k1s0.tier1.health.v1.ReadinessRequest
 */
export declare class ReadinessRequest extends Message<ReadinessRequest> {
    constructor(data?: PartialMessage<ReadinessRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.health.v1.ReadinessRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ReadinessRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ReadinessRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ReadinessRequest;
    static equals(a: ReadinessRequest | PlainMessage<ReadinessRequest> | undefined, b: ReadinessRequest | PlainMessage<ReadinessRequest> | undefined): boolean;
}
/**
 * Liveness response: process の生存と version 情報を返す。
 *
 * @generated from message k1s0.tier1.health.v1.LivenessResponse
 */
export declare class LivenessResponse extends Message<LivenessResponse> {
    /**
     * tier1 facade のビルドバージョン（SemVer）
     *
     * @generated from field: string version = 1;
     */
    version: string;
    /**
     * 起動時刻からの経過時間（秒）
     *
     * @generated from field: int64 uptime_seconds = 2;
     */
    uptimeSeconds: bigint;
    constructor(data?: PartialMessage<LivenessResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.health.v1.LivenessResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): LivenessResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): LivenessResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): LivenessResponse;
    static equals(a: LivenessResponse | PlainMessage<LivenessResponse> | undefined, b: LivenessResponse | PlainMessage<LivenessResponse> | undefined): boolean;
}
/**
 * Readiness response: 各依存の状態を個別に返す。
 *
 * @generated from message k1s0.tier1.health.v1.ReadinessResponse
 */
export declare class ReadinessResponse extends Message<ReadinessResponse> {
    /**
     * 全体としての ready 判定（依存すべて OK なら true）
     *
     * @generated from field: bool ready = 1;
     */
    ready: boolean;
    /**
     * 各依存（postgres / kafka / openbao / keycloak / 等）の個別状態
     *
     * @generated from field: map<string, k1s0.tier1.health.v1.DependencyStatus> dependencies = 2;
     */
    dependencies: {
        [key: string]: DependencyStatus;
    };
    constructor(data?: PartialMessage<ReadinessResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.health.v1.ReadinessResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ReadinessResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ReadinessResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ReadinessResponse;
    static equals(a: ReadinessResponse | PlainMessage<ReadinessResponse> | undefined, b: ReadinessResponse | PlainMessage<ReadinessResponse> | undefined): boolean;
}
/**
 * 個別依存の状態
 *
 * @generated from message k1s0.tier1.health.v1.DependencyStatus
 */
export declare class DependencyStatus extends Message<DependencyStatus> {
    /**
     * 接続可能か
     *
     * @generated from field: bool reachable = 1;
     */
    reachable: boolean;
    /**
     * 直近のエラー（reachable=false の時のみ意味を持つ）
     *
     * @generated from field: string error_message = 2;
     */
    errorMessage: string;
    constructor(data?: PartialMessage<DependencyStatus>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.health.v1.DependencyStatus";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): DependencyStatus;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): DependencyStatus;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): DependencyStatus;
    static equals(a: DependencyStatus | PlainMessage<DependencyStatus> | undefined, b: DependencyStatus | PlainMessage<DependencyStatus> | undefined): boolean;
}
//# sourceMappingURL=health_service_pb.d.ts.map