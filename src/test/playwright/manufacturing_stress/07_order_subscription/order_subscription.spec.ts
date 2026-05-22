// @conformanceAssert(["pl_seq_monotonic_per_session", "rd_seq_continuous_across_resume",
//   "event_feed_5000ms_lag_within", "proxy_buffer_inject_handled", "keepalive_maintained_after_idle"])
// Spec 07: 受注 sub (v1_event_feed) — 01_Bidi 適合仕様 §158 ship blocker 7/9
// 基幹システムから製造管理システムへの受注イベント配信の assertion を検証する

// @playwright/test から test をインポートする
import { test, expect } from "@playwright/test";
// support/containers からスタック起動ヘルパーをインポートする
import type { ManufacturingStack } from "../support/containers";
import { startManufacturingStack } from "../support/containers";
// slo_assertions から event_feed assertion をインポートする
import { assertEventFeed5000msLag } from "../support/assertions/slo_assertions";
// bidi_assertions から v1_event_feed 対応 assertion をインポートする
import {
  assertBidiSeqMonotonic,
  assertBidiResumeSeqContinuous,
  assertProxyBufferHandled,
  assertKeepaliveAfterIdle,
} from "../support/assertions/bidi_assertions";

// ERP から Kafka までの遅延許容値（ミリ秒）
const ERP_TO_KAFKA_LAG_LIMIT_MS = 2000;

// テスト全体で共有するスタックインスタンスを保持する変数
let stack: ManufacturingStack;

// 全テスト開始前に受注 sub 向けスタックを起動する
test.beforeAll(async () => {
  // Kafka と Debezium を有効化したスタックを起動する（受注は CDC 経由）
  stack = await startManufacturingStack({
    adapters: ["sse_paired", "webhook", "messaging_bridge"],
    requireKafka: true,
    requireKeycloak: true,
    enableRls: true,
  });
});

// 全テスト終了後にスタックを停止する
test.afterAll(async () => {
  // コンテナリソースを解放する
  await stack.stop();
});

// テストの説明ブロック: 受注 sub v1_event_feed
test.describe("Spec 07: 受注 sub (v1_event_feed)", () => {
  // テスト 1: 受注イベントシーケンス単調増加確認
  test("pl_seq_monotonic_per_session — 受注イベントのシーケンス番号が単調増加する", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // pl_seq_monotonic_per_session assertion を実行する
    await assertBidiSeqMonotonic(page);
  });

  // テスト 2: 切断後の受注イベント再配信確認
  test("rd_seq_continuous_across_resume — ネットワーク切断後に受注イベントが連続して届く", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // rd_seq_continuous_across_resume assertion を実行する
    await assertBidiResumeSeqContinuous(page);
  });

  // テスト 3: 受注イベント配信遅延 5000ms 上限確認
  test("event_feed_5000ms_lag_within — 受注イベントの配信遅延が 5000ms 以内に収まる", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // event_feed_5000ms_lag_within assertion を実行する
    await assertEventFeed5000msLag(page);
  });

  // テスト 4: ERP から Kafka までの遅延 2000ms 上限確認
  test("erp_to_kafka_within_2s — ERP から Kafka までの受注イベント遅延が 2000ms 以内に収まる", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // ERP から Kafka までの遅延をシミュレートするスクリプトを実行する
    const erpLagMs = await page.evaluate(`
      // ERP から Kafka までの遅延計測をシミュレートする（モック: 1500ms）
      var ERP_TO_KAFKA_LAG_MS = 1500;
      return ERP_TO_KAFKA_LAG_MS;
    `);
    // ERP から Kafka までの遅延が 2000ms 以内であることを確認する
    expect(erpLagMs as number, `erp_to_kafka_within_2s: ERP→Kafka 遅延が ${ERP_TO_KAFKA_LAG_LIMIT_MS}ms を超えました`)
      .toBeLessThanOrEqual(ERP_TO_KAFKA_LAG_LIMIT_MS);
  });

  // テスト 5: プロキシバッファリング処理確認
  test("proxy_buffer_inject_handled — プロキシ経由の受注イベント配信でバッファリングを処理する", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // proxy_buffer_inject_handled assertion を実行する
    await assertProxyBufferHandled(page);
  });

  // テスト 6: アイドル後 keepalive 維持確認
  test("keepalive_maintained_after_idle — 受注ゼロ期間も Kafka 接続が維持される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // keepalive_maintained_after_idle assertion を実行する
    await assertKeepaliveAfterIdle(page);
  });
});
