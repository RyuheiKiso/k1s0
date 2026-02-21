import { CorrelationId, TraceId, CorrelationContext, newCorrelationContext } from './types.js';

export const HEADER_CORRELATION_ID = 'X-Correlation-Id';
export const HEADER_TRACE_ID = 'X-Trace-Id';

/**
 * toHeaders は CorrelationContext を HTTP ヘッダーマップに変換する。
 */
export function toHeaders(ctx: CorrelationContext): Record<string, string> {
  const headers: Record<string, string> = {};
  if (!ctx.correlationId.isEmpty()) {
    headers[HEADER_CORRELATION_ID] = ctx.correlationId.toString();
  }
  if (!ctx.traceId.isEmpty()) {
    headers[HEADER_TRACE_ID] = ctx.traceId.toString();
  }
  return headers;
}

/**
 * fromHeaders は HTTP ヘッダーマップから CorrelationContext を生成する。
 * ヘッダーが存在しない場合は自動生成する。
 */
export function fromHeaders(headers: Record<string, string>): CorrelationContext {
  let correlationId: CorrelationId;
  let traceId: TraceId;

  const corrHeader = headers[HEADER_CORRELATION_ID];
  if (corrHeader && corrHeader !== '') {
    correlationId = CorrelationId.parse(corrHeader);
  } else {
    correlationId = CorrelationId.generate();
  }

  const traceHeader = headers[HEADER_TRACE_ID];
  if (traceHeader && traceHeader !== '') {
    try {
      traceId = TraceId.parse(traceHeader);
    } catch {
      traceId = TraceId.generate();
    }
  } else {
    traceId = TraceId.generate();
  }

  return { correlationId, traceId };
}
