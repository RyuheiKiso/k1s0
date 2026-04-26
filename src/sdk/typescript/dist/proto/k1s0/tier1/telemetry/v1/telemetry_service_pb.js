// 本ファイルは tier1 公開 Telemetry API の正式 proto。
// メトリクス（Counter / Gauge / Histogram）と分散トレース Span 送信を提供する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/08_Telemetry_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/08_Telemetry_API.md
//
// 関連要件: FR-T1-TELEMETRY-001〜004
// proto 構文宣言（proto3）
import { Message, proto3, Timestamp } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * メトリクス種別。
 * 注: 正典 IDL は zero value を `COUNTER = 0` と定義しているため、
 *     buf STANDARD lint の ENUM_ZERO_VALUE_SUFFIX / ENUM_VALUE_PREFIX を ignore する。
 * buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
 * buf:lint:ignore ENUM_VALUE_PREFIX
 *
 * @generated from enum k1s0.tier1.telemetry.v1.MetricKind
 */
export var MetricKind;
(function (MetricKind) {
    /**
     * 単調増加カウンタ（Prometheus _total メトリクスに対応、既定値）
     *
     * @generated from enum value: COUNTER = 0;
     */
    MetricKind[MetricKind["COUNTER"] = 0] = "COUNTER";
    /**
     * 上下する瞬時値ゲージ
     *
     * @generated from enum value: GAUGE = 1;
     */
    MetricKind[MetricKind["GAUGE"] = 1] = "GAUGE";
    /**
     * 分布ヒストグラム（quantile / bucket 計算用）
     *
     * @generated from enum value: HISTOGRAM = 2;
     */
    MetricKind[MetricKind["HISTOGRAM"] = 2] = "HISTOGRAM";
})(MetricKind || (MetricKind = {}));
// Retrieve enum metadata with: proto3.getEnumType(MetricKind)
proto3.util.setEnumType(MetricKind, "k1s0.tier1.telemetry.v1.MetricKind", [
    { no: 0, name: "COUNTER" },
    { no: 1, name: "GAUGE" },
    { no: 2, name: "HISTOGRAM" },
]);
/**
 * 単一メトリクス
 *
 * @generated from message k1s0.tier1.telemetry.v1.Metric
 */
export class Metric extends Message {
    /**
     * メトリクス名（OTel 慣行に従いドット区切り、例: k1s0.tier1.invoke.duration_ms）
     *
     * @generated from field: string name = 1;
     */
    name = "";
    /**
     * メトリクス種別
     *
     * @generated from field: k1s0.tier1.telemetry.v1.MetricKind kind = 2;
     */
    kind = MetricKind.COUNTER;
    /**
     * 値（Counter は加算、Gauge は瞬時値、Histogram は観測値）
     *
     * @generated from field: double value = 3;
     */
    value = 0;
    /**
     * ラベル（service.name / env / status_code 等の OTel attribute）
     *
     * @generated from field: map<string, string> labels = 4;
     */
    labels = {};
    /**
     * タイムスタンプ（観測時刻、UTC）
     *
     * @generated from field: google.protobuf.Timestamp timestamp = 5;
     */
    timestamp;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.telemetry.v1.Metric";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "name", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "kind", kind: "enum", T: proto3.getEnumType(MetricKind) },
        { no: 3, name: "value", kind: "scalar", T: 1 /* ScalarType.DOUBLE */ },
        { no: 4, name: "labels", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
        { no: 5, name: "timestamp", kind: "message", T: Timestamp },
    ]);
    static fromBinary(bytes, options) {
        return new Metric().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new Metric().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new Metric().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(Metric, a, b);
    }
}
/**
 * EmitMetric リクエスト
 *
 * @generated from message k1s0.tier1.telemetry.v1.EmitMetricRequest
 */
export class EmitMetricRequest extends Message {
    /**
     * メトリクス列
     *
     * @generated from field: repeated k1s0.tier1.telemetry.v1.Metric metrics = 1;
     */
    metrics = [];
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
    static typeName = "k1s0.tier1.telemetry.v1.EmitMetricRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "metrics", kind: "message", T: Metric, repeated: true },
        { no: 2, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new EmitMetricRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new EmitMetricRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new EmitMetricRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(EmitMetricRequest, a, b);
    }
}
/**
 * EmitMetric 応答
 *
 * @generated from message k1s0.tier1.telemetry.v1.EmitMetricResponse
 */
export class EmitMetricResponse extends Message {
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.telemetry.v1.EmitMetricResponse";
    static fields = proto3.util.newFieldList(() => []);
    static fromBinary(bytes, options) {
        return new EmitMetricResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new EmitMetricResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new EmitMetricResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(EmitMetricResponse, a, b);
    }
}
/**
 * 単一 Span
 *
 * @generated from message k1s0.tier1.telemetry.v1.Span
 */
export class Span extends Message {
    /**
     * トレース ID（W3C Trace Context、16 byte 相当の 32 文字 hex）
     *
     * @generated from field: string trace_id = 1;
     */
    traceId = "";
    /**
     * Span ID（8 byte 相当の 16 文字 hex）
     *
     * @generated from field: string span_id = 2;
     */
    spanId = "";
    /**
     * 親 Span ID（ルート Span は空文字列）
     *
     * @generated from field: string parent_span_id = 3;
     */
    parentSpanId = "";
    /**
     * Span 名（操作名、例: HTTP GET /api/v1/foo）
     *
     * @generated from field: string name = 4;
     */
    name = "";
    /**
     * 開始時刻
     *
     * @generated from field: google.protobuf.Timestamp start_time = 5;
     */
    startTime;
    /**
     * 終了時刻
     *
     * @generated from field: google.protobuf.Timestamp end_time = 6;
     */
    endTime;
    /**
     * 属性（http.method / db.system 等の OTel semantic conventions）
     *
     * @generated from field: map<string, string> attributes = 7;
     */
    attributes = {};
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.telemetry.v1.Span";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "trace_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "span_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "parent_span_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 4, name: "name", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 5, name: "start_time", kind: "message", T: Timestamp },
        { no: 6, name: "end_time", kind: "message", T: Timestamp },
        { no: 7, name: "attributes", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
    ]);
    static fromBinary(bytes, options) {
        return new Span().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new Span().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new Span().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(Span, a, b);
    }
}
/**
 * EmitSpan リクエスト
 *
 * @generated from message k1s0.tier1.telemetry.v1.EmitSpanRequest
 */
export class EmitSpanRequest extends Message {
    /**
     * Span 列
     *
     * @generated from field: repeated k1s0.tier1.telemetry.v1.Span spans = 1;
     */
    spans = [];
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
    static typeName = "k1s0.tier1.telemetry.v1.EmitSpanRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "spans", kind: "message", T: Span, repeated: true },
        { no: 2, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new EmitSpanRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new EmitSpanRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new EmitSpanRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(EmitSpanRequest, a, b);
    }
}
/**
 * EmitSpan 応答
 *
 * @generated from message k1s0.tier1.telemetry.v1.EmitSpanResponse
 */
export class EmitSpanResponse extends Message {
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.telemetry.v1.EmitSpanResponse";
    static fields = proto3.util.newFieldList(() => []);
    static fromBinary(bytes, options) {
        return new EmitSpanResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new EmitSpanResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new EmitSpanResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(EmitSpanResponse, a, b);
    }
}
//# sourceMappingURL=telemetry_service_pb.js.map