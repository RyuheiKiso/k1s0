import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
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
export declare enum MetricKind {
    /**
     * 単調増加カウンタ（Prometheus _total メトリクスに対応、既定値）
     *
     * @generated from enum value: COUNTER = 0;
     */
    COUNTER = 0,
    /**
     * 上下する瞬時値ゲージ
     *
     * @generated from enum value: GAUGE = 1;
     */
    GAUGE = 1,
    /**
     * 分布ヒストグラム（quantile / bucket 計算用）
     *
     * @generated from enum value: HISTOGRAM = 2;
     */
    HISTOGRAM = 2
}
/**
 * 単一メトリクス
 *
 * @generated from message k1s0.tier1.telemetry.v1.Metric
 */
export declare class Metric extends Message<Metric> {
    /**
     * メトリクス名（OTel 慣行に従いドット区切り、例: k1s0.tier1.invoke.duration_ms）
     *
     * @generated from field: string name = 1;
     */
    name: string;
    /**
     * メトリクス種別
     *
     * @generated from field: k1s0.tier1.telemetry.v1.MetricKind kind = 2;
     */
    kind: MetricKind;
    /**
     * 値（Counter は加算、Gauge は瞬時値、Histogram は観測値）
     *
     * @generated from field: double value = 3;
     */
    value: number;
    /**
     * ラベル（service.name / env / status_code 等の OTel attribute）
     *
     * @generated from field: map<string, string> labels = 4;
     */
    labels: {
        [key: string]: string;
    };
    /**
     * タイムスタンプ（観測時刻、UTC）
     *
     * @generated from field: google.protobuf.Timestamp timestamp = 5;
     */
    timestamp?: Timestamp;
    constructor(data?: PartialMessage<Metric>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.telemetry.v1.Metric";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): Metric;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): Metric;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): Metric;
    static equals(a: Metric | PlainMessage<Metric> | undefined, b: Metric | PlainMessage<Metric> | undefined): boolean;
}
/**
 * EmitMetric リクエスト
 *
 * @generated from message k1s0.tier1.telemetry.v1.EmitMetricRequest
 */
export declare class EmitMetricRequest extends Message<EmitMetricRequest> {
    /**
     * メトリクス列
     *
     * @generated from field: repeated k1s0.tier1.telemetry.v1.Metric metrics = 1;
     */
    metrics: Metric[];
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<EmitMetricRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.telemetry.v1.EmitMetricRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): EmitMetricRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): EmitMetricRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): EmitMetricRequest;
    static equals(a: EmitMetricRequest | PlainMessage<EmitMetricRequest> | undefined, b: EmitMetricRequest | PlainMessage<EmitMetricRequest> | undefined): boolean;
}
/**
 * EmitMetric 応答
 *
 * @generated from message k1s0.tier1.telemetry.v1.EmitMetricResponse
 */
export declare class EmitMetricResponse extends Message<EmitMetricResponse> {
    constructor(data?: PartialMessage<EmitMetricResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.telemetry.v1.EmitMetricResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): EmitMetricResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): EmitMetricResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): EmitMetricResponse;
    static equals(a: EmitMetricResponse | PlainMessage<EmitMetricResponse> | undefined, b: EmitMetricResponse | PlainMessage<EmitMetricResponse> | undefined): boolean;
}
/**
 * 単一 Span
 *
 * @generated from message k1s0.tier1.telemetry.v1.Span
 */
export declare class Span extends Message<Span> {
    /**
     * トレース ID（W3C Trace Context、16 byte 相当の 32 文字 hex）
     *
     * @generated from field: string trace_id = 1;
     */
    traceId: string;
    /**
     * Span ID（8 byte 相当の 16 文字 hex）
     *
     * @generated from field: string span_id = 2;
     */
    spanId: string;
    /**
     * 親 Span ID（ルート Span は空文字列）
     *
     * @generated from field: string parent_span_id = 3;
     */
    parentSpanId: string;
    /**
     * Span 名（操作名、例: HTTP GET /api/v1/foo）
     *
     * @generated from field: string name = 4;
     */
    name: string;
    /**
     * 開始時刻
     *
     * @generated from field: google.protobuf.Timestamp start_time = 5;
     */
    startTime?: Timestamp;
    /**
     * 終了時刻
     *
     * @generated from field: google.protobuf.Timestamp end_time = 6;
     */
    endTime?: Timestamp;
    /**
     * 属性（http.method / db.system 等の OTel semantic conventions）
     *
     * @generated from field: map<string, string> attributes = 7;
     */
    attributes: {
        [key: string]: string;
    };
    constructor(data?: PartialMessage<Span>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.telemetry.v1.Span";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): Span;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): Span;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): Span;
    static equals(a: Span | PlainMessage<Span> | undefined, b: Span | PlainMessage<Span> | undefined): boolean;
}
/**
 * EmitSpan リクエスト
 *
 * @generated from message k1s0.tier1.telemetry.v1.EmitSpanRequest
 */
export declare class EmitSpanRequest extends Message<EmitSpanRequest> {
    /**
     * Span 列
     *
     * @generated from field: repeated k1s0.tier1.telemetry.v1.Span spans = 1;
     */
    spans: Span[];
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<EmitSpanRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.telemetry.v1.EmitSpanRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): EmitSpanRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): EmitSpanRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): EmitSpanRequest;
    static equals(a: EmitSpanRequest | PlainMessage<EmitSpanRequest> | undefined, b: EmitSpanRequest | PlainMessage<EmitSpanRequest> | undefined): boolean;
}
/**
 * EmitSpan 応答
 *
 * @generated from message k1s0.tier1.telemetry.v1.EmitSpanResponse
 */
export declare class EmitSpanResponse extends Message<EmitSpanResponse> {
    constructor(data?: PartialMessage<EmitSpanResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.telemetry.v1.EmitSpanResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): EmitSpanResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): EmitSpanResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): EmitSpanResponse;
    static equals(a: EmitSpanResponse | PlainMessage<EmitSpanResponse> | undefined, b: EmitSpanResponse | PlainMessage<EmitSpanResponse> | undefined): boolean;
}
//# sourceMappingURL=telemetry_service_pb.d.ts.map