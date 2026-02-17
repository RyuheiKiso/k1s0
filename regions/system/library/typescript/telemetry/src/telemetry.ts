import { NodeSDK } from '@opentelemetry/sdk-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';

/**
 * TelemetryConfig は telemetry ライブラリの初期化設定を定義する。
 */
export interface TelemetryConfig {
  serviceName: string;
  version: string;
  tier: string;
  environment: string;
  traceEndpoint?: string;
  sampleRate?: number;
  logLevel: string;
}

let sdk: NodeSDK | undefined;

/**
 * initTelemetry は OpenTelemetry NodeSDK を初期化する。
 * traceEndpoint が指定されている場合、OTLP gRPC エクスポータを設定する。
 */
export function initTelemetry(cfg: TelemetryConfig): void {
  if (cfg.traceEndpoint) {
    const exporter = new OTLPTraceExporter({ url: cfg.traceEndpoint });
    sdk = new NodeSDK({
      traceExporter: exporter,
      serviceName: cfg.serviceName,
    });
    sdk.start();
  }
}

/**
 * shutdown は OpenTelemetry SDK をシャットダウンする。
 */
export function shutdown(): Promise<void> {
  return sdk?.shutdown() ?? Promise.resolve();
}
