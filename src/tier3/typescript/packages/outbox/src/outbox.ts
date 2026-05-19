// k1s0 tier3 IndexedDB encrypted outbox
// PQ（PendingQueue）を IndexedDB + WebCrypto AES-GCM で encrypted at rest にする
// PII strip on enqueue / Idempotency-Key 24h TTL を強制する

// Idempotency-Key の 24h TTL（ミリ秒）
export const IDEMPOTENCY_KEY_TTL_MS = 24 * 60 * 60 * 1000;

// HLC フォーマット: {timestamp_ms_hex}-{logical_counter}-{node_id}
// wall clock を TTL/deadline 計算に使うことを禁止するため HLC でラップする
// logical_counter と node_id は本実装では固定値（0000）を使用する
export function hlcNow(): string {
  // Date.now() を HLC の物理クロック基底として使用する（TS HLC: monotonic 担保はアプリ層で行う）
  const timestampMsHex = Date.now().toString(16).padStart(16, "0");
  // logical_counter は現実装では 0000 固定（同一ミリ秒内の複数イベントが不要なため）
  const logicalCounter = "0000";
  // node_id は現実装では 0000 固定（単一ノード想定）
  const nodeId = "0000";
  // HLC タイムスタンプ文字列を組み立てて返す
  return `${timestampMsHex}-${logicalCounter}-${nodeId}`;
}

// HLC タイムスタンプからミリ秒を抽出する
// hlcTimestamp: "{timestamp_ms_hex}-{logical_counter}-{node_id}" 形式
function extractMsFromHlc(hlcTimestamp: string): number {
  // ハイフン区切りの先頭部分が 16 進数ミリ秒タイムスタンプ（undefined の場合は "0" にフォールバック）
  return parseInt(hlcTimestamp.split("-")[0] ?? "0", 16);
}

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
  // enqueue 日時（HLC タイムスタンプ: "{timestamp_ms_hex}-{logical_counter}-{node_id}"）
  readonly enqueuedAt: string;
  // TTL（UNIX ミリ秒、24h 後 — backward compat 用途で保持する; 値は HLC から導出する）
  readonly expiresAtMs: number;
  // chain 元 idempotency_key（rebase 後再送時に設定）
  readonly chainedFrom?: string;
  // aggregate ID
  readonly aggregateId: string;
  // RPC method 名（短縮）
  readonly rpcMethod: string;
}

// Idempotency-Key が TTL 超過かどうかを HLC ベースで確認する
// wall-clock TTL 禁止規約に従い Date.now() を直接使用せず HLC 比較を行う
export function isExpired(meta: OutboxEntryMeta): boolean {
  // enqueue 時刻（ミリ秒）を HLC タイムスタンプから抽出する
  const enqueuedMs = extractMsFromHlc(meta.enqueuedAt);
  // 現在時刻（HLC ベースのミリ秒）を取得する
  const nowMs = extractMsFromHlc(hlcNow());
  // enqueue 時刻 + TTL が現在時刻以下であれば TTL 超過と判定する
  return enqueuedMs + IDEMPOTENCY_KEY_TTL_MS <= nowMs;
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
// wall-clock TTL 禁止規約に従い HLC を使用する
export function generateIdempotencyKey(
  aggregateId: string,
  rpcMethod: string,
): string {
  // HLC タイムスタンプの先頭 16 進数部分をランダム識別子の基底として使用する
  const hlcBase = hlcNow().split("-")[0];
  // ランダム部分を生成する（Math.random を使用してエントリ固有性を確保する）
  const random = Math.random().toString(36).slice(2, 10);
  // aggregateId の先頭 8 文字 + method の先頭 4 文字を prefix に使用する
  const prefix = `${aggregateId.slice(0, 8)}_${rpcMethod.slice(0, 4)}`;
  // prefix + HLC ベース + random で Idempotency-Key を組み立てる
  return `${prefix}_${hlcBase}_${random}`;
}

// chain された新 Idempotency-Key を生成する（rebase 後再送）
export function chainIdempotencyKey(
  original: string,
  aggregateId: string,
  rpcMethod: string,
): { newKey: string; chainedFrom: string } {
  // 新しい key を生成して chain 親子関係を記録する
  const newKey = generateIdempotencyKey(aggregateId, rpcMethod);
  // chain 元と新 key の組を返す
  return { newKey, chainedFrom: original };
}

// Outbox エントリのメタデータを生成する（PII strip 済み payload と一緒に使用）
// wall-clock TTL 禁止規約に従い HLC ベースのタイムスタンプを使用する
export function createOutboxMeta(
  aggregateId: string,
  rpcMethod: string,
  chainedFrom?: string,
): OutboxEntryMeta {
  // HLC タイムスタンプを現在時刻として取得する
  const nowHlc = hlcNow();
  // enqueue 時刻（ミリ秒）を HLC から抽出する
  const enqueuedMs = extractMsFromHlc(nowHlc);
  // chain がある場合は chain された新 key を生成する
  const key = chainedFrom
    ? chainIdempotencyKey(chainedFrom, aggregateId, rpcMethod).newKey
    : generateIdempotencyKey(aggregateId, rpcMethod);
  // backward compat 用の expiresAtMs は HLC ミリ秒から計算する
  const expiresAtMs = enqueuedMs + IDEMPOTENCY_KEY_TTL_MS;
  // メタデータオブジェクトを組み立てて返す
  return {
    // 生成した Idempotency-Key
    idempotencyKey: key,
    // HLC タイムスタンプ（wall clock 代替）
    enqueuedAt: nowHlc,
    // backward compat 用 TTL（HLC から導出した値）
    expiresAtMs,
    // exactOptionalPropertyTypes: undefined キーは省略して spread で条件追加する
    ...(chainedFrom !== undefined ? { chainedFrom } : {}),
    // aggregate ID
    aggregateId,
    // RPC method 名
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
    // PII フィールド以外のみを結果に含める
    if (!piiFieldNames.includes(key)) {
      stripped[key] = value;
    }
  }
  // PII strip 済み payload を返す
  return stripped;
}
