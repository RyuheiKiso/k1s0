/**
 * k1s0 tier2 Outbox relay TypeScript 実装
 * Rust 実装（outbox.rs）と 4 言語等価強度を持つ TypeScript 版
 * atomic_triple_write と同一 txn で書かれた Outbox エントリを Kafka に非同期転送する
 * Debezium CDC 経由の転送（直接 produce は禁止）
 */

// IDEMPOTENCY_KEY_TTL_MS: 冪等性キーの有効期限（ミリ秒）= 24 時間
// 同一の idempotency_key でのダブル書込みを防止するための TTL
export const IDEMPOTENCY_KEY_TTL_MS = 24 * 60 * 60 * 1000;

/**
 * Outbox ペイロード（PII 平文を含まない設計）
 * Kafka に転送するデータを格納する
 */
// OutboxPayload インターフェース定義（PII 平文を構造的に不在にする）
export interface OutboxPayload {
  // イベントの種別識別子（aggregate の型名を表す）
  readonly aggregateType: string;
  // イベントの内容（PII は redact 済みまたは構造的に不在）
  readonly data: unknown;
  // メタデータ（trace_id / version 等）
  readonly metadata: unknown;
}

/**
 * Outbox テーブルのエントリ（Domain Event を Kafka に転送するための中継記録）
 * Rust の OutboxEntry と意味的に等価な TypeScript 版
 */
// OutboxMessage インターフェース定義
export interface OutboxMessage {
  // Outbox エントリの主キー（UUID 文字列）
  readonly id: string;
  // テナント ID（Debezium CDC が Kafka routing に使用する）
  readonly tenantId: string;
  // 関連する aggregate の ID（UUID 文字列）
  readonly aggregateId: string;
  // イベント種別（Debezium が Kafka topic routing に使用する）
  readonly eventType: string;
  // Kafka に転送するペイロード（PII は含まない）
  readonly payload: OutboxPayload;
  // 冪等性キー（同一メッセージの二重投入を防止する）
  readonly idempotencyKey: string;
  // 書込日時（ISO 8601 文字列）
  readonly createdAt: string;
  // Debezium が処理済みにする日時（undefined = 未処理）
  readonly processedAt?: string;
}

/**
 * OutboxStore<T>: Outbox の永続化インターフェース
 * tenant_id は引数で受け取らず、内部的に TenantContext から取得する
 */
// OutboxStore インターフェース定義（ジェネリクスで型安全を保証する）
export interface OutboxStore<T extends OutboxMessage = OutboxMessage> {
  /**
   * Outbox エントリを永続化する（atomic_triple_write と同一 txn で呼ぶ）
   * msg が undefined の場合はエラーをスローする
   */
  // save メソッド（永続化操作）
  save(msg: T): Promise<void>;

  /**
   * aggregate_id に紐づく未処理の Outbox エントリを取得する
   * tenant_id は引数で受け取らず、実装内部で TenantContext から注入する
   */
  // find メソッド（取得操作）
  find(aggregateId: string): Promise<T[]>;

  /**
   * Outbox エントリを配信済みにする（Debezium CDC が呼ぶ）
   * processedAt を現在時刻で更新する
   */
  // markDelivered メソッド（配信済みマーク操作）
  markDelivered(id: string): Promise<void>;
}

/**
 * isExpired: 冪等性キーが TTL を超過しているかを返す
 * TTL は IDEMPOTENCY_KEY_TTL_MS 定数（24 時間）で定義される
 */
// isExpired 関数（TTL チェック）
export function isExpired(msg: OutboxMessage): boolean {
  // createdAt から IDEMPOTENCY_KEY_TTL_MS を加算した時刻が現在時刻より前かを確認する
  return Date.now() > new Date(msg.createdAt).getTime() + IDEMPOTENCY_KEY_TTL_MS;
}
