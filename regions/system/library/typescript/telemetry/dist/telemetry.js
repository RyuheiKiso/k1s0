import { NodeSDK } from '@opentelemetry/sdk-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';
let sdk;
/**
 * initTelemetry は OpenTelemetry NodeSDK を初期化する。
 * traceEndpoint が指定されている場合、OTLP gRPC エクスポータを設定する。
 */
export function initTelemetry(cfg) {
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
export function shutdown() {
    return sdk?.shutdown() ?? Promise.resolve();
}
