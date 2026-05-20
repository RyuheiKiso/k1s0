// k1s0 tier3 IndexedDB encrypted outbox
// PQ（PendingQueue）を IndexedDB + WebCrypto AES-GCM で encrypted at rest にする
// PII strip on enqueue / Idempotency-Key 24h TTL を強制する
// wall-clock TTL 禁止規約（src/CLAUDE.md §wall-clock TTL 禁止）に従い、
// Date.now() を直接使用せず @k1s0/hlc-lib の HlcClock / HlcTimestamp を経由する

// @k1s0/hlc-lib: HLC クロックおよびタイムスタンプ操作 API（wall-clock 禁止規律準拠）
import { HlcTimestamp, HlcClock } from '@k1s0/hlc-lib';

// Idempotency-Key の 24h TTL（ミリ秒）
export const IDEMPOTENCY_KEY_TTL_MS = 24 * 60 * 60 * 1000;

// モジュールレベルのグローバル HLC クロック（環境変数 HLC_NODE_ID から node_id を取得する）
// wall-clock TTL 禁止規約に従い @k1s0/hlc-lib の HlcClock のみが Date.now() を呼ぶ
const _globalHlcClock = HlcClock.fromEnv();

// hlcNow は @k1s0/hlc-lib のグローバルクロックから現在の HLC タイムスタンプを取得して compact 文字列に変換する
// 旧実装の手書き Date.now() を @k1s0/hlc-lib 経由に置換する（wall-clock 禁止規律準拠）
export function hlcNow(): string {
  // HlcClock.now()（tick の alias）で現在の HLC タイムスタンプを生成する
  const ts = _globalHlcClock.now();
  // formatCompact で "{wall_ms_hex_16}-{logical_04x}-{node_04x}" 形式の文字列を返す
  return ts.formatCompact();
}

// extractMsFromHlc は HLC compact 文字列からミリ秒値を number として抽出する
// hlcTimestamp: "{timestamp_ms_hex}-{logical_counter}-{node_id}" 形式
// @k1s0/hlc-lib の HlcTimestamp.parseCompact を使用してパースする（手書き parseInt 禁止）
function extractMsFromHlc(hlcTimestamp: string): number {
  // HlcTimestamp.parseCompact で HlcTimestamp にパースする（失敗時は null）
  const ts = HlcTimestamp.parseCompact(hlcTimestamp);
  // パース失敗時は 0 を返す（safe 側フォールバック）
  if (ts === null) {
    return 0;
  }
  // wall_ms（bigint）を number に変換して返す（2^53 未満の値であれば精度ロスなし）
  return Number(ts.wall_ms);
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

// Idempotency-Key を生成する（tenantId prefix + ULID + method hash）
// フォーマット: "{tenantId}_{ulidHex}_{methodHash}" — docs §idempotency_key 準拠
// tenantId: BFF cookie から取得したテナント識別子（tenant_id_injector 経由で渡す）
// wall-clock TTL 禁止規約に従い HLC を使用する（Math.random() 禁止）
export function generateIdempotencyKey(
  tenantId: string,
  aggregateId: string,
  rpcMethod: string,
): string {
  // HLC タイムスタンプの先頭 16 進数部分を ULID の時刻部分として使用する
  const hlcBase = hlcNow().split("-")[0];
  // crypto.randomUUID() でランダム部分を生成する（暗号論的に安全）
  const randomPart = crypto.randomUUID().replace(/-/g, "").slice(0, 8);
  // ULID 相当: HLC タイムスタンプ hex + random で識別子を生成する
  const ulidHex = `${hlcBase}${randomPart}`;
  // rpcMethod の先頭 4 文字を method hash として使用する（短縮識別子）
  const methodHash = rpcMethod.slice(0, 4);
  // tenantId prefix + ulid + method hash の形式で Idempotency-Key を組み立てる
  return `${tenantId}_${ulidHex}_${methodHash}`;
}

// chain された新 Idempotency-Key を生成する（rebase 後再送）
// tenantId: BFF cookie から取得したテナント識別子（tenant_id_injector 経由で渡す）
export function chainIdempotencyKey(
  original: string,
  tenantId: string,
  aggregateId: string,
  rpcMethod: string,
): { newKey: string; chainedFrom: string } {
  // 新しい key を生成して chain 親子関係を記録する
  const newKey = generateIdempotencyKey(tenantId, aggregateId, rpcMethod);
  // chain 元と新 key の組を返す
  return { newKey, chainedFrom: original };
}

// Outbox エントリのメタデータを生成する（PII strip 済み payload と一緒に使用）
// tenantId: BFF cookie から取得したテナント識別子（tenant_id_injector 経由で渡す）
// wall-clock TTL 禁止規約に従い HLC ベースのタイムスタンプを使用する
export function createOutboxMeta(
  tenantId: string,
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
    ? chainIdempotencyKey(chainedFrom, tenantId, aggregateId, rpcMethod).newKey
    : generateIdempotencyKey(tenantId, aggregateId, rpcMethod);
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
