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
/**
 * initTelemetry は OpenTelemetry NodeSDK を初期化する。
 * traceEndpoint が指定されている場合、OTLP gRPC エクスポータを設定する。
 */
export declare function initTelemetry(cfg: TelemetryConfig): void;
/**
 * shutdown は OpenTelemetry SDK をシャットダウンする。
 */
export declare function shutdown(): Promise<void>;
