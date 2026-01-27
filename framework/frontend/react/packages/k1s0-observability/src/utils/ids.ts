/**
 * トレースID生成（16バイト/32文字の16進数）
 * W3C Trace Context 形式に準拠
 */
export function generateTraceId(): string {
  const array = new Uint8Array(16);
  crypto.getRandomValues(array);
  return Array.from(array)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * スパンID生成（8バイト/16文字の16進数）
 * W3C Trace Context 形式に準拠
 */
export function generateSpanId(): string {
  const array = new Uint8Array(8);
  crypto.getRandomValues(array);
  return Array.from(array)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * リクエストID生成（UUID v4）
 */
export function generateRequestId(): string {
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  // フォールバック
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

/**
 * ISO 8601 形式のタイムスタンプを生成
 */
export function generateTimestamp(): string {
  return new Date().toISOString();
}

/**
 * W3C Trace Context traceparent ヘッダー値を生成
 *
 * 形式: version-trace_id-parent_id-flags
 * - version: 00 (固定)
 * - flags: 01 = sampled
 */
export function generateTraceparent(traceId: string, spanId: string, sampled: boolean = true): string {
  const flags = sampled ? '01' : '00';
  return `00-${traceId}-${spanId}-${flags}`;
}

/**
 * traceparent ヘッダーをパース
 */
export function parseTraceparent(
  traceparent: string
): { traceId: string; spanId: string; sampled: boolean } | null {
  const match = traceparent.match(
    /^([0-9a-f]{2})-([0-9a-f]{32})-([0-9a-f]{16})-([0-9a-f]{2})$/
  );
  if (!match) return null;

  const [, version, traceId, spanId, flags] = match;

  // version 00 のみサポート
  if (version !== '00') return null;

  return {
    traceId,
    spanId,
    sampled: (parseInt(flags, 16) & 0x01) === 0x01,
  };
}
