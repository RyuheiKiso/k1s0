// k1s0 tier3 Idempotency-Key 形式定義
// docs/03_概要設計/04_tier3設計方針/04_状態管理.md §Idempotency-Key format に準拠する
// format: ULID + tenant_id prefix + RPC method short hash（wall-clock 禁止）
// 11_クライアント状態適合仕様.md §PendingQueueEntry.lineage.idempotencyKey の SoT

// IdempotencyKey の構成要素を型として宣言する
// format: "{ulid}:{tenant_id}:{method_hash_8char}"
// 例: "01HXZ9Y3JQKR6V7B0T3YVHCW5:tenant-001:a1b2c3d4"
export interface IdempotencyKeyComponents {
  // ULID: Universally Unique Lexicographically Sortable Identifier（wall-clock を使用しない）
  // NOTE: wall-clock TTL 禁止（src/CLAUDE.md 参照）—— HLC ベースの ULID を使用する
  readonly ulid: string;
  // tenant_id: テナント識別子プレフィックス
  readonly tenantId: string;
  // method_hash: RPC メソッド名の SHA-256 先頭 8 文字（大文字小文字は lower-case に正規化する）
  readonly methodHash: string;
}

// IDEMPOTENCY_KEY_SEPARATOR: 各構成要素の区切り文字
// format: "{ulid}{SEP}{tenantId}{SEP}{methodHash}"
export const IDEMPOTENCY_KEY_SEPARATOR = ":" as const;

// IDEMPOTENCY_KEY_METHOD_HASH_LENGTH: RPC メソッド名ハッシュの切り取り文字数
export const IDEMPOTENCY_KEY_METHOD_HASH_LENGTH = 8 as const;

// IDEMPOTENCY_KEY_TTL_MS: Idempotency-Key の有効期限（24 時間をミリ秒で表現する）
// 11_クライアント状態適合仕様.md §PendingQueueEntry.lineage.expiresAtMs に対応する
// wall-clock によるデッドライン計算禁止のため、HLC との差分計算で TTL を判定する
export const IDEMPOTENCY_KEY_TTL_MS = 24 * 60 * 60 * 1000 as const;

// OverTtlPolicy: TTL 超過時のポリシー型
// 11_クライアント状態適合仕様.md §over_ttl_policy に準拠する
export type OverTtlPolicy =
  // mark_as_stale: まず古いと印を付ける（ユーザー確認前のデフォルト）
  | "mark_as_stale"
  // require_user_confirmation: ユーザーに明示的な確認を要求する
  | "require_user_confirmation";

// DEFAULT_OVER_TTL_POLICY: TTL 超過時のデフォルトポリシー
// mark_as_stale → require_user_confirmation の順で段階的に escalate する
export const DEFAULT_OVER_TTL_POLICY: OverTtlPolicy = "mark_as_stale" as const;

// buildIdempotencyKey は IdempotencyKeyComponents から Idempotency-Key 文字列を組み立てる
// format: "{ulid}:{tenantId}:{methodHash}"
// NOTE: wall-clock タイムスタンプの埋め込み禁止（src/CLAUDE.md §wall-clock TTL 禁止）
export function buildIdempotencyKey(components: IdempotencyKeyComponents): string {
  // ULID が空でないことを確認する（空の ULID は不正）
  if (components.ulid.length === 0) {
    // 空の ULID は不正なので例外を投げる
    throw new Error("IdempotencyKey: ulid must not be empty");
  }
  // tenant_id が空でないことを確認する
  if (components.tenantId.length === 0) {
    // 空の tenant_id は不正なので例外を投げる
    throw new Error("IdempotencyKey: tenantId must not be empty");
  }
  // method_hash が 8 文字であることを確認する
  if (components.methodHash.length !== IDEMPOTENCY_KEY_METHOD_HASH_LENGTH) {
    // 8 文字でない場合は不正なので例外を投げる
    throw new Error(
      `IdempotencyKey: methodHash must be ${IDEMPOTENCY_KEY_METHOD_HASH_LENGTH} chars, got ${components.methodHash.length}`
    );
  }
  // format: "{ulid}:{tenantId}:{methodHash}" で組み立てる
  return [components.ulid, components.tenantId, components.methodHash].join(
    IDEMPOTENCY_KEY_SEPARATOR
  );
}

// chainIdempotencyKey は既存の Idempotency-Key から chained key を生成する
// parity_vectors.yaml §idempotency_key_chaining の chain_key 操作に対応する
// NOTE: wall-clock タイムスタンプの埋め込み禁止（HLC ベースの ULID を使用する）
export function chainIdempotencyKey(
  originalKey: string,
  newUlid: string
): string {
  // original_key が空でないことを確認する
  if (originalKey.length === 0) {
    // 空の original_key は不正なので例外を投げる
    throw new Error("chainIdempotencyKey: originalKey must not be empty");
  }
  // newUlid が空でないことを確認する
  if (newUlid.length === 0) {
    // 空の newUlid は不正なので例外を投げる
    throw new Error("chainIdempotencyKey: newUlid must not be empty");
  }
  // chained key: original_key を prefix として新しい ULID を suffix として付加する
  // format: "{originalKey}:{newUlid}"
  // starts_with_original_key = true（parity_vectors.yaml §idempotency_key_chaining §expected_output_schema 準拠）
  return `${originalKey}${IDEMPOTENCY_KEY_SEPARATOR}${newUlid}`;
}

// isIdempotencyKeyExpired は Idempotency-Key が TTL を超過しているかを判定する
// NOTE: HLC タイムスタンプとの差分計算で TTL を判定する（wall-clock 禁止）
// expiresAtMs は HLC ベースの ULID から導出した推定 UNIX ミリ秒で表現する
export function isIdempotencyKeyExpired(
  expiresAtMs: number,
  nowMs: number
): boolean {
  // expiresAtMs が 0 以下の場合は即時期限切れとする（不正値のフォールバック）
  if (expiresAtMs <= 0) {
    // 不正値は期限切れとして扱う
    return true;
  }
  // nowMs が expiresAtMs を超えている場合は期限切れとする
  return nowMs > expiresAtMs;
}
