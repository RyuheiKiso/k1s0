// 本ファイルは k1s0 TypeScript SDK の Telemetry 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { TelemetryService } from "./proto/k1s0/tier1/telemetry/v1/telemetry_service_connect.js";
import type { Metric, Span } from "./proto/k1s0/tier1/telemetry/v1/telemetry_service_pb.js";

/** TelemetryFacade は TelemetryService の動詞統一 facade。 */
export class TelemetryFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  /** emitMetric はメトリクス送信。 */
  async emitMetric(metrics: Metric[]): Promise<void> {
    const raw = createPromiseClient(TelemetryService, this.client.transport);
    await raw.emitMetric({ metrics, context: this.client.tenantContext() });
  }

  /** emitSpan は Span 送信。 */
  async emitSpan(spans: Span[]): Promise<void> {
    const raw = createPromiseClient(TelemetryService, this.client.transport);
    await raw.emitSpan({ spans, context: this.client.tenantContext() });
  }
}
