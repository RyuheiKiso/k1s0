/**
 * Metrics は Prometheus 互換メトリクスのヘルパークラスである。
 * RED メソッド（Rate, Errors, Duration）のメトリクスを提供する。
 * Go 実装の metrics.go と同等の機能を持つ。
 */
export declare class Metrics {
    readonly serviceName: string;
    private httpRequestsTotal;
    private httpRequestDuration;
    private grpcHandledTotal;
    private grpcHandlingDuration;
    constructor(serviceName: string);
    recordHTTPRequest(method: string, path: string, status: number): void;
    recordHTTPDuration(method: string, path: string, durationSeconds: number): void;
    recordGRPCRequest(grpcService: string, grpcMethod: string, grpcCode: string): void;
    recordGRPCDuration(grpcService: string, grpcMethod: string, durationSeconds: number): void;
    getMetrics(): string;
    private serializeCounter;
    private serializeHistogram;
}
