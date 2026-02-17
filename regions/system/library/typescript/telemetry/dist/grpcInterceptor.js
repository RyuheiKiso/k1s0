import { trace, SpanStatusCode } from '@opentelemetry/api';
/**
 * gRPC メソッドパスからサービス名とメソッド名を抽出する。
 * パス形式: /package.ServiceName/MethodName
 */
function parseMethodPath(fullMethod) {
    const match = fullMethod.match(/^\/?(.+)\/([^/]+)$/);
    if (match) {
        return { service: match[1], method: match[2] };
    }
    return { service: 'unknown', method: fullMethod };
}
/**
 * createGrpcInterceptor は gRPC unary interceptor を生成する。
 * - OpenTelemetry span を生成
 * - リクエスト duration を計測
 * - エラーステータスを記録
 * - Metrics にリクエストカウンタと duration を記録
 *
 * Go の GRPCUnaryInterceptor と同等の機能を持つ。
 */
export function createGrpcInterceptor(metrics) {
    return async (method, request, invoker) => {
        const tracer = trace.getTracer('k1s0-grpc');
        const { service, method: methodName } = parseMethodPath(method);
        const span = tracer.startSpan(method, {
            attributes: {
                'rpc.system': 'grpc',
                'rpc.service': service,
                'rpc.method': methodName,
            },
        });
        const start = performance.now();
        try {
            const result = await invoker(request);
            const durationSeconds = (performance.now() - start) / 1000;
            span.setStatus({ code: SpanStatusCode.OK });
            metrics.recordGRPCRequest(service, methodName, 'OK');
            metrics.recordGRPCDuration(service, methodName, durationSeconds);
            return result;
        }
        catch (error) {
            const durationSeconds = (performance.now() - start) / 1000;
            const message = error instanceof Error ? error.message : String(error);
            span.setStatus({ code: SpanStatusCode.ERROR, message });
            metrics.recordGRPCRequest(service, methodName, 'ERROR');
            metrics.recordGRPCDuration(service, methodName, durationSeconds);
            throw error;
        }
        finally {
            span.end();
        }
    };
}
