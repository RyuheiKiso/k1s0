import { EmitMetricRequest, EmitMetricResponse, EmitSpanRequest, EmitSpanResponse } from "./telemetry_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * Telemetry API。OTel Collector → Mimir / Tempo に転送する経路で使う。
 *
 * @generated from service k1s0.tier1.telemetry.v1.TelemetryService
 */
export declare const TelemetryService: {
    readonly typeName: "k1s0.tier1.telemetry.v1.TelemetryService";
    readonly methods: {
        /**
         * メトリクス送信（Counter / Gauge / Histogram の混在可）
         *
         * @generated from rpc k1s0.tier1.telemetry.v1.TelemetryService.EmitMetric
         */
        readonly emitMetric: {
            readonly name: "EmitMetric";
            readonly I: typeof EmitMetricRequest;
            readonly O: typeof EmitMetricResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * Span 送信（既に終了済みの Span のみ受け付ける、開始 Span は OTel SDK で）
         *
         * @generated from rpc k1s0.tier1.telemetry.v1.TelemetryService.EmitSpan
         */
        readonly emitSpan: {
            readonly name: "EmitSpan";
            readonly I: typeof EmitSpanRequest;
            readonly O: typeof EmitSpanResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=telemetry_service_connect.d.ts.map