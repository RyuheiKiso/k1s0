import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3, Timestamp } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * 重大度（OpenTelemetry Log Severity の数値仕様と整合）。
 * 注: 数値タグは OTel 仕様（trace=0 / debug=5 / info=9 / warn=13 / error=17 / fatal=21）に
 *     固定されているため、buf STANDARD lint の ENUM_ZERO_VALUE_SUFFIX /
 *     ENUM_VALUE_PREFIX を ignore する。
 * buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
 * buf:lint:ignore ENUM_VALUE_PREFIX
 *
 * @generated from enum k1s0.tier1.log.v1.Severity
 */
export declare enum Severity {
    /**
     * OTel SeverityNumber TRACE（既定値）
     *
     * @generated from enum value: TRACE = 0;
     */
    TRACE = 0,
    /**
     * OTel SeverityNumber DEBUG
     *
     * @generated from enum value: DEBUG = 5;
     */
    DEBUG = 5,
    /**
     * OTel SeverityNumber INFO
     *
     * @generated from enum value: INFO = 9;
     */
    INFO = 9,
    /**
     * OTel SeverityNumber WARN
     *
     * @generated from enum value: WARN = 13;
     */
    WARN = 13,
    /**
     * OTel SeverityNumber ERROR
     *
     * @generated from enum value: ERROR = 17;
     */
    ERROR = 17,
    /**
     * OTel SeverityNumber FATAL
     *
     * @generated from enum value: FATAL = 21;
     */
    FATAL = 21
}
/**
 * LogEntry（OTel LogRecord と等価）
 *
 * @generated from message k1s0.tier1.log.v1.LogEntry
 */
export declare class LogEntry extends Message<LogEntry> {
    /**
     * 発生時刻（UTC、tier2 側で OTel SDK 経由付与）
     *
     * @generated from field: google.protobuf.Timestamp timestamp = 1;
     */
    timestamp?: Timestamp;
    /**
     * 重大度
     *
     * @generated from field: k1s0.tier1.log.v1.Severity severity = 2;
     */
    severity: Severity;
    /**
     * メッセージ本文（PII 自動検出対象、Pii API で Mask 推奨）
     *
     * @generated from field: string body = 3;
     */
    body: string;
    /**
     * 属性（service.name / env / trace_id / span_id を含む）
     *
     * @generated from field: map<string, string> attributes = 4;
     */
    attributes: {
        [key: string]: string;
    };
    /**
     * 関連する例外スタック（オプション、マルチライン許容）
     *
     * @generated from field: string stack_trace = 5;
     */
    stackTrace: string;
    constructor(data?: PartialMessage<LogEntry>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.log.v1.LogEntry";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): LogEntry;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): LogEntry;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): LogEntry;
    static equals(a: LogEntry | PlainMessage<LogEntry> | undefined, b: LogEntry | PlainMessage<LogEntry> | undefined): boolean;
}
/**
 * Send リクエスト
 *
 * @generated from message k1s0.tier1.log.v1.SendLogRequest
 */
export declare class SendLogRequest extends Message<SendLogRequest> {
    /**
     * 送信エントリ
     *
     * @generated from field: k1s0.tier1.log.v1.LogEntry entry = 1;
     */
    entry?: LogEntry;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<SendLogRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.log.v1.SendLogRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): SendLogRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): SendLogRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): SendLogRequest;
    static equals(a: SendLogRequest | PlainMessage<SendLogRequest> | undefined, b: SendLogRequest | PlainMessage<SendLogRequest> | undefined): boolean;
}
/**
 * Send 応答
 *
 * @generated from message k1s0.tier1.log.v1.SendLogResponse
 */
export declare class SendLogResponse extends Message<SendLogResponse> {
    constructor(data?: PartialMessage<SendLogResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.log.v1.SendLogResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): SendLogResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): SendLogResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): SendLogResponse;
    static equals(a: SendLogResponse | PlainMessage<SendLogResponse> | undefined, b: SendLogResponse | PlainMessage<SendLogResponse> | undefined): boolean;
}
/**
 * BulkSend リクエスト
 *
 * @generated from message k1s0.tier1.log.v1.BulkSendLogRequest
 */
export declare class BulkSendLogRequest extends Message<BulkSendLogRequest> {
    /**
     * 送信エントリ列
     *
     * @generated from field: repeated k1s0.tier1.log.v1.LogEntry entries = 1;
     */
    entries: LogEntry[];
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<BulkSendLogRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.log.v1.BulkSendLogRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkSendLogRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkSendLogRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkSendLogRequest;
    static equals(a: BulkSendLogRequest | PlainMessage<BulkSendLogRequest> | undefined, b: BulkSendLogRequest | PlainMessage<BulkSendLogRequest> | undefined): boolean;
}
/**
 * BulkSend 応答
 *
 * @generated from message k1s0.tier1.log.v1.BulkSendLogResponse
 */
export declare class BulkSendLogResponse extends Message<BulkSendLogResponse> {
    /**
     * 受理件数（OTel パイプラインに渡された件数）
     *
     * @generated from field: int32 accepted = 1;
     */
    accepted: number;
    /**
     * 拒否件数（PII フィルタや schema 違反で却下された件数）
     *
     * @generated from field: int32 rejected = 2;
     */
    rejected: number;
    constructor(data?: PartialMessage<BulkSendLogResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.log.v1.BulkSendLogResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkSendLogResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkSendLogResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkSendLogResponse;
    static equals(a: BulkSendLogResponse | PlainMessage<BulkSendLogResponse> | undefined, b: BulkSendLogResponse | PlainMessage<BulkSendLogResponse> | undefined): boolean;
}
//# sourceMappingURL=log_service_pb.d.ts.map