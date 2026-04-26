import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3, Timestamp } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * 監査イベント
 *
 * @generated from message k1s0.tier1.audit.v1.AuditEvent
 */
export declare class AuditEvent extends Message<AuditEvent> {
    /**
     * 発生時刻（UTC）
     *
     * @generated from field: google.protobuf.Timestamp timestamp = 1;
     */
    timestamp?: Timestamp;
    /**
     * 操作主体（user_id / workload_id）
     *
     * @generated from field: string actor = 2;
     */
    actor: string;
    /**
     * 操作種別（CREATE / READ / UPDATE / DELETE / LOGIN / EXPORT）
     *
     * @generated from field: string action = 3;
     */
    action: string;
    /**
     * 対象リソース（URN 形式: k1s0:tenant:<tid>:resource:<type>/<id>）
     *
     * @generated from field: string resource = 4;
     */
    resource: string;
    /**
     * 操作結果（SUCCESS / DENIED / ERROR）
     *
     * @generated from field: string outcome = 5;
     */
    outcome: string;
    /**
     * 追加コンテキスト（ip / user_agent / request_id 等）
     *
     * @generated from field: map<string, string> attributes = 6;
     */
    attributes: {
        [key: string]: string;
    };
    constructor(data?: PartialMessage<AuditEvent>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.AuditEvent";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): AuditEvent;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): AuditEvent;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): AuditEvent;
    static equals(a: AuditEvent | PlainMessage<AuditEvent> | undefined, b: AuditEvent | PlainMessage<AuditEvent> | undefined): boolean;
}
/**
 * Record リクエスト
 *
 * @generated from message k1s0.tier1.audit.v1.RecordAuditRequest
 */
export declare class RecordAuditRequest extends Message<RecordAuditRequest> {
    /**
     * 記録対象イベント
     *
     * @generated from field: k1s0.tier1.audit.v1.AuditEvent event = 1;
     */
    event?: AuditEvent;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<RecordAuditRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.RecordAuditRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RecordAuditRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RecordAuditRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RecordAuditRequest;
    static equals(a: RecordAuditRequest | PlainMessage<RecordAuditRequest> | undefined, b: RecordAuditRequest | PlainMessage<RecordAuditRequest> | undefined): boolean;
}
/**
 * Record 応答
 *
 * @generated from message k1s0.tier1.audit.v1.RecordAuditResponse
 */
export declare class RecordAuditResponse extends Message<RecordAuditResponse> {
    /**
     * WORM ストアでの固有 ID（再現性のあるハッシュ含む）
     *
     * @generated from field: string audit_id = 1;
     */
    auditId: string;
    constructor(data?: PartialMessage<RecordAuditResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.RecordAuditResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RecordAuditResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RecordAuditResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RecordAuditResponse;
    static equals(a: RecordAuditResponse | PlainMessage<RecordAuditResponse> | undefined, b: RecordAuditResponse | PlainMessage<RecordAuditResponse> | undefined): boolean;
}
/**
 * Query リクエスト
 *
 * @generated from message k1s0.tier1.audit.v1.QueryAuditRequest
 */
export declare class QueryAuditRequest extends Message<QueryAuditRequest> {
    /**
     * 範囲開始
     *
     * @generated from field: google.protobuf.Timestamp from = 1;
     */
    from?: Timestamp;
    /**
     * 範囲終了
     *
     * @generated from field: google.protobuf.Timestamp to = 2;
     */
    to?: Timestamp;
    /**
     * フィルタ（任意の attributes 等価一致、AND 結合）
     *
     * @generated from field: map<string, string> filters = 3;
     */
    filters: {
        [key: string]: string;
    };
    /**
     * 件数上限（既定 100、最大 1000）
     *
     * @generated from field: int32 limit = 4;
     */
    limit: number;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<QueryAuditRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.QueryAuditRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): QueryAuditRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): QueryAuditRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): QueryAuditRequest;
    static equals(a: QueryAuditRequest | PlainMessage<QueryAuditRequest> | undefined, b: QueryAuditRequest | PlainMessage<QueryAuditRequest> | undefined): boolean;
}
/**
 * Query 応答
 *
 * @generated from message k1s0.tier1.audit.v1.QueryAuditResponse
 */
export declare class QueryAuditResponse extends Message<QueryAuditResponse> {
    /**
     * 検索結果（時刻昇順、出力時に PII Mask 自動適用）
     *
     * @generated from field: repeated k1s0.tier1.audit.v1.AuditEvent events = 1;
     */
    events: AuditEvent[];
    constructor(data?: PartialMessage<QueryAuditResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.QueryAuditResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): QueryAuditResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): QueryAuditResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): QueryAuditResponse;
    static equals(a: QueryAuditResponse | PlainMessage<QueryAuditResponse> | undefined, b: QueryAuditResponse | PlainMessage<QueryAuditResponse> | undefined): boolean;
}
//# sourceMappingURL=audit_service_pb.d.ts.map