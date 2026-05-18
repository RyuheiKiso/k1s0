// ttl_policy.ts — Idempotency-Key の over_ttl_policy resolver
// spec 11 §04_状態管理 §Idempotency-Key: TTL 超過時の 3 分岐ポリシーを実装する

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
// keyCreatedAtMs: key 生成時の UNIX ミリ秒タイムスタンプ
export function isIdempotencyKeyExpired(keyCreatedAtMs: number): boolean {
  // 現在時刻との差が TTL を超えているか確認する
  return Date.now() - keyCreatedAtMs > IDEMPOTENCY_KEY_TTL_MS;
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
