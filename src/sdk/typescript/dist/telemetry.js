// 本ファイルは k1s0 TypeScript SDK の Telemetry 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import { TelemetryService } from "./proto/k1s0/tier1/telemetry/v1/telemetry_service_connect.js";
/** TelemetryFacade は TelemetryService の動詞統一 facade。 */
export class TelemetryFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    /** emitMetric はメトリクス送信。 */
    async emitMetric(metrics) {
        const raw = createPromiseClient(TelemetryService, this.client.transport);
        await raw.emitMetric({ metrics, context: this.client.tenantContext() });
    }
    /** emitSpan は Span 送信。 */
    async emitSpan(spans) {
        const raw = createPromiseClient(TelemetryService, this.client.transport);
        await raw.emitSpan({ spans, context: this.client.tenantContext() });
    }
}
//# sourceMappingURL=telemetry.js.map