// @conformanceAssert(["pl_seq_monotonic_per_session", "rd_seq_continuous_across_resume",
//   "event_feed_5000ms_lag_within", "proxy_buffer_inject_handled", "keepalive_maintained_after_idle"])
// Spec 03: 品質検査結果配信 (v1_event_feed) — 01_Bidi 適合仕様 §158 ship blocker 3/9
// 製造ラインの品質検査結果をリアルタイム配信する event_feed クラスの assertion を検証する

// @playwright/test から test をインポートする
import { test } from "@playwright/test";
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

// テスト全体で共有するスタックインスタンスを保持する変数
let stack: ManufacturingStack;

// 全テスト開始前に event_feed 向けスタックを起動する
test.beforeAll(async () => {
  // Kafka (event feed) を有効化したスタックを起動する
  stack = await startManufacturingStack({
    adapters: ["sse_paired", "webhook"],
    requireKafka: true,
    enableRls: true,
  });
});

// 全テスト終了後にスタックを停止する
test.afterAll(async () => {
  // コンテナリソースを解放する
  await stack.stop();
});

// テストの説明ブロック: 品質検査結果配信 v1_event_feed
test.describe("Spec 03: 品質検査結果配信 (v1_event_feed)", () => {
  // テスト 1: シーケンス単調増加確認 (event_feed クラス)
  test("pl_seq_monotonic_per_session — 品質検査イベントのシーケンスが単調増加する", async ({ page }) => {
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

  // テスト 2: 切断後シーケンス連続性確認
  test("rd_seq_continuous_across_resume — ネットワーク切断後に品質検査イベントが連続して届く", async ({ page }) => {
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

  // テスト 3: 5000ms 遅延上限確認
  test("event_feed_5000ms_lag_within — 品質検査結果の配信遅延が 5000ms 以内に収まる", async ({ page }) => {
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

  // テスト 4: プロキシバッファリング処理確認
  test("proxy_buffer_inject_handled — プロキシ経由の検査結果配信でバッファリング遅延を処理する", async ({ page }) => {
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

  // テスト 5: アイドル後 keepalive 維持確認
  test("keepalive_maintained_after_idle — 品質検査イベントのないアイドル期間も接続が維持される", async ({ page }) => {
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
