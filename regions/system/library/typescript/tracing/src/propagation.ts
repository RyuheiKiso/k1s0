import type { TraceContext } from './types.js';
import { toTraceparent, fromTraceparent } from './types.js';
import { Baggage } from './baggage.js';

export function injectContext(
  headers: Record<string, string>,
  ctx: TraceContext,
  baggage?: Baggage,
): void {
  headers['traceparent'] = toTraceparent(ctx);
  if (baggage) {
    const h = baggage.toHeader();
    if (h) {
      headers['baggage'] = h;
    }
  }
}

export function extractContext(headers: Record<string, string>): {
  context: TraceContext | null;
  baggage: Baggage;
} {
  const context = headers['traceparent'] ? fromTraceparent(headers['traceparent']) : null;
  const baggage = headers['baggage'] ? Baggage.fromHeader(headers['baggage']) : new Baggage();
  return { context, baggage };
}
