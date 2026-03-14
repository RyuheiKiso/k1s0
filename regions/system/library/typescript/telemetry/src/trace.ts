import { context, trace, SpanStatusCode, Span, SpanOptions } from '@opentelemetry/api';

/**
 * withTrace は非同期関数をトレーススパンでラップして実行する汎用ヘルパー。
 * fn がエラーをスローした場合、スパンにエラーを記録する。
 */
export async function withTrace<T>(
  tracerName: string,
  spanName: string,
  fn: (span: Span) => Promise<T>,
  options?: SpanOptions,
): Promise<T> {
  const tracer = trace.getTracer(tracerName);
  return tracer.startActiveSpan(spanName, options ?? {}, async (span) => {
    try {
      const result = await fn(span);
      span.setStatus({ code: SpanStatusCode.OK });
      return result;
    } catch (err) {
      // エラーをスパンに記録してから再スローする
      span.recordException(err as Error);
      span.setStatus({ code: SpanStatusCode.ERROR, message: String(err) });
      throw err;
    } finally {
      span.end();
    }
  });
}

/**
 * Trace デコレータはクラスメソッドをトレーススパンでラップする。
 * @param tracerName - スパンの計装名（通常はサービス名）
 * @param spanName - スパン名（省略時はメソッド名）
 */
export function Trace(tracerName: string, spanName?: string): MethodDecorator {
  return function (target: object, propertyKey: string | symbol, descriptor: PropertyDescriptor) {
    const originalMethod = descriptor.value as (...args: unknown[]) => unknown;
    const name = spanName ?? String(propertyKey);

    descriptor.value = async function (...args: unknown[]) {
      const tracer = trace.getTracer(tracerName);
      return tracer.startActiveSpan(name, async (span: Span) => {
        try {
          const result = await originalMethod.apply(this, args);
          span.setStatus({ code: SpanStatusCode.OK });
          return result;
        } catch (err) {
          span.recordException(err as Error);
          span.setStatus({ code: SpanStatusCode.ERROR, message: String(err) });
          throw err;
        } finally {
          span.end();
        }
      });
    };

    return descriptor;
  };
}

/**
 * getCurrentSpan は現在アクティブなスパンを返す。
 * スパンが存在しない場合は undefined を返す。
 */
export function getCurrentSpan(): Span | undefined {
  const span = trace.getActiveSpan();
  return span ?? undefined;
}

/**
 * addSpanAttribute は現在のスパンに属性を追加する。
 * スパンが存在しない場合は何もしない。
 */
export function addSpanAttribute(key: string, value: string | number | boolean): void {
  const span = trace.getActiveSpan();
  if (span) {
    span.setAttribute(key, value);
  }
}
