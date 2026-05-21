// ttl_policy.ts — Idempotency-Key の over_ttl_policy resolver
// spec 11 §04_状態管理 §Idempotency-Key: TTL 超過時の 3 分岐ポリシーを実装する
// wall-clock TTL 禁止規約（src/CLAUDE.md §wall-clock TTL 禁止）に従い、
// TTL 比較は @k1s0/hlc-lib の HLC elapsed を使用する（Date.now() 直接参照禁止）

// @k1s0/hlc-lib: HLC タイムスタンプのパース・比較 API を提供する（wall-clock 禁止規律準拠）
import { HlcTimestamp, HlcClock } from '@k1s0/hlc-lib';

// モジュールレベルのグローバル HLC クロック（環境変数 HLC_NODE_ID から node_id を取得する）
const _hlcClock = HlcClock.fromEnv();

// TTL 超過時のポリシー種別
export type OverTtlPolicy = 'stale' | 'user_confirm' | 'new_key_resend';

// TTL 超過時の解決結果
export interface TtlPolicyResult {
  // 選択されたポリシー
  policy: OverTtlPolicy;
  // 新しい idempotency key（new_key_resend の場合のみ）
  newKey?: string;
  // ユーザーへのメッセージ（user_confirm の場合のみ）
  confirmMessage?: string;
}

// idempotency key の TTL（24 時間をミリ秒で表現する）
export const IDEMPOTENCY_KEY_TTL_MS = 24 * 60 * 60 * 1000;

// idempotency key の有効期限を確認する関数
// keyCreatedAtHlc: key 生成時の HLC タイムスタンプ文字列（"{wall_ms_hex_16}-{logical_04x}-{node_04x}" 形式）
// wall-clock TTL 禁止規約に従い Date.now() を直接使用せず HLC elapsed で比較する
export function isIdempotencyKeyExpired(keyCreatedAtHlc: string): boolean {
  // 引数の HLC 文字列を HlcTimestamp にパースする（パース失敗時は期限切れとして扱う）
  const createdAt = HlcTimestamp.parseCompact(keyCreatedAtHlc);
  // パース失敗（null）の場合は安全側（expired = true）を返す
  if (createdAt === null) {
    return true;
  }
  // 現在の HLC タイムスタンプを取得する（Date.now() を直接使わず HLC 経由で取得する）
  const now = _hlcClock.now();
  // key 生成時刻から TTL 分加算した deadline を計算する
  const deadline = createdAt.addMs(BigInt(IDEMPOTENCY_KEY_TTL_MS));
  // 現在の HLC が deadline を超えていれば期限切れと判定する
  return deadline.isExpiredAt(now);
}

// TTL 超過ポリシーを解決する関数
// key: 超過した idempotency key
// context: 判断に使うコンテキスト（操作の重要度など）
export function resolveOverTtlPolicy(
  key: string,
  context: { critical: boolean; allowAutoResend: boolean }
): TtlPolicyResult {
  // 重要な操作の場合はユーザー確認を要求する
  if (context.critical) {
    return {
      policy: 'user_confirm',
      confirmMessage: `操作 ${key} の有効期限が切れています。再送しますか？`,
    };
  }
  // 自動再送が許可されている場合は新しい key で再送する
  if (context.allowAutoResend) {
    // UUIDv4 ベースの新しい key を生成する（wall-clock 依存を避ける）
    const newKey = `${key}_resend_${crypto.randomUUID()}`;
    return {
      policy: 'new_key_resend',
      newKey,
    };
  }
  // それ以外は stale として破棄する
  return { policy: 'stale' };
}
