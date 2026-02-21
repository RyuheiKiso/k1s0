/**
 * CorrelationId は分散トレーシングのリクエスト相関 ID。
 * UUID v4 文字列ラッパー（バリデーションなし）。
 */
export class CorrelationId {
  private readonly value: string;

  private constructor(value: string) {
    this.value = value;
  }

  static generate(): CorrelationId {
    return new CorrelationId(crypto.randomUUID());
  }

  static parse(s: string): CorrelationId {
    return new CorrelationId(s);
  }

  toString(): string {
    return this.value;
  }

  isEmpty(): boolean {
    return this.value === '';
  }
}

/**
 * TraceId は OpenTelemetry 互換の 32 文字小文字 hex トレース ID。
 */
export class TraceId {
  private readonly value: string;

  private constructor(value: string) {
    this.value = value;
  }

  static generate(): TraceId {
    const uuid = crypto.randomUUID().replace(/-/g, '');
    return new TraceId(uuid);
  }

  static parse(s: string): TraceId {
    if (s.length !== 32) {
      throw new Error(`Invalid trace id length: expected 32, got ${s.length}`);
    }
    if (!/^[0-9a-f]{32}$/.test(s)) {
      throw new Error(`Invalid trace id: must be 32 lowercase hex characters`);
    }
    return new TraceId(s);
  }

  toString(): string {
    return this.value;
  }

  isEmpty(): boolean {
    return this.value === '';
  }
}

/**
 * CorrelationContext は CorrelationId と TraceId を保持するコンテキスト。
 */
export interface CorrelationContext {
  readonly correlationId: CorrelationId;
  readonly traceId: TraceId;
}

export function newCorrelationContext(): CorrelationContext {
  return {
    correlationId: CorrelationId.generate(),
    traceId: TraceId.generate(),
  };
}
