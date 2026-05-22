// @conformanceAssert(["bulk_upload_at_least_once_idempotent", "backpressure_applied_correctly",
//   "half_close_terminates_cleanly", "large_msg_fragmented_correctly",
//   "continuous_stream_rate_50hz"])
// Spec 09: 計量装置連続データ (v1_bulk_upload) — 01_Bidi 適合仕様 §158 ship blocker 9/9
// 計量装置から 50Hz (20ms 間隔) で連続送信される計測データの assertion を検証する

// @playwright/test から test をインポートする
import { test } from "@playwright/test";
// support/containers からスタック起動ヘルパーをインポートする
import type { ManufacturingStack } from "../support/containers";
import { startManufacturingStack } from "../support/containers";
// slo_assertions から bulk_upload / stream_rate assertion をインポートする
import {
  assertBulkUploadIdempotent,
  assertContinuousStreamRate50hz,
} from "../support/assertions/slo_assertions";
// bidi_assertions から v1_bulk_upload 対応 assertion をインポートする
import {
  assertBackpressureApplied,
  assertHalfCloseTerminatesCleanly,
  assertLargeMsgFragmented,
} from "../support/assertions/bidi_assertions";

// テスト全体で共有するスタックインスタンスを保持する変数
let stack: ManufacturingStack;

// 全テスト開始前に計量装置向けスタックを起動する
test.beforeAll(async () => {
  // Kafka と PostgreSQL を有効化したスタックを起動する（計量データは高頻度）
  stack = await startManufacturingStack({
    adapters: ["grpc_native", "messaging_bridge"],
    requireKafka: true,
    enableRls: true,
  });
});

// 全テスト終了後にスタックを停止する
test.afterAll(async () => {
  // コンテナリソースを解放する
  await stack.stop();
});

// テストの説明ブロック: 計量装置連続データ v1_bulk_upload
test.describe("Spec 09: 計量装置連続データ (v1_bulk_upload)", () => {
  // テスト 1: at-least-once 冪等性確認 (計量データ重複排除)
  test("bulk_upload_at_least_once_idempotent — 計量データが重複送信されても冪等に受信される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // bulk_upload_at_least_once_idempotent assertion を実行する
    await assertBulkUploadIdempotent(page);
  });

  // テスト 2: 高頻度データ送信時の backpressure 確認
  test("backpressure_applied_correctly — 50Hz 送信時に backpressure が適用される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // backpressure_applied_correctly assertion を実行する
    await assertBackpressureApplied(page);
  });

  // テスト 3: 計量セッション終了後の half-close 確認
  test("half_close_terminates_cleanly — 計量セッション完了後にコネクションがクリーンに終了する", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // half_close_terminates_cleanly assertion を実行する
    await assertHalfCloseTerminatesCleanly(page);
  });

  // テスト 4: 大きな計量データバッチのフラグメンテーション確認
  test("large_msg_fragmented_correctly — 大きな計量データバッチが正しくフラグメント化される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // large_msg_fragmented_correctly assertion を実行する
    await assertLargeMsgFragmented(page);
  });

  // テスト 5: 50Hz ストリームレート維持確認
  test("continuous_stream_rate_50hz — 計量装置から 50Hz (20ms 間隔) でデータが継続的に届く", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // continuous_stream_rate_50hz assertion を実行する
    await assertContinuousStreamRate50hz(page);
  });
});
