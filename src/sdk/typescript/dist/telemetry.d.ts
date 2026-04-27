import type { K1s0Client } from "./client.js";
import type { Metric, Span } from "./proto/k1s0/tier1/telemetry/v1/telemetry_service_pb.js";
/** TelemetryFacade は TelemetryService の動詞統一 facade。 */
export declare class TelemetryFacade {
    private readonly client;
    constructor(client: K1s0Client);
    /** emitMetric はメトリクス送信。 */
    emitMetric(metrics: Metric[]): Promise<void>;
    /** emitSpan は Span 送信。 */
    emitSpan(spans: Span[]): Promise<void>;
}
//# sourceMappingURL=telemetry.d.ts.map