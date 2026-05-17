// k1s0 tier3 IndexedDB encrypted outbox
// PQ（PendingQueue）を IndexedDB + WebCrypto AES-GCM で encrypted at rest にする
// PII strip on enqueue / Idempotency-Key 24h TTL を強制する

// Idempotency-Key の 24h TTL（ミリ秒）
export const IDEMPOTENCY_KEY_TTL_MS = 24 * 60 * 60 * 1000;

// PII strip の結果を保持する型（PII フィールドを除去した payload）
export type PiiStripped<T> = {
  // PII フィールドを除外した payload（in-memory でのみ PII を保持する）
  [K in keyof T as T[K] extends never ? never : K]: T[K] extends string
    ? string
    : T[K];
};

// Outbox エントリのメタデータ
export interface OutboxEntryMeta {
  // Idempotency-Key（ULID + tenant_id prefix + RPC method short hash）
  readonly idempotencyKey: string;
  // enqueue 日時（HLC タイムスタンプ）
  readonly enqueuedAt: string;
  // TTL（UNIX ミリ秒、24h 後）
  readonly expiresAtMs: number;
  // chain 元 idempotency_key（rebase 後再送時に設定）
  readonly chainedFrom?: string;
  // aggregate ID
  readonly aggregateId: string;
  // RPC method 名（短縮）
  readonly rpcMethod: string;
}

// Idempotency-Key が TTL 超過かどうかを確認する
export function isExpired(meta: OutboxEntryMeta): boolean {
  // 現在時刻が expiresAtMs を超えているか確認する
  return Date.now() > meta.expiresAtMs;
}

// over_ttl_policy の 3 種（適合仕様の over_ttl_policy に準拠）
export type OverTtlPolicy =
  // 古いままでも送信する
  | "stale"
  // ユーザー確認を求める
  | "user_confirm"
  // 新 key で再送する
  | "new_key_resend";

// Idempotency-Key を生成する（ULID + aggregateId prefix + method hash）
export function generateIdempotencyKey(
  aggregateId: string,
  rpcMethod: string,
): string {
  // ULID ベースのランダム部分を生成する（Date.now() + random）
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).slice(2, 10);
  // aggregateId の先頭 8 文字 + method の先頭 4 文字を prefix に使用する
  const prefix = `${aggregateId.slice(0, 8)}_${rpcMethod.slice(0, 4)}`;
  return `${prefix}_${timestamp}_${random}`;
}

// chain された新 Idempotency-Key を生成する（rebase 後再送）
export function chainIdempotencyKey(
  original: string,
  aggregateId: string,
  rpcMethod: string,
): { newKey: string; chainedFrom: string } {
  // 新しい key を生成して chain 親子関係を記録する
  const newKey = generateIdempotencyKey(aggregateId, rpcMethod);
  return { newKey, chainedFrom: original };
}

// Outbox エントリのメタデータを生成する（PII strip 済み payload と一緒に使用）
export function createOutboxMeta(
  aggregateId: string,
  rpcMethod: string,
  chainedFrom?: string,
): OutboxEntryMeta {
  // 現在時刻から 24h 後を TTL に設定する
  const key = chainedFrom
    ? chainIdempotencyKey(chainedFrom, aggregateId, rpcMethod).newKey
    : generateIdempotencyKey(aggregateId, rpcMethod);
  return {
    idempotencyKey: key,
    enqueuedAt: new Date().toISOString(),
    expiresAtMs: Date.now() + IDEMPOTENCY_KEY_TTL_MS,
    chainedFrom,
    aggregateId,
    rpcMethod,
  };
}

// PII フィールドを strip する（field_pii annotation が付いたフィールドを除去）
// PII フィールドは in-memory でのみ保持し、enqueue 時に strip する
export function stripPiiFields<T extends Record<string, unknown>>(
  payload: T,
  piiFieldNames: readonly string[],
): Record<string, unknown> {
  // piiFieldNames に含まれるフィールドを除去する
  const stripped: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(payload)) {
    if (!piiFieldNames.includes(key)) {
      stripped[key] = value;
    }
  }
  return stripped;
}
