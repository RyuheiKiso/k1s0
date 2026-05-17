// k1s0 tier3 outbox unit test
// PII strip と Idempotency-Key TTL 検査（整合 6 / 整合 7 の物理証跡）

import { describe, it, expect } from "vitest";
import {
  stripPiiFields,
  isExpired,
  createOutboxMeta,
  generateIdempotencyKey,
  chainIdempotencyKey,
  IDEMPOTENCY_KEY_TTL_MS,
} from "../src/outbox.js";

// PII strip のテスト（整合 6: PII を含むまま PQ に enqueue する経路ゼロ）
describe("stripPiiFields（PII strip on enqueue）", () => {
  it("PII フィールドを strip した payload を返す", () => {
    // PII フィールドを含む payload を用意する
    const payload = {
      orderId: "order-001",
      quantity: 100,
      // PII フィールド: customerName / email は strip 対象
      customerName: "山田太郎",
      email: "yamada@example.com",
    };
    // PII フィールドを strip する
    const stripped = stripPiiFields(payload, ["customerName", "email"]);
    // PII フィールドが除去されることを確認する
    expect(stripped).not.toHaveProperty("customerName");
    expect(stripped).not.toHaveProperty("email");
    // 非 PII フィールドは保持されることを確認する
    expect(stripped).toHaveProperty("orderId", "order-001");
    expect(stripped).toHaveProperty("quantity", 100);
  });

  it("PII フィールドが無い場合は payload をそのまま返す", () => {
    // PII フィールドが無い payload を用意する
    const payload = { orderId: "order-002", quantity: 50 };
    const stripped = stripPiiFields(payload, ["customerName"]);
    // 全フィールドが保持されることを確認する
    expect(stripped).toEqual(payload);
  });
});

// Idempotency-Key TTL のテスト（整合 7: Idempotency-Key 24h TTL の SDK runtime enforcement）
describe("isExpired（Idempotency-Key 24h TTL）", () => {
  it("expiresAtMs が現在より未来のとき expired でない", () => {
    // TTL 内のメタデータを生成する
    const meta = createOutboxMeta("agg-001", "UpdateOrder");
    // TTL 内のため expired でないことを確認する
    expect(isExpired(meta)).toBe(false);
  });

  it("expiresAtMs が現在より過去のとき expired である", () => {
    // TTL 超過のメタデータを生成する（過去の expiresAtMs を設定する）
    const meta = {
      ...createOutboxMeta("agg-002", "UpdateOrder"),
      expiresAtMs: Date.now() - 1,
    };
    // TTL 超過のため expired であることを確認する
    expect(isExpired(meta)).toBe(true);
  });
});

// chained idempotency_key のテスト（stale_write rebase 後再送）
describe("chainIdempotencyKey（rebase 後再送）", () => {
  it("chain 後の新 key が生成され chain 元が記録される", () => {
    // 元の key を生成する
    const original = generateIdempotencyKey("agg-003", "UpdateOrder");
    // chain 後の新 key を生成する
    const { newKey, chainedFrom } = chainIdempotencyKey(original, "agg-003", "UpdateOrder");
    // 新 key が元の key と異なることを確認する
    expect(newKey).not.toBe(original);
    // chain 元が正しく記録されることを確認する
    expect(chainedFrom).toBe(original);
  });
});
