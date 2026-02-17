import type { Metrics } from './metrics';
export type GrpcInvoker<TReq, TRes> = (req: TReq) => Promise<TRes>;
export type GrpcInterceptorFn = <TReq, TRes>(method: string, request: TReq, invoker: GrpcInvoker<TReq, TRes>) => Promise<TRes>;
/**
 * createGrpcInterceptor は gRPC unary interceptor を生成する。
 * - OpenTelemetry span を生成
 * - リクエスト duration を計測
 * - エラーステータスを記録
 * - Metrics にリクエストカウンタと duration を記録
 *
 * Go の GRPCUnaryInterceptor と同等の機能を持つ。
 */
export declare function createGrpcInterceptor(metrics: Metrics): GrpcInterceptorFn;
