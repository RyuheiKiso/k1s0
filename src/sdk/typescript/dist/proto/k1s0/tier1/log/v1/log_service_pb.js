// 本ファイルは tier1 公開 Log API の正式 proto。
// 構造化ログ送信（OpenTelemetry Logs 準拠）を提供する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/07_Log_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/07_Log_API.md
//
// 関連要件: FR-T1-LOG-001〜004
// proto 構文宣言（proto3）
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
export var Severity;
(function (Severity) {
    /**
     * OTel SeverityNumber TRACE（既定値）
     *
     * @generated from enum value: TRACE = 0;
     */
    Severity[Severity["TRACE"] = 0] = "TRACE";
    /**
     * OTel SeverityNumber DEBUG
     *
     * @generated from enum value: DEBUG = 5;
     */
    Severity[Severity["DEBUG"] = 5] = "DEBUG";
    /**
     * OTel SeverityNumber INFO
     *
     * @generated from enum value: INFO = 9;
     */
    Severity[Severity["INFO"] = 9] = "INFO";
    /**
     * OTel SeverityNumber WARN
     *
     * @generated from enum value: WARN = 13;
     */
    Severity[Severity["WARN"] = 13] = "WARN";
    /**
     * OTel SeverityNumber ERROR
     *
     * @generated from enum value: ERROR = 17;
     */
    Severity[Severity["ERROR"] = 17] = "ERROR";
    /**
     * OTel SeverityNumber FATAL
     *
     * @generated from enum value: FATAL = 21;
     */
    Severity[Severity["FATAL"] = 21] = "FATAL";
})(Severity || (Severity = {}));
// Retrieve enum metadata with: proto3.getEnumType(Severity)
proto3.util.setEnumType(Severity, "k1s0.tier1.log.v1.Severity", [
    { no: 0, name: "TRACE" },
    { no: 5, name: "DEBUG" },
    { no: 9, name: "INFO" },
    { no: 13, name: "WARN" },
    { no: 17, name: "ERROR" },
    { no: 21, name: "FATAL" },
]);
/**
 * LogEntry（OTel LogRecord と等価）
 *
 * @generated from message k1s0.tier1.log.v1.LogEntry
 */
export class LogEntry extends Message {
    /**
     * 発生時刻（UTC、tier2 側で OTel SDK 経由付与）
     *
     * @generated from field: google.protobuf.Timestamp timestamp = 1;
     */
    timestamp;
    /**
     * 重大度
     *
     * @generated from field: k1s0.tier1.log.v1.Severity severity = 2;
     */
    severity = Severity.TRACE;
    /**
     * メッセージ本文（PII 自動検出対象、Pii API で Mask 推奨）
     *
     * @generated from field: string body = 3;
     */
    body = "";
    /**
     * 属性（service.name / env / trace_id / span_id を含む）
     *
     * @generated from field: map<string, string> attributes = 4;
     */
    attributes = {};
    /**
     * 関連する例外スタック（オプション、マルチライン許容）
     *
     * @generated from field: string stack_trace = 5;
     */
    stackTrace = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.log.v1.LogEntry";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "timestamp", kind: "message", T: Timestamp },
        { no: 2, name: "severity", kind: "enum", T: proto3.getEnumType(Severity) },
        { no: 3, name: "body", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 4, name: "attributes", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
        { no: 5, name: "stack_trace", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new LogEntry().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new LogEntry().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new LogEntry().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(LogEntry, a, b);
    }
}
/**
 * Send リクエスト
 *
 * @generated from message k1s0.tier1.log.v1.SendLogRequest
 */
export class SendLogRequest extends Message {
    /**
     * 送信エントリ
     *
     * @generated from field: k1s0.tier1.log.v1.LogEntry entry = 1;
     */
    entry;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.log.v1.SendLogRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "entry", kind: "message", T: LogEntry },
        { no: 2, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new SendLogRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new SendLogRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new SendLogRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(SendLogRequest, a, b);
    }
}
/**
 * Send 応答
 *
 * @generated from message k1s0.tier1.log.v1.SendLogResponse
 */
export class SendLogResponse extends Message {
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.log.v1.SendLogResponse";
    static fields = proto3.util.newFieldList(() => []);
    static fromBinary(bytes, options) {
        return new SendLogResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new SendLogResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new SendLogResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(SendLogResponse, a, b);
    }
}
/**
 * BulkSend リクエスト
 *
 * @generated from message k1s0.tier1.log.v1.BulkSendLogRequest
 */
export class BulkSendLogRequest extends Message {
    /**
     * 送信エントリ列
     *
     * @generated from field: repeated k1s0.tier1.log.v1.LogEntry entries = 1;
     */
    entries = [];
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.log.v1.BulkSendLogRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "entries", kind: "message", T: LogEntry, repeated: true },
        { no: 2, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new BulkSendLogRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BulkSendLogRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BulkSendLogRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BulkSendLogRequest, a, b);
    }
}
/**
 * BulkSend 応答
 *
 * @generated from message k1s0.tier1.log.v1.BulkSendLogResponse
 */
export class BulkSendLogResponse extends Message {
    /**
     * 受理件数（OTel パイプラインに渡された件数）
     *
     * @generated from field: int32 accepted = 1;
     */
    accepted = 0;
    /**
     * 拒否件数（PII フィルタや schema 違反で却下された件数）
     *
     * @generated from field: int32 rejected = 2;
     */
    rejected = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.log.v1.BulkSendLogResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "accepted", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 2, name: "rejected", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
    ]);
    static fromBinary(bytes, options) {
        return new BulkSendLogResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BulkSendLogResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BulkSendLogResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BulkSendLogResponse, a, b);
    }
}
//# sourceMappingURL=log_service_pb.js.map